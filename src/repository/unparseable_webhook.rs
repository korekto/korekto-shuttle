use crate::entities::UnparseableWebhook;
use crate::repository::Repository;
use anyhow::Context;

impl Repository {
    pub async fn insert_unparseable_webhook(
        &self,
        origin: &str,
        event: &str,
        payload: &str,
        error: &str,
    ) -> anyhow::Result<()> {
        const QUERY: &str = "INSERT INTO unparseable_webhook
        (origin, event, payload, error)
        VALUES ($1, $2, $3, $4)";

        sqlx::query(QUERY)
            .bind(origin)
            .bind(event)
            .bind(payload)
            .bind(error)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .context(format!("[sql] insert_unparseable_webhook(origin={event:?}, origin={event:?}, payload={payload:?})"))
    }

    pub async fn get_unparseable_webhooks(
        &self,
        page: i32,
        per_page: i32,
    ) -> anyhow::Result<Vec<UnparseableWebhook>> {
        const QUERY: &str = "\
            SELECT *, (count(*) OVER ())::integer as total_count
            FROM unparseable_webhook
            ORDER BY created_at DESC
            LIMIT $1
            OFFSET $2
        ";

        let offset = if page == 1 { 0 } else { (page - 1) * per_page };

        sqlx::query_as::<_, UnparseableWebhook>(QUERY)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
            .context(format!(
                "[sql] get_unparseable_webhooks(page={page:?}, per_page={per_page:?})"
            ))
    }

    pub async fn delete_unparseable_webhooks(&self) -> anyhow::Result<()> {
        const QUERY: &str = "TRUNCATE unparseable_webhook";

        sqlx::query(QUERY)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .context("[sql] delete_unparseable_webhooks()")
    }
}
