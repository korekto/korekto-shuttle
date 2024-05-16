use shuttle_runtime::SecretStore;
use std::num::{NonZeroU16, NonZeroU32, NonZeroUsize};

#[derive(serde::Deserialize, Clone)]
pub struct Config {
    pub cookie_secret_key: String,
    pub github_app_id: u64,
    pub github_app_name: String,
    pub github_app_client_id: String,
    pub github_app_client_secret: String,
    pub github_app_redirect_url: String,
    pub github_app_private_key: String,
    pub github_app_webhook_secret: String,
    pub github_client_cache_size: NonZeroUsize,
    #[serde(default = "default_scheduler_interval_in_secs")]
    pub scheduler_interval_in_secs: NonZeroU32,
    #[serde(default = "default_min_grading_interval_in_secs")]
    pub min_grading_interval_in_secs: NonZeroU16,
    #[serde(default = "default_max_parallel_gradings")]
    pub max_parallel_gradings: NonZeroU16,
}

fn default_scheduler_interval_in_secs() -> NonZeroU32 {
    #[allow(clippy::expect_used)]
    NonZeroU32::new(15).expect("Infallible !")
}

fn default_min_grading_interval_in_secs() -> NonZeroU16 {
    #[allow(clippy::expect_used)]
    NonZeroU16::new(15 * 60).expect("Infallible !")
}

fn default_max_parallel_gradings() -> NonZeroU16 {
    #[allow(clippy::expect_used)]
    NonZeroU16::new(3).expect("Infallible !")
}

impl Config {
    #[must_use]
    pub fn scheduler_interval_in_secs(&self) -> u64 {
        u64::from(self.scheduler_interval_in_secs.get())
    }

    #[must_use]
    pub fn min_grading_interval_in_secs(&self) -> i32 {
        i32::from(self.min_grading_interval_in_secs.get())
    }

    #[must_use]
    pub fn max_parallel_gradings(&self) -> i32 {
        i32::from(self.max_parallel_gradings.get())
    }
}

impl TryFrom<SecretStore> for Config {
    type Error = anyhow::Error;

    fn try_from(value: SecretStore) -> Result<Self, Self::Error> {
        Ok(envy::from_iter::<_, Self>(value)?)
    }
}
