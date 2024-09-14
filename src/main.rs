use korekto::shuttle::{KorektoService, KorektoServiceResult};
use korekto::{config::Config, router, router::state::AppState, scheduler::Scheduler};
use shuttle_runtime::SecretStore;
use sqlx::PgPool;

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
    #[shuttle_shared_db::Postgres] pool: PgPool,
) -> KorektoServiceResult {
    let config = Config::try_from(secret_store)?;
    let state = AppState::new(&config, pool).await?;
    korekto::tracing::setup()?;

    state.service.repo.run_migrations().await?;

    let router = router::router(state.clone());
    let scheduler = Scheduler::new(state);

    KorektoService::new(router, scheduler)
}
