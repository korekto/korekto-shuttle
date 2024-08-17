use crate::config::Config;
use axum::Router;
use tracing::info;

#[cfg(feature = "sentry")]
pub fn configure_router(router: Router) -> Router {
    use axum::extract::Request;
    use std::sync::Arc;

    info!("Starting server with Sentry configuration");
    let hub = Arc::new(sentry::Hub::with(|hub| sentry::Hub::new_from_top(hub)));
    router.layer(sentry_tower::SentryLayer::<_, _, Request>::new(hub))
}

#[cfg(not(feature = "sentry"))]
pub fn configure_router(router: Router) -> Router {
    info!("Starting server without Sentry");
    router
}

#[cfg(feature = "sentry")]
#[derive(Clone)]
pub struct Holder {
    _guard: std::sync::Arc<sentry::ClientInitGuard>,
}

#[cfg(not(feature = "sentry"))]
#[derive(Clone)]
pub struct Holder {}

impl Holder {
    #[cfg(feature = "sentry")]
    #[must_use]
    pub fn new(config: &Config) -> Self {
        let guard = sentry::init((
            config.sentry_dsn.clone(),
            sentry::ClientOptions {
                release: sentry::release_name!(),
                ..Default::default()
            },
        ));
        Self {
            _guard: std::sync::Arc::new(guard),
        }
    }

    #[cfg(not(feature = "sentry"))]
    #[must_use]
    pub const fn new(_config: &Config) -> Self {
        Self {}
    }
}
