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

        let tasks = Repository::reserve_grading_tasks_to_execute_transact(
            min_execution_interval_in_secs,
            max_tasks,
            &mut *transaction,
        )
        .await?;

        for task in &tasks {
            let send_result = runner
                .send_grading_command(task)
                .await
                .map_err(|err| format!("not ordered: {err:?}"));

            match send_result {
                Ok(()) => {
                    Repository::update_grading_task_non_terminal_status_transact(
                        &task.uuid,
                        &Status::ORDERED,
                        &mut *transaction,
                    )
                    .await?;
                    info!("Grading task ordered");
                }
                Err(err) => {
                    warn!("Grading task errored: {err}");
                    Repository::delete_grading_task_transact(
                        &task.uuid,
                        Some(err),
                        &mut *transaction,
                    )
                    .await?;
                }
            }
        }

        transaction.commit().await?;

        Ok(tasks.len())
    }
}
