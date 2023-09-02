use crate::entities;
use anyhow::anyhow;

use super::Repository;

impl Repository {
    pub async fn find_user_by_id(&self, user_id: &i32) -> anyhow::Result<entities::User> {
        const QUERY: &str = "SELECT * FROM \"user\" WHERE id = $1";

        match sqlx::query_as::<_, entities::User>(QUERY)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
        {
            Err(err) => Err(anyhow!("find_user_by_id({}): {:?}", user_id, &err)),
            Ok(None) => Err(anyhow!("User not found")),
            Ok(Some(res)) => Ok(res),
        }
    }
}
