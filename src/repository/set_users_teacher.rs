use super::Repository;
use anyhow::Context;

impl Repository {
    pub async fn set_users_teacher(&self, user_ids: &[i32]) -> anyhow::Result<u64> {
        const QUERY: &str = "UPDATE \"user\"
        SET teacher = true
        WHERE id = ANY($1)";

        sqlx::query(QUERY)
            .bind(user_ids)
            .execute(&self.pool)
            .await
            .map(|q| q.rows_affected())
            .context(format!("[sql] set_users_teacher(user_ids={user_ids:?})"))
    }
}
