use crate::config::Config;
use crate::router::state::AppState;
use shuttle_secrets::SecretStore;
use sqlx::PgPool;
use std::path::PathBuf;

mod config;
mod entities;
mod github;
mod repository;
mod router;
mod service;

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn main(
    #[shuttle_static_folder::StaticFolder] static_folder: PathBuf,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_aws_rds::Postgres] pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    let config = Config::try_from(secret_store)?;
    let state = AppState::new(&config, pool)?;

    state.service.repo.reset_migrations().await?;
    state.service.repo.run_migrations().await?;

    let router = router::router(&static_folder, state);

    Ok(router)
}
