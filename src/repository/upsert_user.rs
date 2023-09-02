use anyhow::anyhow;

use crate::entities;

use super::Repository;

impl Repository {
    pub async fn upsert_user(&self, user: &entities::NewUser) -> anyhow::Result<entities::User> {
        const QUERY: &str = "INSERT INTO \"user\"
        (name, provider_login, email, avatar_url, github_user_tokens)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (provider_login) DO UPDATE
        SET (name, email, avatar_url, github_user_tokens)
        = ($1, $3, $4, $5)
        RETURNING *";

        sqlx::query_as::<_, entities::User>(QUERY)
            .bind(&user.name)
            .bind(&user.provider_login)
            .bind(&user.email)
            .bind(&user.avatar_url)
            .bind(&user.github_user_tokens)
            .fetch_one(&self.pool)
            .await
            .map_err(|err| {
                anyhow!(
                    "upsert_user({:?}, {:?}): {:?}",
                    &user.provider_login,
                    &user.email,
                    &err
                )
            })
    }
}
