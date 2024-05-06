use crate::entities::NewGradingTask;
use crate::repository::Repository;
use time::OffsetDateTime;

impl Repository {
    pub async fn upsert_grading_task(
        &self,
        task: &NewGradingTask,
    ) -> anyhow::Result<OffsetDateTime> {
        match task {
            NewGradingTask::Internal {
                user_assignment_id,
                user_provider_name,
                repository,
                grader_repository,
            } => {
                self.upsert_grading_task_internal(
                    *user_assignment_id,
                    user_provider_name,
                    repository,
                    grader_repository,
                )
                .await
            }
            NewGradingTask::External {
                user_assignment_uuid,
                user_uuid,
            } => {
                self.upsert_grading_task_external(user_assignment_uuid, user_uuid)
                    .await
            }
        }
    }

    async fn upsert_grading_task_internal(
        &self,
        user_assignment_id: i32,
        user_provider_name: &str,
        repository: &str,
        grader_repository: &str,
    ) -> anyhow::Result<OffsetDateTime> {
        const QUERY: &str = "INSERT INTO grading_task
          (user_assignment_id, user_provider_login, status, repository, grader_repository, updated_at)
        VALUES ($1, $2, 'queued', $3, $4, NOW())
        ON CONFLICT (user_assignment_id, user_provider_login, status) DO UPDATE
        SET updated_at= NOW()
        RETURNING updated_at";

        Ok(sqlx::query_scalar(QUERY)
            .bind(user_assignment_id)
            .bind(user_provider_name)
            .bind(repository)
            .bind(grader_repository)
            .fetch_one(&self.pool)
            .await?)
    }

    async fn upsert_grading_task_external(
        &self,
        user_assignment_uuid: &str,
        user_uuid: &str,
    ) -> anyhow::Result<OffsetDateTime> {
        const QUERY: &str = "INSERT INTO grading_task
          (user_assignment_id, user_provider_login, status, repository, grader_repository, updated_at)
        SELECT ua.id, u.provider_login, 'queued', a.repository_name, a.grader_url, NOW()
        FROM user_assignment ua, \"user\" u, assignment a,
        WHERE
          ua.user_id = u.id
          AND ua.assignment_id = a.id,
          AND ua.uuid::varchar = $1
          AND u.uuid::varchar = $2
        ON CONFLICT (user_assignment_id, user_provider_login, status) DO UPDATE
        SET updated_at= NOW()
        RETURNING updated_at";

        Ok(sqlx::query_scalar(QUERY)
            .bind(user_assignment_uuid)
            .bind(user_uuid)
            .fetch_one(&self.pool)
            .await?)
    }
}
