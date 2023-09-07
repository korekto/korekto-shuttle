use super::Repository;
use tracing::error;

impl Repository {
    pub async fn delete_users_by_id(&self, user_ids: &[i32]) -> anyhow::Result<u64> {
        const QUERY: &str = "DELETE FROM \"user\" WHERE id = ANY($1)";

        match sqlx::query(QUERY).bind(user_ids).execute(&self.pool).await {
            Err(err) => {
                error!("delete_users_by_id({:?}): {:?}", user_ids, &err);
                Err(err.into())
            }
            Ok(query_result) => Ok(query_result.rows_affected()),
        }
    }
}
