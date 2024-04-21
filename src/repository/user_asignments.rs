use crate::entities::Assignment;
use crate::repository::Repository;

impl Repository {
    pub async fn link_user_repositories_to_assignments(
        &self,
        provider_login: &str,
        repositories: &[&str],
    ) -> anyhow::Result<Vec<Assignment>> {
        const QUERY: &str = "\
            WITH MATCHING_ASSIGNMENT AS (
              SELECT *
              FROM \"assignment\"
              WHERE repository_name = ANY($2)
            )
            UPDATE user_assignment as ua SET
              repository_linked = TRUE
            FROM MATCHING_ASSIGNMENT ma
            INNER JOIN \"user\" u ON mu.id = ua2.user_id
            WHERE ua.assignment_id = ma.id
                AND u.provider_login = $1
            RETURNING ma.*
        ";

        Ok(sqlx::query_as::<_, Assignment>(QUERY)
            .bind(provider_login)
            .bind(repositories)
            .fetch_all(&self.pool)
            .await?)
    }
}
