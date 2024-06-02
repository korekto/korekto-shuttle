use super::Repository;

impl Repository {
    pub async fn set_user_admin(&self, user_id: i32) -> anyhow::Result<()> {
        const QUERY: &str = "UPDATE \"user\"
        SET admin = true
        WHERE id = $1";

        sqlx::query(QUERY).bind(user_id).execute(&self.pool).await?;
        Ok(())
    }
}
