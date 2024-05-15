use crate::router::state::AppState;
use std::time::Duration;
use tracing::info;

pub struct Scheduler {
    state: AppState,
}

impl Scheduler {
    pub const fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn start(&self) {
        let secs = self.state.config.scheduler_interval_in_secs.get() as u64;
        let mut interval = tokio::time::interval(Duration::from_secs(secs));
        info!("[scheduler] Starting scheduler every {secs} secs");

        loop {
            interval.tick().await;
            info!("[scheduler] Ticking");
        }
    }
}
