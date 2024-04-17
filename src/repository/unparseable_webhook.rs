use crate::repository::Repository;
use anyhow::anyhow;

impl Repository {
    pub async fn insert_unparseable_webhook(
        &self,
        origin: &str,
        event: &str,
        payload: &str,
        error: &str,
    ) -> anyhow::Result<()> {
        const QUERY: &str = "INSERT INTO \"unparseable_webhook\"
        (origin, event, payload, error)
        VALUES ($1, $2, $3, $4)";

        sqlx::query(QUERY)
            .bind(origin)
            .bind(event)
            .bind(payload)
            .bind(error)
            .execute(&self.pool)
            .await
            .map_err(|err| {
                anyhow!(
                    "insert_unparseable_webhook({:?}, {:?}): {:?}",
                    origin,
                    event,
                    &err
                )
            })?;
        Ok(())
    }
}
