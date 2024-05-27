use crate::router::state::AppState;
use std::time::Duration;
use tracing::{error, info};

pub struct Scheduler {
    state: AppState,
}

impl Scheduler {
    pub const fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn start(&self) {
        let secs = self.state.config.scheduler_interval_in_secs;
        let mut interval = tokio::time::interval(Duration::from_secs(secs));
        info!("[scheduler] Starting scheduler every {secs} secs");

        loop {
            interval.tick().await;
            if let Err(err) = self.tick().await {
                error!("Scheduler encountered an error: {err:?}");
            }
        }
    }

    pub async fn tick(&self) -> anyhow::Result<()> {
        let stats = self
            .state
            .service
            .schedule_tasks(
                self.state.config.min_grading_interval_in_secs,
                self.state.config.grading_ordered_timeout_in_secs,
                self.state.config.grading_started_timeout_in_secs,
                self.state.config.max_parallel_gradings,
                &self.state.gh_runner,
            )
            .await?;
        info!(
            "[scheduler] Ticking, {stats} (min queue time={} sec)",
            self.state.config.min_grading_interval_in_secs
        );
        Ok(())
    }
}
