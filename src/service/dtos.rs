use crate::entities;
use crate::entities::{
    Assignment, AssignmentGrade, Details, EmbeddedAssignmentDesc, GradingTask, InstantGrade,
    Module, ModuleDesc, StudentGrades, UnparseableWebhook, UserAssignment, UserAssignmentDesc,
    UserModule, UserModuleDesc,
};
use crate::repository::grading_task::GradingStatus;
use crate::service::webhook_models::RunnerGradePart;
use rust_decimal::Decimal;
use serde::Serialize;
use std::str::FromStr;
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
#[cfg_attr(
    feature = "automatic_test_feature",
    derive(derive_builder::Builder, PartialEq, Eq),
    builder(setter(into, strip_option))
)]
pub struct UserModuleDescResponse {
    pub id: String,
    pub name: String,
    #[serde(with = "dto_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "dto_time_serde")]
    pub stop: OffsetDateTime,
    pub linked_repo_count: i32,
    pub assignment_count: i32,
    pub grade: Decimal,
    #[serde(with = "dto_time_serde::option")]
    pub latest_update: Option<OffsetDateTime>,
}

impl From<UserModuleDesc> for UserModuleDescResponse {
    fn from(value: UserModuleDesc) -> Self {
        Self {
            id: value.uuid,
            name: value.name,
            start: value.start,
            stop: value.stop,
            linked_repo_count: value.linked_repo_count,
            assignment_count: value.assignment_count,
            grade: to_decimal(value.grade),
            latest_update: value.latest_update,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TeacherModuleDescResponse {
    pub id: String,
    pub name: String,
    #[serde(with = "dto_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "dto_time_serde")]
    pub stop: OffsetDateTime,
    pub assignment_count: i64,
}

impl From<ModuleDesc> for TeacherModuleDescResponse {
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
pub struct TeacherModuleResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(with = "dto_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "dto_time_serde")]
    pub stop: OffsetDateTime,
    pub unlock_key: String,
    pub source_url: String,
    pub assignments: Vec<TeacherAssignmentDescResponse>,
}

impl From<Module> for TeacherModuleResponse {
    fn from(value: Module) -> Self {
        Self {
            id: value.uuid,
            name: value.name,
            description: value.description,
            start: value.start,
            stop: value.stop,
            unlock_key: value.unlock_key,
            source_url: value.source_url,
            assignments: value.assignments.0.vec_into(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct TeacherAssignmentDescResponse {
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

impl From<EmbeddedAssignmentDesc> for TeacherAssignmentDescResponse {
    fn from(value: EmbeddedAssignmentDesc) -> Self {
        Self {
            id: value.uuid,
            name: value.name,
            a_type: value.a_type,
            start: value.start,
            stop: value.stop,
            factor_percentage: value.factor_percentage,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct TeacherAssignmentResponse {
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

impl From<Assignment> for TeacherAssignmentResponse {
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
    #[serde(with = "dto_time_serde")]
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

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "automatic_test_feature",
    derive(derive_builder::Builder),
    builder(setter(into, strip_option))
)]
pub struct UserModuleResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(with = "dto_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "dto_time_serde")]
    pub stop: OffsetDateTime,
    pub latest_update: Option<OffsetDateTime>,
    pub source_url: String,
    pub locked: bool,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub lock_reason: Option<String>,
    pub assignments: Vec<UserAssignmentDescResponse>,
}

impl From<UserModule> for UserModuleResponse {
    fn from(value: UserModule) -> Self {
        Self {
            id: value.uuid,
            name: value.name,
            description: value.description,
            start: value.start,
            stop: value.stop,
            latest_update: value.latest_update,
            source_url: value.source_url,
            locked: false,
            lock_reason: None,
            assignments: value.assignments.0.vec_into(),
        }
    }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "automatic_test_feature",
    derive(derive_builder::Builder),
    builder(setter(into, strip_option))
)]
pub struct UserAssignmentDescResponse {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(with = "dto_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "dto_time_serde")]
    pub stop: OffsetDateTime,
    #[serde(rename = "type")]
    pub a_type: String,
    pub factor_percentage: i32,
    pub locked: bool,
    pub grade: Decimal,
    pub repo_linked: bool,
    pub repository_name: String,
}

impl From<UserAssignmentDesc> for UserAssignmentDescResponse {
    fn from(value: UserAssignmentDesc) -> Self {
        Self {
            id: value.uuid,
            name: value.name,
            description: value.description,
            start: value.start,
            stop: value.stop,
            a_type: value.a_type,
            factor_percentage: value.factor_percentage,
            locked: false,
            grade: to_decimal(value.grade),
            repo_linked: value.repo_linked,
            repository_name: value.repository_name,
        }
    }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "automatic_test_feature",
    derive(derive_builder::Builder),
    builder(setter(into, strip_option))
)]
pub struct UserAssignmentResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub a_type: String,
    pub name: String,
    pub description: String,
    #[serde(with = "dto_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "dto_time_serde")]
    pub stop: OffsetDateTime,
    pub repo_linked: bool,
    pub repository_name: String,
    pub subject_url: String,
    pub grader_url: String,
    pub repository_url: String,
    pub factor_percentage: i32,
    pub normalized_grade: f32,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub status: Option<GradingStatus>,
    pub queue_due_to: i32,
    pub locked: bool,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub lock_reason: Option<String>,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub latest_run: Option<CompleteRunInfoResponse>,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub ongoing_run: Option<RunInfo>,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub error: Option<String>,
}

fn compute_status(value: &UserAssignment) -> Option<GradingStatus> {
    let mut tasks: Vec<GradingStatus> = value
        .grading_tasks
        .0
        .clone()
        .iter()
        .filter_map(|gt| GradingStatus::from_str(&gt.status).ok())
        .collect();
    tasks.sort();

    let status = tasks.pop();
    status.or_else(|| {
        if value.previous_grading_error.is_some() {
            Some(GradingStatus::ERROR)
        } else {
            None
        }
    })
}

fn compute_ongoing_run(value: &UserAssignment) -> Option<RunInfo> {
    value.clone().running_grading_metadata.map(|m| RunInfo {
        short_commit_id: m.0.short_commit_id,
        commit_url: m.0.commit_url,
        grading_log_url: m.0.full_log_url,
    })
}

impl TryFrom<UserAssignment> for UserAssignmentResponse {
    type Error = anyhow::Error;

    fn try_from(value: UserAssignment) -> Result<Self, Self::Error> {
        fn capitalize(s: &str) -> String {
            let lowered = s.to_lowercase();
            let mut c = lowered.chars();
            c.next().map_or_else(String::new, |f| {
                f.to_uppercase().collect::<String>() + c.as_str()
            })
        }
        fn gh_url_encode_description(a_type: &str, desc: &str) -> String {
            format!("🎓 {}: {}", capitalize(a_type), desc).replace(' ', "+")
        }

        let status = compute_status(&value);
        let ongoing_run = compute_ongoing_run(&value);
        let repository_url = if value.repo_linked {
            format!(
                "https://github.com/{}/{}",
                &value.user_provider_login, &value.repository_name
            )
        } else {
            format!(
                "https://github.com/new?name={}&owner={}&visibility=private&description={}",
                &value.repository_name,
                &value.user_provider_login,
                gh_url_encode_description(&value.a_type, &value.description)
            )
        };
        Ok(Self {
            id: value.uuid,
            name: value.name,
            description: value.description,
            start: value.start,
            stop: value.stop,
            a_type: value.a_type,
            factor_percentage: value.factor_percentage,
            normalized_grade: value.normalized_grade,
            status,
            queue_due_to: value.queue_due_to,
            repo_linked: value.repo_linked,
            repository_url,
            repository_name: value.repository_name,
            subject_url: value.subject_url,
            grader_url: value.grader_url,
            latest_run: value.grades_history.last().map(|g| g.clone().into()),
            locked: false,
            lock_reason: None,
            ongoing_run,
            error: value.previous_grading_error,
        })
    }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct RunInfo {
    pub short_commit_id: String,
    pub commit_url: String,
    pub grading_log_url: String,
}

#[derive(serde::Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "automatic_test_feature",
    derive(derive_builder::Builder),
    builder(setter(into, strip_option))
)]
pub struct CompleteRunInfoResponse {
    pub short_commit_id: String,
    pub commit_url: String,
    pub grading_log_url: String,
    #[serde(with = "dto_time_serde")]
    pub time: OffsetDateTime,
    pub details: Vec<DetailsResponse>,
}

impl From<InstantGrade> for CompleteRunInfoResponse {
    fn from(value: InstantGrade) -> Self {
        Self {
            short_commit_id: value.short_commit_id,
            commit_url: value.commit_url,
            grading_log_url: value.grading_log_url,
            time: value.time,
            details: value.details.vec_into(),
        }
    }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "automatic_test_feature",
    derive(derive_builder::Builder),
    builder(setter(into, strip_option))
)]
pub struct DetailsResponse {
    pub name: String,
    pub grade: f32,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub max_grade: Option<f32>,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub messages: Vec<String>,
}

impl From<Details> for DetailsResponse {
    fn from(value: Details) -> Self {
        Self {
            name: value.name,
            grade: value.grade,
            max_grade: value.max_grade,
            messages: value.messages,
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct NewGradeRequest {
    pub time: Option<OffsetDateTime>,
    pub short_commit_id: String,
    pub commit_url: String,
    pub grading_log_url: String,
    pub details: Vec<NewGradeDetailRequest>,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct NewGradeDetailRequest {
    pub name: String,
    pub grade: f32,
    pub max_grade: Option<f32>,
    pub messages: Vec<String>,
}

impl From<RunnerGradePart> for NewGradeDetailRequest {
    fn from(value: RunnerGradePart) -> Self {
        Self {
            name: value.id,
            grade: value.grade,
            max_grade: value.max_grade,
            messages: value.comments,
        }
    }
}

impl From<NewGradeDetailRequest> for Details {
    fn from(value: NewGradeDetailRequest) -> Self {
        Self {
            name: value.name,
            grade: value.grade,
            max_grade: value.max_grade,
            messages: value.messages,
        }
    }
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct GradingTaskResponse {
    module_id: String,
    assignment_id: String,
    provider_login: String,
    status: String,
    #[serde(with = "dto_time_serde")]
    created_at: OffsetDateTime,
    #[serde(with = "dto_time_serde")]
    updated_at: OffsetDateTime,
    repository_name: String,
}

impl From<GradingTask> for GradingTaskResponse {
    fn from(value: GradingTask) -> Self {
        Self {
            module_id: value.module_uuid,
            assignment_id: value.assignment_uuid,
            provider_login: value.provider_login,
            status: value.status,
            created_at: value.created_at,
            updated_at: value.updated_at,
            repository_name: value.repository_name,
        }
    }
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct ModuleGradesResponse {
    pub assignments: Vec<GradeAssignmentResponse>,
    pub students: Vec<StudentGradesResponse>,
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct GradeAssignmentResponse {
    short_name: String,
    name: String,
    description: String,
    a_type: String,
    factor_percentage: i32,
}

impl From<(usize, &AssignmentGrade)> for GradeAssignmentResponse {
    fn from(value: (usize, &AssignmentGrade)) -> Self {
        let short_name = if value.1.a_type == "EXERCISE" {
            format!("Ex {}", value.0 + 1)
        } else {
            "Project".to_string()
        };
        Self {
            short_name,
            name: value.1.name.clone(),
            description: value.1.description.clone(),
            a_type: value.1.a_type.clone(),
            factor_percentage: value.1.factor_percentage,
        }
    }
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct StudentGradesResponse {
    first_name: String,
    last_name: String,
    school_email: String,
    grades: Vec<Decimal>,
    total: Decimal,
}

impl From<StudentGrades> for StudentGradesResponse {
    fn from(value: StudentGrades) -> Self {
        Self {
            first_name: value.first_name,
            last_name: value.last_name,
            school_email: value.school_email,
            grades: value.grades.0.iter().map(|g| to_decimal(g.grade)).collect(),
            total: to_decimal(value.total),
        }
    }
}

fn to_decimal(value: f32) -> Decimal {
    Decimal::from_f32_retain(value)
        .unwrap_or_default()
        .round_dp(2)
}
