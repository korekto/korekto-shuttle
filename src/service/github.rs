use crate::entities::GitHubGradingTask;
use crate::github;
use crate::repository::{grading_task::Status, Repository};
use crate::service::Service;
use tracing::{info, warn};

impl Service {
    pub async fn launch_grading_tasks(
        &self,
        min_execution_interval_in_secs: i32,
        max_tasks: i32,
        runner: &github::runner::Runner,
    ) -> anyhow::Result<usize> {
        let mut transaction = self.repo.start_transaction().await?;

        let tasks = Repository::reserve_grading_tasks_to_execute(
            min_execution_interval_in_secs,
            max_tasks,
            &mut *transaction,
        )
        .await?;

        let queued_tasks: Vec<GitHubGradingTask> =
            tasks.into_iter().filter(|t| t.status == "queued").collect();

        for task in &queued_tasks {
            let error_message = runner
                .send_grading_command(task)
                .await
                .err()
                .map(|err| format!("not ordered: {err:?}"));

            Repository::update_grading_task_status(
                &task.uuid,
                if error_message.is_none() {
                    info!("Grading task ordered");
                    &Status::ORDERED
                } else {
                    warn!("Grading task errored: {:?}", &error_message);
                    &Status::ERROR
                },
                &error_message,
                &mut *transaction,
            )
            .await?;
        }

        transaction.commit().await?;

        Ok(queued_tasks.len())
    }
}
