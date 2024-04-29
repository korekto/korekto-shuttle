use assert_matches2::assert_matches;
use korekto::service::dtos::{Page, PaginationQuery};
use korekto::service::Service;

mod common;

#[tokio::test]
#[cfg_attr(not(feature = "tests-with-docker"), ignore)]
async fn upsert_user_insert_it_when_missing() -> anyhow::Result<()> {
    let service: Service = common::init_repo().await?.into();
    service
        .repo
        .insert_unparseable_webhook("test", "toto", "{}", "A test payload meant to be in error")
        .await?;

    let page = service
        .get_unparseable_webhooks(&PaginationQuery::new(1, 10))
        .await?;

    assert_matches!(
        page,
        Page {
            page: 1,
            per_page: 10,
            total_page: 1,
            total_count: 1,
            data,
        }
    );

    pretty_assertions::assert_eq!(data.len(), 1, "Length of page data field");

    Ok(())
}
