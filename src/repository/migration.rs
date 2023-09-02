use super::Repository;

impl Repository {
    pub async fn reset_migrations(&self) -> anyhow::Result<()> {
        const QUERY: &str = "DROP TABLE IF EXISTS \"_sqlx_migrations\"";

        sqlx::query(QUERY).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }
}
