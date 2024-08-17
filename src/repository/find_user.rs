use crate::entities;
use anyhow::{anyhow, Context};
use sqlx::{Encode, Postgres, Type};
use std::fmt::Display;

use super::Repository;

impl Repository {
    pub async fn find_user_by_id(&self, user_id: &i32) -> anyhow::Result<entities::User> {
        const QUERY: &str = "SELECT *, uuid::varchar as uuid FROM \"user\" WHERE id = $1";
        self.find_user_by(QUERY, user_id, "id").await
    }

    pub async fn find_user_by_provider_login(&self, login: &str) -> anyhow::Result<entities::User> {
        const QUERY: &str =
            "SELECT *, uuid::varchar as uuid FROM \"user\" WHERE provider_login = $1";
        self.find_user_by(QUERY, login, "provider_login").await
    }

    async fn find_user_by<
        'q,
        T: 'q + Send + Encode<'q, Postgres> + Type<Postgres> + Display + Copy,
    >(
        &self,
        query: &'q str,
        key: T,
        field: &str,
    ) -> anyhow::Result<entities::User> {
        match sqlx::query_as::<_, entities::User>(query)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
        {
            Err(err) => Err(err).context(format!("[sql] find_user_by_{field}(key={key})")),
            Ok(None) => Err(anyhow!("User not found")),
            Ok(Some(res)) => Ok(res),
        }
    }
}
