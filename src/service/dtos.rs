use crate::entities::{Assignment, EmbeddedAssignmentDesc, Module, ModuleDesc, UserModuleDesc};
use serde::Serialize;
use time::serde::rfc3339 as dto_time_serde;
use time::OffsetDateTime;

pub trait VecInto<D> {
    fn vec_into(self) -> Vec<D>;
}

impl<E, D> VecInto<D> for Vec<E>
where
    D: From<E>,
{
    fn vec_into(self) -> Vec<D> {
        self.into_iter().map(std::convert::Into::into).collect()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UserModuleResponse {
    pub id: String,
    pub name: String,
    #[serde(with = "dto_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "dto_time_serde")]
    pub stop: OffsetDateTime,
    pub linked_repo_count: i32,
    pub assignment_count: i32,
    pub grade: f32,
    //#[serde(with = "time_serde")]
    pub latest_update: Option<OffsetDateTime>,
}

impl From<UserModuleDesc> for UserModuleResponse {
    fn from(value: UserModuleDesc) -> Self {
        Self {
            id: value.uuid,
            name: value.name,
            start: value.start,
            stop: value.stop,
            linked_repo_count: value.linked_repo_count,
            assignment_count: value.assignment_count,
            grade: value.grade,
            latest_update: value.latest_update,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleDescResponse {
    pub id: String,
    pub name: String,
    #[serde(with = "dto_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "dto_time_serde")]
    pub stop: OffsetDateTime,
    pub assignment_count: i64,
}

impl From<ModuleDesc> for ModuleDescResponse {
    fn from(value: ModuleDesc) -> Self {
        Self {
            id: value.uuid,
            name: value.name,
            start: value.start,
            stop: value.stop,
            assignment_count: value.assignment_count,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleResponse {
    pub id: String,
    pub name: String,
    #[serde(with = "dto_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "dto_time_serde")]
    pub stop: OffsetDateTime,
    pub unlock_key: String,
    pub assignments: Vec<AssignmentDescResponse>,
}

impl From<Module> for ModuleResponse {
    fn from(value: Module) -> Self {
        Self {
            id: value.uuid,
            name: value.name,
            start: value.start,
            stop: value.stop,
            unlock_key: value.unlock_key,
            assignments: value.assignments.vec_into(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct AssignmentDescResponse {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub a_type: String,
    #[serde(with = "dto_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "dto_time_serde")]
    pub stop: OffsetDateTime,
    pub factor_percentage: i32,
}

impl From<EmbeddedAssignmentDesc> for AssignmentDescResponse {
    fn from(value: EmbeddedAssignmentDesc) -> Self {
        Self {
            id: value.id,
            name: value.name,
            a_type: value.a_type,
            start: value.start,
            stop: value.stop,
            factor_percentage: value.factor_percentage,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct AssignmentResponse {
    pub id: String,
    pub name: String,
    #[serde(with = "dto_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "dto_time_serde")]
    pub stop: OffsetDateTime,
    pub description: String,
    #[serde(rename = "type")]
    pub a_type: String,
    pub subject_url: String,
    pub grader_url: String,
    pub repository_name: String,
    pub factor_percentage: i32,
    pub grader_run_url: String,
}

impl From<Assignment> for AssignmentResponse {
    fn from(value: Assignment) -> Self {
        Self {
            id: value.uuid,
            name: value.name,
            start: value.start,
            stop: value.stop,
            description: value.description,
            a_type: value.a_type,
            subject_url: value.subject_url,
            grader_url: value.grader_url,
            repository_name: value.repository_name,
            factor_percentage: value.factor_percentage,
            grader_run_url: value.grader_run_url,
        }
    }
}
