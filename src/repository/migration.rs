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

    pub async fn wipe_database(&self) -> anyhow::Result<()> {
        const QUERY: &str = "
        DO $$ DECLARE
            r RECORD;
        BEGIN
            FOR r IN (SELECT tablename FROM pg_tables WHERE schemaname = current_schema()) LOOP
                EXECUTE 'DROP TABLE IF EXISTS ' || quote_ident(r.tablename) || ' CASCADE';
            END LOOP;
        END $$;
        ";
        sqlx::query(QUERY).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn drop_table(&self, table_name: &str) -> anyhow::Result<()> {
        let query = format!("DROP TABLE IF EXISTS {table_name} CASCADE");

        sqlx::query(&query).execute(&self.pool).await?;
        Ok(())
    }
}
