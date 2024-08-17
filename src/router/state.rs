use anyhow::anyhow;
use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use sqlx::PgPool;
use std::num::NonZeroUsize;
use std::str::FromStr;
use uuid::Uuid;

use crate::github::{runner::Runner, GitRepoSlug};
use crate::service::Service;
use crate::{config::Config, github, github::client_cache::ClientCache};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub cookie_key: Key,
    pub oauth: OAuth,
    pub github_clients: ClientCache,
    pub service: Service,
    pub instance_secret: String,
    pub gh_runner: Runner,
    _sentry: crate::sentry::Holder,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}

impl AppState {
    pub async fn new(config: &Config, pool: PgPool) -> anyhow::Result<Self> {
        let instance_secret = Uuid::new_v4().to_string();
        tracing::info!("Instance secret: {}", &instance_secret);

        let slug = GitRepoSlug::from_str(&config.github_runner_repo_slug)?;

        let gh_runner_app_client = github::create_gh_app_client(
            config.github_runner_app_id,
            &config.github_runner_app_private_key,
        )?;

        let runner_client =
            gh_runner_app_client.installation(config.github_runner_installation_id.into());

        let gh_runner = Runner::new(
            slug.org,
            slug.repo,
            gh_runner_app_client,
            runner_client,
            config.clone(),
        )
        .await?;

        let gh_app_client =
            github::create_gh_app_client(config.github_app_id, &config.github_app_private_key)?;
        let github_clients = ClientCache::new(
            gh_app_client,
            NonZeroUsize::new(config.github_client_cache_size)
                .ok_or_else(|| anyhow!("Configured GITHUB_CLIENT_CACHE_SIZE must be > 0"))?,
            config.github_app_id,
        );
        let sentry = crate::sentry::Holder::new(config);

        Ok(Self {
            config: config.clone(),
            cookie_key: config
                .cookie_secret_key
                .clone()
                .map_or_else(Key::generate, |src| Key::derive_from(src.as_ref())),
            oauth: OAuth::new(config)?,
            github_clients,
            service: Service::new(pool),
            instance_secret,
            gh_runner,
            _sentry: sentry,
        })
    }
}

#[derive(Clone)]
pub struct OAuth {
    pub gh_client: BasicClient,
    pub redirect_url: RedirectUrl,
}

impl OAuth {
    fn new(config: &Config) -> anyhow::Result<Self> {
        let github_client_id = ClientId::new(config.github_app_client_id.to_string());
        let github_client_secret = ClientSecret::new(config.github_app_client_secret.to_string());
        let gh_auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
            .map_err(|_| anyhow!("Invalid authorization endpoint URL"))?;
        let gh_token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
            .map_err(|_| anyhow!("Invalid token endpoint URL"))?;

        let gh_client = BasicClient::new(
            github_client_id,
            Some(github_client_secret),
            gh_auth_url,
            Some(gh_token_url),
        );

        let redirect_url = RedirectUrl::new(format!("{}/auth/gh/authorized", config.base_url))
            .map_err(|_| anyhow!("Unparseable GH redirect URL"))?;
        Ok(Self {
            gh_client,
            redirect_url,
        })
    }
}

impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}
