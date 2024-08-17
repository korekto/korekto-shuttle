use super::Repository;
use anyhow::Context;

impl Repository {
    pub async fn set_user_admin(&self, user_id: i32) -> anyhow::Result<()> {
        const QUERY: &str = "UPDATE \"user\"
        SET admin = true
        WHERE id = $1";

        sqlx::query(QUERY)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .context(format!("[sql] set_user_admin(user_id={user_id:?})"))
    }
}
