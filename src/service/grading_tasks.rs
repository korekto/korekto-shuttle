use crate::github;
use crate::repository::{grading_task::GradingStatus, Repository};
use crate::service::Service;
use std::fmt;
use tracing::warn;

#[derive(Default)]
pub struct TaskStats {
    pub ordered: i32,
    pub errored: i32,
    pub ordered_timeout: i32,
    pub started_timeout: i32,
}

impl TaskStats {
    pub const fn total(&self) -> i32 {
        self.ordered + self.errored
    }
}

impl fmt::Display for TaskStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Tasks: 🚀 ordered={}, ❌ errored={}, ⏱️ ordered_timeout={}, ⏱️ started_timeout={}",
            self.ordered, self.errored, self.ordered_timeout, self.started_timeout
        )
    }
}

impl Service {
    pub async fn schedule_tasks(
        &self,
        min_execution_interval_in_secs: i32,
        ordered_timeout_in_secs: i32,
        started_timeout_in_secs: i32,
        max_tasks: i32,
        runner: &github::runner::Runner,
    ) -> anyhow::Result<TaskStats> {
        let mut stats = TaskStats::default();

        self.launch_grading_tasks(
            min_execution_interval_in_secs,
            max_tasks,
            runner,
            &mut stats,
        )
        .await?;

        stats.ordered_timeout += self
            .repo
            .timeout_grading_tasks(&GradingStatus::ORDERED, ordered_timeout_in_secs)
            .await?;
        stats.started_timeout += self
            .repo
            .timeout_grading_tasks(&GradingStatus::STARTED, started_timeout_in_secs)
            .await?;

        Ok(stats)
    }

    async fn launch_grading_tasks(
        &self,
        min_execution_interval_in_secs: i32,
        max_tasks: i32,
        runner: &github::runner::Runner,
        stats: &mut TaskStats,
    ) -> anyhow::Result<()> {
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
                        &GradingStatus::ORDERED,
                        &mut *transaction,
                    )
                    .await?;
                    stats.ordered += 1;
                }
                Err(err) => {
                    warn!("Grading task errored: {err}");
                    stats.errored += 1;
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

        Ok(())
    }
}
