use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use time::{OffsetDateTime, PrimitiveDateTime};

use time::serde::rfc3339 as time_serde;

#[derive(Debug)]
pub struct NewUser {
    pub name: String,
    pub provider_login: String,
    // This is the discriminant for upsert
    pub email: String,
    pub avatar_url: String,
    pub github_user_tokens: Option<Json<GitHubUserTokens>>,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub provider_login: String,
    pub email: String,
    pub avatar_url: String,
    pub installation_id: Option<String>,
    pub github_user_tokens: Option<Json<GitHubUserTokens>>,
    pub created_at: PrimitiveDateTime,
    pub admin: bool,
    pub teacher: bool,
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
pub struct NewModule {
    pub name: String,
    #[serde(with = "time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "time_serde")]
    pub stop: OffsetDateTime,
    pub unlock_key: String,
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize)]
pub struct ModuleDesc {
    pub id: String,
    pub name: String,
    #[serde(with = "time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "time_serde")]
    pub stop: OffsetDateTime,
    pub assignment_count: i64,
}

#[derive(Serialize, Debug, Clone)]
pub struct Module {
    pub id: String,
    pub name: String,
    #[serde(with = "time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "time_serde")]
    pub stop: OffsetDateTime,
    pub unlock_key: String,
    pub assignments: Vec<EmbeddedAssignmentDesc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmbeddedAssignmentDesc {
    pub id: String,
    pub name: String,
    pub a_type: String,
    #[serde(with = "time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "time_serde")]
    pub stop: OffsetDateTime,
    pub factor_percentage: i32,
}
