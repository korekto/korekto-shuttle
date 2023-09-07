use axum::{extract::State, routing::get, Router};
use http::StatusCode;

use crate::{
    entities::Table,
    router::{auth::AdminUser, state::AppState},
};

pub fn router() -> Router<AppState> {
    Router::new().route("/table", get(get_tables))
}

async fn get_tables(
    _user: AdminUser,
    State(state): State<AppState>,
) -> Result<axum::Json<Vec<Table>>, StatusCode> {
    let users = state
        .service
        .repo
        .find_tables()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(axum::Json(users))
}
