use axum::extract::State;
use axum::{
    routing::{get, patch},
    Json, Router,
};
use http::StatusCode;
use tracing::{error, warn};

use crate::router::auth::AuthenticatedUser;
use crate::router::state::AppState;

mod admin;
mod teacher;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/user/self", get(user_self))
        .route("/settings/redeem_code", patch(redeem_code))
        .nest("/admin", admin::router())
        .nest("/teacher", teacher::router())
        .fallback(crate::router::fallback)
}

#[allow(clippy::unused_async)]
async fn user_self(AuthenticatedUser(user): AuthenticatedUser) -> Json<User> {
    let mut role = if user.admin { "Admin" } else { "Student" }.to_string();
    if user.teacher {
        role.push_str(" & Teacher");
    }
    Json(User {
        firstname: user.first_name,
        lastname: user.last_name,
        school_group: user.school_group,
        school_email: user.school_email,
        role,
        avatar_url: user.avatar_url,
        installation_id: user.installation_id,
        admin: user.admin,
        teacher: user.teacher,
    })
}

async fn redeem_code(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    payload: String,
) -> Result<(), StatusCode> {
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    if state.instance_secret.eq(&payload) {
        state
            .service
            .repo
            .set_user_admin(&user.0.id)
            .await
            .map_err(|err| {
                error!("{err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        Ok(())
    } else {
        warn!(
            "Provided code was: {}, but expected: {}",
            &payload, state.instance_secret
        );
        Err(StatusCode::FORBIDDEN)
    }
}

#[derive(Debug, serde::Serialize)]
struct User {
    firstname: String,
    lastname: String,
    school_group: String,
    school_email: String,
    role: String,
    avatar_url: String,
    installation_id: Option<String>,
    admin: bool,
    teacher: bool,
}
