use crate::service::dtos::{
    GradingTaskResponse, Page, PaginationQuery, UnparseableWebhookResponse, VecInto,
};
use crate::service::Service;
use anyhow::anyhow;
use std::future::Future;

impl Service {
    pub async fn get_unparseable_webhooks(
        &self,
        pagination: &PaginationQuery,
    ) -> anyhow::Result<Page<UnparseableWebhookResponse>> {
        self.get_trackable(pagination, |i1, i2| {
            self.repo.get_unparseable_webhooks(i1, i2)
        })
        .await
    }

    pub async fn get_grading_tasks(
        &self,
        pagination: &PaginationQuery,
    ) -> anyhow::Result<Page<GradingTaskResponse>> {
        self.get_trackable(pagination, |i1, i2| self.repo.get_grading_tasks(i1, i2))
            .await
    }

    async fn get_trackable<F, Fut, E, D>(
        &self,
        pagination: &PaginationQuery,
        f: F,
    ) -> anyhow::Result<Page<D>>
    where
        F: FnOnce(i32, i32) -> Fut + Send,
        Fut: Future<Output = anyhow::Result<Vec<E>>> + Send,
        D: From<E> + serde::Serialize + std::fmt::Debug,
        E: WithTotalCount,
    {
        let rows = f(pagination.page, pagination.per_page).await?;

        if rows.is_empty() {
            Ok(Page::empty(pagination.per_page))
        } else {
            let first = rows.first().ok_or_else(|| anyhow!("Could not happen"))?;
            Ok(Page {
                page: pagination.page,
                per_page: pagination.per_page,
                #[allow(clippy::cast_possible_truncation)]
                #[allow(clippy::cast_precision_loss)]
                total_page: (first.total_count() as f32 / pagination.per_page as f32).ceil() as i32,
                total_count: first.total_count(),
                data: rows.vec_into(),
            })
        }
    }
}

pub trait WithTotalCount {
    fn total_count(&self) -> i32;
}
