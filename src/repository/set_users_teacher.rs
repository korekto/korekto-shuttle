use super::Repository;
use tracing::error;

impl Repository {
    pub async fn set_users_teacher(&self, user_ids: &[i32]) -> anyhow::Result<u64> {
        const QUERY: &str = "UPDATE \"user\"
        SET teacher = true
        WHERE id = ANY($1)";

        match sqlx::query(QUERY).bind(user_ids).execute(&self.pool).await {
            Err(err) => {
                error!("set_users_teacher({:?}): {:?}", user_ids, &err);
                Err(err.into())
            }
            Ok(query_result) => Ok(query_result.rows_affected()),
        }
    }
}
