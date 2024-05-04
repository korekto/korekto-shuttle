use crate::entities::{Assignment, InstantGrade, User, UserAssignment};
use crate::repository::Repository;
use anyhow::anyhow;
use const_format::formatcp;
use sqlx::types::Json;

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
            SELECT a.*, a.uuid::varchar as uuid, type as a_type from upserted u
            JOIN assignment a ON a.id = u.assignment_id
        ";

        Ok(sqlx::query_as::<_, Assignment>(QUERY)
            .bind(provider_login)
            .bind(repositories)
            .bind(linked)
            .fetch_all(&self.pool)
            .await?)
    }

    pub async fn update_assignment_grade(
        &self,
        user_uuid: &str,
        assignment_uuid: &str,
        grade: &InstantGrade,
    ) -> anyhow::Result<()> {
        const QUERY: &str = "\
            UPDATE user_assignment ua
            SET
              updated_at = $3,
              normalized_grade = GREATEST($4::NUMERIC(4, 2), normalized_grade),
              grades_history = grades_history || $5
            FROM assignment a, \"user\" u
            WHERE
              ua.assignment_id = a.id
              AND ua.user_id = u.id
              AND u.uuid::varchar = $1
              AND a.uuid::varchar = $2
        ";

        let normalized_grade = grade.grade * 20.0 / grade.max_grade;

        sqlx::query(QUERY)
            .bind(user_uuid)
            .bind(assignment_uuid)
            .bind(grade.time)
            .bind(normalized_grade)
            .bind(Json(grade))
            .execute(&self.pool)
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
              ua.grades_history
            FROM assignment a
            INNER JOIN module m ON m.id = a.module_id
            LEFT JOIN user_assignment ua ON ua.assignment_id = a.id
            LEFT JOIN \"user\" u ON u.id = ua.user_id
            WHERE u.id = $1
              AND m.uuid::varchar = $2
              AND a.uuid::varchar = $3
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
}
