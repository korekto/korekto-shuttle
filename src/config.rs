use anyhow::anyhow;
use shuttle_runtime::SecretStore;
use std::num::NonZeroUsize;

#[derive(Clone)]
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
}

impl TryFrom<SecretStore> for Config {
    type Error = anyhow::Error;

    fn try_from(value: SecretStore) -> Result<Self, Self::Error> {
        fn get_secret(secret_store: &SecretStore, key: &str) -> Result<String, anyhow::Error> {
            secret_store
                .get(key)
                .ok_or_else(|| anyhow!("Missing secret {}", key))
        }
        Ok(Self {
            cookie_secret_key: get_secret(&value, "COOKIE_SECRET_KEY")?,
            github_app_id: get_secret(&value, "GITHUB_APP_ID")?
                .parse()
                .map_err(|_| anyhow!("GITHUB_APP_ID should be a number (u64)"))?,
            github_app_name: get_secret(&value, "GITHUB_APP_NAME")?,
            github_app_client_id: get_secret(&value, "GITHUB_APP_CLIENT_ID")?,
            github_app_client_secret: get_secret(&value, "GITHUB_APP_CLIENT_SECRET")?,
            github_app_redirect_url: get_secret(&value, "GITHUB_APP_REDIRECT_URL")?,
            github_app_private_key: get_secret(&value, "GITHUB_APP_PRIVATE_KEY")?,
            github_app_webhook_secret: get_secret(&value, "GITHUB_APP_WEBHOOK_SECRET")?,
            github_client_cache_size: get_secret(&value, "GITHUB_CLIENT_CACHE_SIZE")?
                .parse()
                .map_err(|_| anyhow!("GITHUB_APP_PRIVATE_KEY should be a number (usize > 0)"))?,
        })
    }
}
