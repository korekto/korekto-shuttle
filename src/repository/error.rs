use crate::entities::User;
use crate::repository::Repository;
use anyhow::Context;

impl Repository {
    pub async fn trigger_error(&self, user: &User) -> anyhow::Result<i32> {
        const QUERY: &str = "\
            SELECT count(*)::integer
            FROM not_existing_table
        ";

        sqlx::query_scalar(QUERY)
            .fetch_one(&self.pool)
            .await
            .context(format!("[sql] trigger_error(user={user})"))
    }
}
