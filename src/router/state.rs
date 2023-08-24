use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub cookie_key: Key,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}

impl AppState {
    pub fn new(config: &Config) -> Self {
        Self {
            cookie_key: Key::derive_from(config.cookie_secret_key.as_ref()),
        }
    }
}
