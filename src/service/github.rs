use crate::github::client_cache::ClientCache;
use crate::service::Service;
use tracing::{info, warn};

impl Service {
    pub async fn resync_github(&self, clients_cache: &ClientCache) -> anyhow::Result<()> {
        let installations = clients_cache.list_accessible_installations().await?;
        for installation in installations {
            let user_result = self
                .repo
                .find_user_by_provider_login(&installation.login)
                .await;
            if let Ok(user) = user_result {
                if user.installation_id.unwrap_or_default() == installation.installation_id {
                    info!(
                        "Already correct provider installation ID for user : {} - {}",
                        installation.login, installation.installation_id
                    );
                } else {
                    info!(
                        "Updating provider installation ID for user : {} - {}",
                        installation.login, installation.installation_id
                    );
                    self.repo
                        .update_installation_id(&user.id, &installation.installation_id)
                        .await?;
                }
            } else {
                warn!("No user for provider login: {}", installation.login);
            }
        }
        Ok(())
    }
}
