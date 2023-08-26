use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

use crate::{config::Config, github, github::client_cache::ClientCache};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub cookie_key: Key,
    pub oauth: OAuth,
    pub github_clients: ClientCache,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}

impl AppState {
    pub fn new(config: &Config) -> Self {
        let gh_app_client = github::create_gh_app_client(config);

        Self {
            config: config.clone(),
            cookie_key: Key::derive_from(config.cookie_secret_key.as_ref()),
            oauth: OAuth::new(config),
            github_clients: ClientCache::new(
                gh_app_client,
                config.github_client_cache_size,
                config.github_app_id,
            ),
        }
    }
}

#[derive(Clone)]
pub struct OAuth {
    pub gh_client: BasicClient,
    pub redirect_url: RedirectUrl,
}

impl OAuth {
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let github_client_id = ClientId::new(config.github_app_client_id.to_string());
        let github_client_secret = ClientSecret::new(config.github_app_client_secret.to_string());
        let gh_auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
            .expect("Invalid authorization endpoint URL");
        let gh_token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
            .expect("Invalid token endpoint URL");

        let gh_client = BasicClient::new(
            github_client_id,
            Some(github_client_secret),
            gh_auth_url,
            Some(gh_token_url),
        );

        let redirect_url = RedirectUrl::new(config.github_app_redirect_url.clone())
            .expect("Unparseable GH redirect URL");
        Self {
            gh_client,
            redirect_url,
        }
    }
}

impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}