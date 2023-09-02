use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use time::PrimitiveDateTime;

#[derive(Debug)]
pub struct NewUser {
    pub name: String,
    pub provider_login: String, // This is the discriminant for upsert
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
