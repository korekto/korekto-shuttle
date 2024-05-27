use crate::entities::{Assignment, InstantGrade, User, UserAssignment};
use crate::repository::Repository;
use anyhow::anyhow;
use const_format::formatcp;
use sqlx::types::Json;
use sqlx::{Executor, Postgres};

impl Repository {
    pub async fn upsert_user_assignments(
        &self,
        provider_login: &str,
        repositories: &[&str],
        linked: bool,
    ) -> anyhow::Result<Vec<Assignment>> {
        const QUERY: &str = "\
            WITH upserted AS (
              INSERT INTO user_assignment
                (user_id, assignment_id, repository_linked)
              SELECT
                u.id as user_id,
                a.id as assignment_id,
                $3 as repository_linked
              FROM assignment a
              JOIN \"user\" u ON u.provider_login = $1
              WHERE a.repository_name = ANY($2)
              ON CONFLICT (user_id, assignment_id) DO UPDATE
                SET repository_linked = $3
              RETURNING *
            )
            SELECT a.*, a.uuid::varchar as uuid, type as a_type
            FROM upserted u
            JOIN assignment a ON a.id = u.assignment_id
        ";

        Ok(sqlx::query_as::<_, Assignment>(QUERY)
            .bind(provider_login)
            .bind(repositories)
            .bind(linked)
            .fetch_all(&self.pool)
            .await?)
    }

    pub async fn update_assignment_grade_transact<'e, 'c: 'e, E>(
        user_assignment_id: i32,
        grade: &InstantGrade,
        transaction: E,
    ) -> anyhow::Result<()>
    where
        E: 'e + Executor<'c, Database = Postgres>,
    {
        const QUERY: &str = "\
            UPDATE user_assignment ua
            SET
              updated_at = $2,
              normalized_grade = GREATEST($3::NUMERIC(4, 2), normalized_grade),
              grades_history = grades_history || $4,
              graded_last_at = NOW()
            WHERE
              ua.id = $1
        ";

        let normalized_grade = grade.grade * 20.0 / grade.max_grade;

        sqlx::query(QUERY)
            .bind(user_assignment_id)
            .bind(grade.time)
            .bind(normalized_grade)
            .bind(Json(grade))
            .execute(transaction)
            .await?;
        Ok(())
    }

    pub async fn get_assignment(
        &self,
        user: &User,
        module_uuid: &str,
        assignment_uuid: &str,
    ) -> anyhow::Result<Option<UserAssignment>> {
        const QUERY: &str = formatcp!(
            "\
            SELECT
              a.id,
              a.uuid::varchar as uuid,
              a.name,
              a.description,
              a.start,
              a.stop,
              a.type as a_type,
              a.factor_percentage,
              a.subject_url,
              a.grader_url,
              a.repository_name,
              COALESCE(ua.repository_linked, FALSE) as repo_linked,
              u.provider_login as user_provider_login,
              COALESCE(ua.normalized_grade, 0)::real as normalized_grade,
              COALESCE(ua.grades_history, '[]'::jsonb) as grades_history,
              coalesce(json_agg(to_jsonb(gt.*) ORDER BY gt.created_at asc) FILTER (WHERE gt.id IS NOT NULL), '[]'::json) AS grading_tasks
            FROM assignment a
            INNER JOIN module m ON m.id = a.module_id
            INNER JOIN user_module um ON um.module_id = m.id
            INNER JOIN \"user\" u ON u.id = um.user_id
            LEFT JOIN user_assignment ua ON ua.assignment_id = a.id
            LEFT JOIN grading_task gt ON gt.user_assignment_id = ua.id
            WHERE u.id = $1
              AND m.uuid::varchar = $2
              AND a.uuid::varchar = $3
            GROUP BY a.id, ua.id, u.id
        "
        );

        sqlx::query_as::<_, UserAssignment>(QUERY)
            .bind(user.id)
            .bind(module_uuid)
            .bind(assignment_uuid)
            .fetch_optional(&self.pool)
            .await
            .map_err(|err| {
                anyhow!(
                    "get_assignment({:?}, {module_uuid}, {assignment_uuid}): {err:?}",
                    &user.provider_login
                )
            })
    }

    pub async fn update_assignment_current_grading_metadata<'e, 'c: 'e, E>(
        user_assignment_id: i32,
        short_commit_id: &str,
        commit_url: &str,
        full_log_url: &str,
        transaction: E,
    ) -> anyhow::Result<()>
    where
        E: 'e + Executor<'c, Database = Postgres>,
    {
        const QUERY: &str = "\
            UPDATE user_assignment ua
            SET
              updated_at = NOW(),
              running_grading_short_commit_id = $2,
              running_grading_commit_url = $3,
              running_grading_log_url = $4
            WHERE
              ua.id = $1
        ";

        sqlx::query(QUERY)
            .bind(user_assignment_id)
            .bind(short_commit_id)
            .bind(commit_url)
            .bind(full_log_url)
            .execute(transaction)
            .await?;
        Ok(())
    }
}
