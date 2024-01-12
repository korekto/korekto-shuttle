use axum::extract::Path;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use http::StatusCode;
use tracing::{error, info};

use crate::{
    entities::{Assignment, Module, ModuleDesc, NewAssignment, NewModule},
    router::{auth::TeacherUser, state::AppState},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/module",
            get(get_modules).post(create_module).delete(delete_modules),
        )
        .route("/module/:module_id", get(get_module).put(update_module))
        .route(
            "/module/:module_id/assignment",
            post(create_assignment).delete(delete_assignments),
        )
        .route(
            "/module/:module_id/assignment/:assignment_id",
            get(get_assignment).put(update_assignment),
        )
}

async fn get_modules(
    _user: TeacherUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<ModuleDesc>>, StatusCode> {
    let modules = state.service.repo.find_modules().await.map_err(|err| {
        info!("toto");
        error!("get_modules {err:#?}");
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
            error!("create_module {err:#?}");
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
            error!("get_module {err:#?}");
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
            error!("update_module {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(module))
}

async fn delete_modules(
    _user: TeacherUser,
    State(state): State<AppState>,
    Json(module_ids): Json<Vec<String>>,
) -> Result<(), StatusCode> {
    state
        .service
        .repo
        .delete_modules(&module_ids)
        .await
        .map_err(|err| {
            error!("delete_modules {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}

async fn create_assignment(
    _user: TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
    Json(assignment): Json<NewAssignment>,
) -> Result<Json<Assignment>, StatusCode> {
    let assignment = state
        .service
        .repo
        .create_assignment(&module_id, &assignment)
        .await
        .map_err(|err| {
            error!("create_assignment {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(assignment))
}

async fn get_assignment(
    _user: TeacherUser,
    State(state): State<AppState>,
    Path((module_id, assignment_id)): Path<(String, String)>,
) -> Result<Json<Assignment>, StatusCode> {
    let assignment = state
        .service
        .repo
        .find_assignment(&module_id, &assignment_id)
        .await
        .map_err(|err| {
            error!("get_assignment {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(assignment))
}

async fn update_assignment(
    _user: TeacherUser,
    State(state): State<AppState>,
    Path((module_id, assignment_id)): Path<(String, String)>,
    Json(assignment): Json<NewAssignment>,
) -> Result<Json<Assignment>, StatusCode> {
    let assignment = state
        .service
        .repo
        .update_assignment(&module_id, &assignment_id, &assignment)
        .await
        .map_err(|err| {
            error!("update_assignment {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(assignment))
}

async fn delete_assignments(
    _user: TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
    Json(assignment_ids): Json<Vec<String>>,
) -> Result<(), StatusCode> {
    state
        .service
        .repo
        .delete_assignments(&module_id, &assignment_ids)
        .await
        .map_err(|err| {
            error!("delete_assignments {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}
