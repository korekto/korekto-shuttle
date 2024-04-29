use crate::entities;
use crate::entities::{
    Assignment, EmbeddedAssignmentDesc, Module, ModuleDesc, UnparseableWebhook, UserModuleDesc,
};
use serde::Serialize;
use time::format_description::well_known::Iso8601;
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

#[derive(serde::Deserialize, validator::Validate, Debug)]
pub struct PaginationQuery {
    #[serde(default)]
    #[validate(range(min = 1))]
    pub page: i32,
    #[serde(default)]
    #[validate(range(min = 10, max = 30))]
    pub per_page: i32,
}

impl PaginationQuery {
    #[must_use]
    pub const fn new(page: i32, per_page: i32) -> Self {
        Self { page, per_page }
    }
}

#[derive(serde::Serialize, Debug)]
pub struct Page<T>
where
    T: serde::Serialize + std::fmt::Debug,
{
    pub page: i32,
    pub per_page: i32,
    pub total_page: i32,
    pub total_count: i32,
    pub data: Vec<T>,
}

impl<T> Page<T>
where
    T: serde::Serialize + std::fmt::Debug,
{
    #[must_use]
    pub const fn empty(per_page: i32) -> Self {
        Self {
            page: 0,
            per_page,
            total_page: 0,
            total_count: 0,
            data: vec![],
        }
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

#[derive(serde::Serialize, Clone)]
pub struct UserForAdminResponse {
    pub id: i32,
    pub provider_login: String,
    pub firstname: String,
    pub lastname: String,
    pub school_group: String,
    pub school_email: String,
    pub provider_email: String,
    pub accessible_repos: u8,
    pub teacher: bool,
    pub admin: bool,
    pub installation_id: String,
    pub created_at: String,
}

impl TryFrom<entities::User> for UserForAdminResponse {
    type Error = anyhow::Error;

    fn try_from(user: entities::User) -> Result<Self, Self::Error> {
        Ok(Self {
            id: user.id,
            provider_login: user.provider_login,
            firstname: user.first_name,
            lastname: user.last_name,
            school_group: user.school_group,
            school_email: user.school_email,
            provider_email: user.provider_email,
            accessible_repos: 0,
            teacher: user.teacher,
            admin: user.admin,
            installation_id: user.installation_id.unwrap_or_default(),
            created_at: user.created_at.format(&Iso8601::DEFAULT)?,
        })
    }
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct UnparseableWebhookResponse {
    pub created_at: OffsetDateTime,
    pub origin: String,
    pub event: String,
    pub payload: String,
    pub error: String,
}

impl From<UnparseableWebhook> for UnparseableWebhookResponse {
    fn from(value: UnparseableWebhook) -> Self {
        Self {
            created_at: value.created_at,
            origin: value.origin,
            event: value.event,
            payload: value.payload,
            error: value.error,
        }
    }
}
