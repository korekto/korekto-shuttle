use super::Repository;
use anyhow::Context;

impl Repository {
    pub async fn delete_users_by_id(&self, user_ids: &[i32]) -> anyhow::Result<u64> {
        const QUERY: &str = "DELETE FROM \"user\" WHERE id = ANY($1)";

        sqlx::query(QUERY)
            .bind(user_ids)
            .execute(&self.pool)
            .await
            .map(|q| q.rows_affected())
            .context(format!("[sql] delete_users_by_id(user_ids={user_ids:?})"))
    }
}
