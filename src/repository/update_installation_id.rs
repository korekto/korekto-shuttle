use anyhow::anyhow;

use super::Repository;

impl Repository {
    pub async fn update_installation_id(
        &self,
        user_id: &i32,
        installation_id: &str,
    ) -> anyhow::Result<()> {
        const QUERY: &str = "UPDATE \"user\" \
                             SET installation_id = $2 \
                             WHERE id = $1";

        sqlx::query(QUERY)
            .bind(user_id)
            .bind(installation_id)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                anyhow!(
                    "update_installation_id({:?}, {:?}): {:?}",
                    user_id,
                    installation_id,
                    &err
                )
            })?;
        Ok(())
    }
}
