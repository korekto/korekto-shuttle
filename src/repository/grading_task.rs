use crate::entities::NewGradingTask;
use crate::repository::Repository;
use time::OffsetDateTime;

impl Repository {
    pub async fn upsert_grading_task(
        &self,
        task: &NewGradingTask,
    ) -> anyhow::Result<OffsetDateTime> {
        const QUERY: &str = "INSERT INTO grading_task
        (user_assignment_id, user_provider_name, repository, grader_repository, latest_code_update)
        VALUES ($1, $2, $3, $4, NOW())
        ON CONFLICT (user_assignment_id, user_provider_name) DO UPDATE
        SET latest_code_update= NOW()
        RETURNING latest_code_update";

        Ok(sqlx::query_scalar(QUERY)
            .bind(&task.user_assignment_id)
            .bind(&task.user_provider_name)
            .bind(&task.repository)
            .bind(&task.grader_repository)
            .fetch_one(&self.pool)
            .await?)
    }
}
