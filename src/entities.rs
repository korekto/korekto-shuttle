use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use std::fmt;
use time::{OffsetDateTime, PrimitiveDateTime};

use time::serde::rfc3339 as entity_time_serde;

#[derive(Debug)]
#[cfg_attr(
    feature = "automatic_test_feature",
    derive(derive_builder::Builder),
    builder(setter(into, strip_option))
)]
pub struct NewUser {
    pub provider_name: String,
    // This is the discriminant for upsert
    pub provider_login: String,
    pub provider_email: String,
    pub avatar_url: String,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub github_user_tokens: Option<Json<GitHubUserTokens>>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub uuid: String,
    pub provider_name: String,
    pub provider_login: String,
    pub provider_email: String,
    pub avatar_url: String,
    pub installation_id: Option<String>,
    pub github_user_tokens: Option<Json<GitHubUserTokens>>,
    pub created_at: OffsetDateTime,
    pub admin: bool,
    pub teacher: bool,
    pub first_name: String,
    pub last_name: String,
    pub school_group: String,
    pub school_email: String,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "provider_login={}, provider_name={}, provider_email={}",
            self.provider_login, self.provider_name, self.provider_email
        )
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct UserProfileUpdate {
    pub firstname: String,
    pub lastname: String,
    pub school_group: String,
    pub school_email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitHubUserTokens {
    pub access_token: Token,
    pub refresh_token: Token,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Token {
    pub value: String,
    pub expiration_date: PrimitiveDateTime,
}

#[derive(sqlx::FromRow, Serialize, Debug, Clone)]
pub struct Table {
    pub name: String,
    pub row_count: i32,
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(
    feature = "automatic_test_feature",
    derive(derive_builder::Builder),
    builder(setter(into, strip_option))
)]
pub struct NewModule {
    pub name: String,
    pub description: String,
    #[serde(with = "entity_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "entity_time_serde")]
    pub stop: OffsetDateTime,
    pub unlock_key: String,
    pub source_url: String,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct ModuleDesc {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub start: OffsetDateTime,
    pub stop: OffsetDateTime,
    pub assignment_count: i64,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub start: OffsetDateTime,
    pub stop: OffsetDateTime,
    pub unlock_key: String,
    pub source_url: String,
    pub assignments: Vec<EmbeddedAssignmentDesc>,
}

pub struct ModuleId {
    pub uuid: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EmbeddedAssignmentDesc {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    #[serde(rename = "type")]
    pub a_type: String,
    #[serde(with = "entity_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "entity_time_serde")]
    pub stop: OffsetDateTime,
    pub factor_percentage: i32,
}

#[derive(Deserialize, Debug, Clone)]
#[cfg_attr(
    feature = "automatic_test_feature",
    derive(derive_builder::Builder),
    builder(setter(into, strip_option))
)]
pub struct NewAssignment {
    pub name: String,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub description: String,
    #[serde(with = "entity_time_serde")]
    #[cfg_attr(
        feature = "automatic_test_feature",
        builder(default = "OffsetDateTime::UNIX_EPOCH")
    )]
    pub start: OffsetDateTime,
    #[serde(with = "entity_time_serde")]
    #[cfg_attr(
        feature = "automatic_test_feature",
        builder(default = "OffsetDateTime::UNIX_EPOCH")
    )]
    pub stop: OffsetDateTime,
    #[serde(rename = "type")]
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub a_type: String,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub subject_url: String,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub grader_url: String,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub repository_name: String,
    pub factor_percentage: i32,
    #[cfg_attr(feature = "automatic_test_feature", builder(default))]
    pub grader_run_url: String,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Assignment {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub start: OffsetDateTime,
    pub stop: OffsetDateTime,
    pub description: String,
    pub a_type: String,
    pub subject_url: String,
    pub grader_url: String,
    pub repository_name: String,
    pub factor_percentage: i32,
    pub grader_run_url: String,
}

pub enum NewGradingTask {
    Internal {
        user_assignment_id: i32,
        user_provider_name: String,
        repository: String,
        grader_repository: String,
    },
    External {
        assignment_uuid: String,
        user_uuid: String,
    },
}

#[derive(sqlx::FromRow, Debug, Clone, PartialEq)]
pub struct UserModuleDesc {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub start: OffsetDateTime,
    pub stop: OffsetDateTime,
    pub linked_repo_count: i32,
    pub assignment_count: i32,
    pub grade: f32,
    pub latest_update: Option<OffsetDateTime>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct UnparseableWebhook {
    pub created_at: OffsetDateTime,
    pub origin: String,
    pub event: String,
    pub payload: String,
    pub error: String,
    pub total_count: i32,
}

impl crate::service::trackable::WithTotalCount for UnparseableWebhook {
    fn total_count(&self) -> i32 {
        self.total_count
    }
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct UserModule {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub start: OffsetDateTime,
    pub stop: OffsetDateTime,
    pub latest_update: Option<OffsetDateTime>,
    pub source_url: String,
    pub assignments: Json<Vec<UserAssignmentDesc>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UserAssignmentDesc {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub description: String,
    #[serde(with = "entity_time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "entity_time_serde")]
    pub stop: OffsetDateTime,
    pub a_type: String,
    pub subject_url: String,
    pub grader_url: String,
    pub repository_name: String,
    pub factor_percentage: i32,
    pub grade: f32,
    pub repo_linked: bool,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct UserAssignment {
    pub id: i32,
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub start: OffsetDateTime,
    pub stop: OffsetDateTime,
    pub a_type: String,
    pub factor_percentage: i32,
    pub subject_url: String,
    pub grader_url: String,
    pub repository_name: String,
    pub repo_linked: bool,
    pub user_provider_login: String,
    pub normalized_grade: f32,
    pub grades_history: Json<Vec<InstantGrade>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InstantGrade {
    pub grade: f32,
    pub max_grade: f32,
    #[serde(with = "entity_time_serde")]
    pub time: OffsetDateTime,
    pub short_commit_id: String,
    pub commit_url: String,
    pub grading_log_url: String,
    pub details: Vec<Details>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Details {
    pub name: String,
    pub grade: f32,
    pub max_grade: Option<f32>,
    pub messages: Vec<String>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct GradingTask {
    pub module_uuid: String,
    pub assignment_uuid: String,
    pub provider_login: String,
    pub status: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub repository_name: String,
    total_count: i32,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct GitHubGradingTask {
    pub id: i32,
    pub uuid: String,
    pub user_assignment_uuid: String,
    pub provider_login: String,
    pub status: String,
    pub version: i32,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub repository_name: String,
    pub installation_id: String,
    pub grader_url: String,
}

impl crate::service::trackable::WithTotalCount for GradingTask {
    fn total_count(&self) -> i32 {
        self.total_count
    }
}
