use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use lru::LruCache;
use oauth2::{basic::BasicTokenResponse, TokenResponse};
use tracing::warn;

use crate::entities::GitHubUserTokens;
use crate::{
    entities::User,
    github::{client::GitHubClient, GitHubUserLogged},
};

#[derive(Clone)]
pub struct ClientCache {
    app_client: octocrab::Octocrab,
    inner_cache: Arc<Mutex<LruCache<u64, GitHubClient>>>,
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

    pub fn get_for_installation(&self, installation_id: u64) -> anyhow::Result<GitHubClient> {
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
                GitHubClient::new(self.app_client.installation(installation_id.into()));
            inner_cache.put(installation_id, gh_installation_client.clone());
            gh_installation_client
        })
    }

    pub async fn get_user_info(
        &self,
        token_response: &BasicTokenResponse,
    ) -> anyhow::Result<GitHubUserLogged> {
        let user_token = token_response.access_token().secret().to_string();
        let gh_user_client = octocrab::Octocrab::builder()
            .personal_token(user_token)
            .build()?;

        let gh_user = GitHubClient::new(gh_user_client.clone())
            .current_user()
            .await?;

        Ok(GitHubUserLogged {
            login: gh_user.login,
            name: gh_user.name,
            avatar_url: gh_user.avatar_url,
            email: gh_user.email,
        })
    }

    pub async fn get_user_installation_id(&self, user: &User) -> Option<String> {
        async fn get_user_installation_id_internal(
            client_cache: &ClientCache,
            user_tokens: &GitHubUserTokens,
            provider_login: &str,
        ) -> anyhow::Result<String> {
            let gh_user_client = octocrab::Octocrab::builder()
                .personal_token(user_tokens.access_token.value.clone())
                .build()?;
            let user_installations_page_1 = gh_user_client
                .current()
                .list_app_installations_accessible_to_user()
                .send()
                .await?;
            drop(gh_user_client);

            let installation_id = user_installations_page_1
                .items
                .into_iter()
                .find(|i| i.app_id.is_some_and(|id| id.0 == client_cache.app_id))
                .map(|i| i.id.to_string())
                .ok_or_else(|| {
                    anyhow!(
                        "Installation of app (id: {}) not found for User {}",
                        client_cache.app_id,
                        provider_login
                    )
                })?;

            Ok(installation_id)
        }

        if let Some(user_tokens) = &user.github_user_tokens {
            let installation_id_result =
                get_user_installation_id_internal(self, user_tokens, &user.provider_login).await;
            match installation_id_result {
                Ok(installation_id) => Some(installation_id),
                Err(err) => {
                    warn!("{err}");
                    None
                }
            }
        } else {
            None
        }
    }
}
