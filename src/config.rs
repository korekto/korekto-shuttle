use anyhow::anyhow;
use shuttle_secrets::SecretStore;

pub struct Config {
    pub cookie_secret_key: String,
}

impl TryFrom<SecretStore> for Config {
    type Error = anyhow::Error;

    fn try_from(value: SecretStore) -> Result<Self, Self::Error> {
        fn get_secret(secret_store: &SecretStore, key: &str) -> Result<String, anyhow::Error> {
            secret_store
                .get(key)
                .ok_or_else(|| anyhow!("Missing secret {}", key))
        }
        Ok(Config {
            cookie_secret_key: get_secret(&value, "COOKIE_SECRET_KEY")?,
        })
    }
}
