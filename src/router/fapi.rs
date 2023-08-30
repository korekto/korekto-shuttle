use crate::router::auth::AuthenticatedUser;
use axum::{routing::get, Json, Router};

use crate::router::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/user/self", get(user_self))
        .fallback(crate::router::fallback)
}

#[allow(clippy::unused_async)]
async fn user_self(AuthenticatedUser(user): AuthenticatedUser) -> Json<User> {
    Json(User {
        name: user,
        role: String::from("Admin, for sure"),
        avatar_url: String::from("https://avatars.githubusercontent.com/u/6298315?v=4"),
    })
}

#[derive(Debug, serde::Serialize)]
struct User {
    name: String,
    role: String,
    avatar_url: String,
}
