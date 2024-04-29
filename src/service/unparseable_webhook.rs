use crate::service::dtos::{Page, PaginationQuery, UnparseableWebhookResponse, VecInto};
use crate::service::Service;
use anyhow::anyhow;

impl Service {
    pub async fn get_unparseable_webhooks(
        &self,
        pagination: &PaginationQuery,
    ) -> anyhow::Result<Page<UnparseableWebhookResponse>> {
        let rows = self
            .repo
            .get_unparseable_webhooks(pagination.page, pagination.per_page)
            .await?;

        if rows.is_empty() {
            Ok(Page::empty(pagination.per_page))
        } else {
            let first = rows.first().ok_or_else(|| anyhow!("Could not happen"))?;
            Ok(Page {
                page: pagination.page,
                per_page: pagination.per_page,
                #[allow(clippy::cast_possible_truncation)]
                #[allow(clippy::cast_precision_loss)]
                total_page: (first.total_count as f32 / pagination.per_page as f32).ceil() as i32,
                total_count: first.total_count,
                data: rows.vec_into(),
            })
        }
    }
}
