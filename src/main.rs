use crate::config::Config;
use shuttle_secrets::SecretStore;
use std::path::PathBuf;

mod config;
mod github;
mod router;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_static_folder::StaticFolder] static_folder: PathBuf,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let config = Config::try_from(secret_store)?;
    let router = router::router(static_folder, &config);

    Ok(router)
}
