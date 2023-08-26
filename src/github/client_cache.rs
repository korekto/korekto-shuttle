use anyhow::anyhow;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use lru::LruCache;
use oauth2::{basic::BasicTokenResponse, TokenResponse};

use crate::github::client::InstallationClient;
use crate::github::GitHubUser;

#[derive(Clone)]
pub struct ClientCache {
    app_client: octocrab::Octocrab,
    inner_cache: Arc<Mutex<LruCache<u64, InstallationClient>>>,
    app_id: u64,
}

impl ClientCache {
    pub fn new(app_client: octocrab::Octocrab, size: NonZeroUsize, app_id: u64) -> Self {
        Self {
            app_client,
            inner_cache: Arc::new(Mutex::new(LruCache::new(size))),
            app_id,
        }
    }

    pub fn get_for_installation(&self, installation_id: u64) -> anyhow::Result<InstallationClient> {
        let cloned_arc = Arc::clone(&self.inner_cache);
        let mut inner_cache = cloned_arc
            .lock()
            .map_err(|_| anyhow!("Previous thread using the mutex panicked"))?;

        let installation_client = inner_cache.get(&installation_id);

        #[allow(clippy::option_if_let_else)]
        Ok(if let Some(installation_client) = installation_client {
            installation_client.clone()
        } else {
            let gh_installation_client =
                InstallationClient::new(self.app_client.installation(installation_id.into()));
            inner_cache.put(installation_id, gh_installation_client.clone());
            gh_installation_client
        })
    }

    pub async fn get_user_info(
        &self,
        token_response: &BasicTokenResponse,
    ) -> anyhow::Result<GitHubUser> {
        let user_token = token_response.access_token().secret().to_string();
        let gh_user_client = octocrab::Octocrab::builder()
            .personal_token(user_token)
            .build()?;

        let gh_user = gh_user_client.current().user().await?;

        let user_installations_page_1 = gh_user_client
            .current()
            .list_app_installations_accessible_to_user()
            .send()
            .await?;
        drop(gh_user_client);

        let installation = user_installations_page_1
            .items
            .into_iter()
            .find(|i| i.app_id.is_some_and(|id| id.0 == self.app_id))
            .ok_or_else(|| anyhow!("Could not find matching installation"))?;

        Ok(GitHubUser {
            login: gh_user.login,
            installation_id: installation.id.to_string(),
            avatar_url: gh_user.avatar_url.to_string(),
        })
    }
}
