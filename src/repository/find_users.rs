use crate::entities;
use anyhow::Context;

use super::Repository;

impl Repository {
    pub async fn find_users(&self) -> anyhow::Result<Vec<entities::User>> {
        const QUERY: &str = "SELECT *, uuid::varchar as uuid FROM \"user\"";

        sqlx::query_as::<_, entities::User>(QUERY)
            .fetch_all(&self.pool)
            .await
            .context("[sql] find_users)")
    }
}
