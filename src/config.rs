use shuttle_runtime::SecretStore;
use std::num::NonZeroUsize;

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
    pub scheduler_interval_in_secs: NonZeroUsize,
}

fn default_scheduler_interval_in_secs() -> NonZeroUsize {
    #[allow(clippy::expect_used)]
    NonZeroUsize::new(15).expect("Infallible !")
}

impl TryFrom<SecretStore> for Config {
    type Error = anyhow::Error;

    fn try_from(value: SecretStore) -> Result<Self, Self::Error> {
        Ok(envy::from_iter::<_, Self>(value)?)
    }
}
