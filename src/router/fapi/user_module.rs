use axum::extract::Query;
use axum::response::Redirect;
use axum::{extract::State, routing::get, Router};
use http::StatusCode;
use tracing::warn;

use crate::router::auth::AuthenticatedUser;
use crate::router::state::AppState;
use crate::service::ObfuscatedStr;

pub fn router() -> Router<AppState> {
    Router::new().route("/redeem-module", get(redeem_module))
}

async fn redeem_module(
    AuthenticatedUser(user): AuthenticatedUser,
    State(state): State<AppState>,
    Query(query): Query<RedeemQuery>,
) -> Result<Redirect, StatusCode> {
    let module_id = state
        .service
        .redeem_module(&query.key, &user)
        .await
        .map_err(|err| {
            warn!(
                "Unable to redeem module with key: {}, for user {user}: {err}",
                &query.key
            );
            StatusCode::FORBIDDEN
        })?;

    Ok(Redirect::to(&format!("/module/{}", module_id.uuid)))
}

#[derive(serde::Deserialize)]
struct RedeemQuery {
    key: ObfuscatedStr,
}
