use crate::entities;
use anyhow::anyhow;

use super::Repository;

impl Repository {
    pub async fn find_users(&self) -> anyhow::Result<Vec<entities::User>> {
        const QUERY: &str = "SELECT *, uuid::varchar as uuid FROM \"user\"";

        sqlx::query_as::<_, entities::User>(QUERY)
            .fetch_all(&self.pool)
            .await
            .map_err(|err| anyhow!("find_users: {:?}", &err))
    }
}
