use korekto::repository::Repository;
use sqlx::PgPool;

pub async fn init_repo() -> anyhow::Result<Repository> {
    let pg_pool: PgPool =
        PgPool::connect("postgres://postgres:mysecretpassword@127.0.0.1:5433/postgres")
            .await
            .expect("Unable to connect to Database");

    let repository = Repository::new(pg_pool);

    repository.wipe_database().await?;
    repository.run_migrations().await?;

    Ok(repository)
}
