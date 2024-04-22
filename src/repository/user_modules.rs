use crate::entities::{ModuleDesc, User};
use crate::repository::Repository;
use crate::service::ObfuscatedStr;

impl Repository {
    pub async fn create_user_module(&self, user: &User, module_id: i32) -> anyhow::Result<()> {
        const QUERY: &str = "INSERT INTO user_module
        (user_id, module_id)
        VALUES ($1, $2)";

        sqlx::query(QUERY)
            .bind(user.id)
            .bind(module_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn find_module_by_key(
        &self,
        key: &ObfuscatedStr,
    ) -> anyhow::Result<Option<ModuleDesc>> {
        const QUERY: &str = "SELECT
            m.id,
            m.uuid::varchar as uuid,
            m.name,
            m.start,
            m.stop,
            count(a.id) as assignment_count
            FROM \"module\" m
            LEFT JOIN assignment a ON a.module_id = m.id
            WHERE m.unlock_key = $1";

        Ok(sqlx::query_as::<_, ModuleDesc>(QUERY)
            .bind(&key.0)
            .fetch_optional(&self.pool)
            .await?)
    }
}
