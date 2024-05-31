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
    user: TeacherUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<TeacherModuleDescResponse>>, StatusCode> {
    let modules = state
        .service
        .repo
        .find_modules(&user.0)
        .await
        .map_err(|err| {
            error!("get_modules {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(modules.vec_into()))
}

async fn create_module(
    user: TeacherUser,
    State(state): State<AppState>,
    Json(module): Json<NewModule>,
) -> Result<Json<TeacherModuleResponse>, StatusCode> {
    let module = state
        .service
        .repo
        .create_module(&module, &user.0)
        .await
        .map_err(|err| {
            error!("create_module {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(module.into()))
}

async fn get_module(
    user: TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
) -> Result<Json<TeacherModuleResponse>, StatusCode> {
    let module = state
        .service
        .repo
        .find_module(&module_id, &user.0)
        .await
        .map_err(|err| {
            error!("get_module {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(module.into()))
}

async fn update_module(
    user: TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
    Json(module): Json<NewModule>,
) -> Result<Json<TeacherModuleResponse>, StatusCode> {
    let module = state
        .service
        .repo
        .update_module(&module_id, &module, &user.0)
        .await
        .map_err(|err| {
            error!("update_module {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(module.into()))
}

async fn delete_modules(
    user: TeacherUser,
    State(state): State<AppState>,
    Json(module_ids): Json<Vec<String>>,
) -> Result<(), StatusCode> {
    state
        .service
        .repo
        .delete_modules(&module_ids, &user.0)
        .await
        .map_err(|err| {
            error!("delete_modules {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}

async fn get_grades(
    user: TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
) -> Result<Json<ModuleGradesResponse>, StatusCode> {
    Ok(Json(
        state
            .service
            .get_grades(&module_id, &user.0)
            .await
            .map_err(|err| {
                error!("get_grades {err:#?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
    ))
}

async fn create_assignment(
    user: TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
    Json(assignment): Json<NewAssignment>,
) -> Result<Json<TeacherAssignmentResponse>, StatusCode> {
    let assignment = state
        .service
        .repo
        .create_assignment(&module_id, &assignment, &user.0)
        .await
        .map_err(|err| {
            error!("create_assignment {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(assignment.into()))
}

async fn get_assignment(
    user: TeacherUser,
    State(state): State<AppState>,
    Path((module_id, assignment_id)): Path<(String, String)>,
) -> Result<Json<TeacherAssignmentResponse>, StatusCode> {
    let assignment = state
        .service
        .repo
        .find_assignment(&module_id, &assignment_id, &user.0)
        .await
        .map_err(|err| {
            error!("get_assignment {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(assignment.into()))
}

async fn update_assignment(
    user: TeacherUser,
    State(state): State<AppState>,
    Path((module_id, assignment_id)): Path<(String, String)>,
    Json(assignment): Json<NewAssignment>,
) -> Result<Json<TeacherAssignmentResponse>, StatusCode> {
    let assignment = state
        .service
        .repo
        .update_assignment(&module_id, &assignment_id, &assignment, &user.0)
        .await
        .map_err(|err| {
            error!("update_assignment {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(assignment.into()))
}

async fn delete_assignments(
    user: TeacherUser,
    State(state): State<AppState>,
    Path(module_id): Path<String>,
    Json(assignment_ids): Json<Vec<String>>,
) -> Result<(), StatusCode> {
    state
        .service
        .repo
        .delete_assignments(&module_id, &assignment_ids, &user.0)
        .await
        .map_err(|err| {
            error!("delete_assignments {err:#?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(())
}
