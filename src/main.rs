use std::path::PathBuf;

mod router;

#[shuttle_runtime::main]
async fn main(
    #[shuttle_static_folder::StaticFolder] static_folder: PathBuf,
) -> shuttle_axum::ShuttleAxum {
    let router = router::router(static_folder);

    Ok(router)
}
