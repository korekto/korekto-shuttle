use axum::extract::Path;
use axum::{extract::State, routing::get, Json, Router};
use http::StatusCode;
use tracing::error;

use crate::entities::{Module, NewModule};
use crate::{
    entities::ModuleDesc,
    router::{auth::TeacherUser, state::AppState},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/module", get(get_modules).post(create_module))
        .route(
            "/module/:module_id",
            get(get_module).put(update_module),
            //.delete(delete_modules)
        )
    //.route("/module/:module_id/assignment", post(create_assignment))
    //.route("/module/:module_id/assignment/:assignment_id", get(get_assignment).put(update_assignment).delete(delete_assignments))
}

async fn get_modules(
    _user: TeacherUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<ModuleDesc>>, StatusCode> {
    let modules = state.service.repo.find_modules().await.map_err(|err| {
        error!("{err:#?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(modules))
}

async fn create_module(
    _user: TeacherUser,
    State(state): State<AppState>,
    Json(module): Json<NewModule>,
) -> Result<Json<Module>, StatusCode> {
    let module = state
        .service
        .repo
        .create_module(&module)
        .await
        .map_err(|err| {
            error!("{err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(module))
}

async fn get_module(
    _user: TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
) -> Result<Json<Module>, StatusCode> {
    let module = state
        .service
        .repo
        .find_module(&module_id)
        .await
        .map_err(|err| {
            error!("{err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(module))
}

async fn update_module(
    _user: TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
    Json(module): Json<NewModule>,
) -> Result<Json<Module>, StatusCode> {
    let module = state
        .service
        .repo
        .update_module(&module_id, &module)
        .await
        .map_err(|err| {
            error!("{err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(module))
}
