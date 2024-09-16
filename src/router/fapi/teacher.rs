use axum::extract::Path;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use http::StatusCode;
use tracing::error;

use crate::service::dtos::{
    ModuleGradesResponse, TeacherAssignmentResponse, TeacherModuleDescResponse,
    TeacherModuleResponse, VecInto,
};
use crate::{
    entities::{NewAssignment, NewModule},
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
        .route("/module/:module_id/grade", get(get_grades))
        .route(
            "/module/:module_id/assignment/:assignment_id",
            get(get_assignment).put(update_assignment),
        )
}

async fn get_modules(
    TeacherUser(user): TeacherUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<TeacherModuleDescResponse>>, StatusCode> {
    let modules = state
        .service
        .repo
        .find_modules(&user)
        .await
        .map_err(|err| {
            error!(error = ?err, %user, "[http] get_modules");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(modules.vec_into()))
}

async fn create_module(
    TeacherUser(user): TeacherUser,
    State(state): State<AppState>,
    Json(module): Json<NewModule>,
) -> Result<Json<TeacherModuleResponse>, StatusCode> {
    let module = state
        .service
        .repo
        .create_module(&module, &user)
        .await
        .map_err(|err| {
            error!(error = ?err, %user, "[http] create_module");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(module.into()))
}

async fn get_module(
    TeacherUser(user): TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
) -> Result<Json<TeacherModuleResponse>, StatusCode> {
    let module = state
        .service
        .repo
        .find_module(&module_id, &user)
        .await
        .map_err(|err| {
            error!(error = ?err, %user, module_id, "[http] get_module");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(module.into()))
}

async fn update_module(
    TeacherUser(user): TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
    Json(module): Json<NewModule>,
) -> Result<Json<TeacherModuleResponse>, StatusCode> {
    let module = state
        .service
        .repo
        .update_module(&module_id, &module, &user)
        .await
        .map_err(|err| {
            error!(error = ?err, %user, module_id, "[http] update_module");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(module.into()))
}

async fn delete_modules(
    TeacherUser(user): TeacherUser,
    State(state): State<AppState>,
    Json(module_ids): Json<Vec<String>>,
) -> Result<(), StatusCode> {
    state
        .service
        .repo
        .delete_modules(&module_ids, &user)
        .await
        .map_err(|err| {
            error!(error = ?err, %user, ?module_ids, "[http] delete_modules");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}

async fn get_grades(
    TeacherUser(user): TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
) -> Result<Json<ModuleGradesResponse>, StatusCode> {
    Ok(Json(
        state
            .service
            .get_module_grades(&module_id, &user)
            .await
            .map_err(|err| {
                error!(error = ?err, %user, module_id, "[http] delete_modules");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
    ))
}

async fn create_assignment(
    TeacherUser(user): TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
    Json(assignment): Json<NewAssignment>,
) -> Result<Json<TeacherAssignmentResponse>, StatusCode> {
    let assignment = state
        .service
        .repo
        .create_assignment(&module_id, &assignment, &user)
        .await
        .map_err(|err| {
            error!(error = ?err, %user, module_id, "[http] create_assignment");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(assignment.into()))
}

async fn get_assignment(
    TeacherUser(user): TeacherUser,
    State(state): State<AppState>,
    Path((module_id, assignment_id)): Path<(String, String)>,
) -> Result<Json<TeacherAssignmentResponse>, StatusCode> {
    let assignment = state
        .service
        .repo
        .find_assignment(&module_id, &assignment_id, &user)
        .await
        .map_err(|err| {
            error!(error = ?err, %user, module_id, assignment_id, "[http] get_assignment");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(assignment.into()))
}

async fn update_assignment(
    TeacherUser(user): TeacherUser,
    State(state): State<AppState>,
    Path((module_id, assignment_id)): Path<(String, String)>,
    Json(assignment): Json<NewAssignment>,
) -> Result<Json<TeacherAssignmentResponse>, StatusCode> {
    let assignment = state
        .service
        .repo
        .update_assignment(&module_id, &assignment_id, &assignment, &user)
        .await
        .map_err(|err| {
            error!(error = ?err, %user, module_id, assignment_id, "[http] update_assignment");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(assignment.into()))
}

async fn delete_assignments(
    TeacherUser(user): TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
    Json(assignment_ids): Json<Vec<String>>,
) -> Result<(), StatusCode> {
    state
        .service
        .repo
        .delete_assignments(&module_id, &assignment_ids, &user)
        .await
        .map_err(|err| {
            error!(error = ?err, %user, module_id, ?assignment_ids, "[http] update_assignment");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}
