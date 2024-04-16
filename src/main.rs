use korekto::{config::Config, router, router::state::AppState};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    let config = Config::try_from(secret_store)?;
    let state = AppState::new(&config, pool)?;

    state.service.repo.reset_migrations().await?;
    state.service.repo.run_migrations().await?;

    let router = router::router(state);

    Ok(router)
}
