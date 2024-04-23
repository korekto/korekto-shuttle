use crate::entities::Assignment;
use crate::repository::Repository;
use time::OffsetDateTime;

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
        user_id: i32,
        assignment_id: i32,
        grade: f32,
        updated_date: &OffsetDateTime,
    ) -> anyhow::Result<()> {
        const QUERY: &str = "\
            UPDATE user_assignment ua
            SET grade = $3, updated_at = $4
            WHERE ua.user_id = $1
            AND ua.assignment_id = $2
        ";

        sqlx::query(QUERY)
            .bind(user_id)
            .bind(assignment_id)
            .bind(grade)
            .bind(updated_date)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
