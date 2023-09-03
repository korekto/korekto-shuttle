use axum::{routing::get, Json, Router};

use crate::router::auth::AuthenticatedUser;
use crate::router::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/user/self", get(user_self))
        .fallback(crate::router::fallback)
}

#[allow(clippy::unused_async)]
async fn user_self(AuthenticatedUser(user): AuthenticatedUser) -> Json<User> {
    let mut role = if user.admin { "Admin" } else { "Student" }.to_string();
    if user.teacher {
        role.push_str(" & Teacher");
    }
    Json(User {
        name: user.name,
        role,
        avatar_url: user.avatar_url,
        installation_id: user.installation_id,
    })
}

#[derive(Debug, serde::Serialize)]
struct User {
    name: String,
    role: String,
    avatar_url: String,
    installation_id: Option<String>,
}
