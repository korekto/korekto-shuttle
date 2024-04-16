use assert_matches2::assert_matches;
use korekto::entities::{NewUser, User};
use korekto::repository::Repository;
use pretty_assertions::assert_eq;
use sqlx::PgPool;

async fn init_repo() -> anyhow::Result<Repository> {
    let pg_pool: PgPool =
        PgPool::connect("postgres://postgres:mysecretpassword@127.0.0.1:5433/postgres")
            .await
            .expect("Unable to connect to Database");

    let repository = Repository::new(pg_pool);

    repository.wipe_database().await?;
    repository.run_migrations().await?;

    Ok(repository)
}

#[tokio::test]
async fn upsert_user_insert_it_when_missing() -> anyhow::Result<()> {
    let repo = init_repo().await?;

    let user = NewUser {
        provider_name: "Jean Michel Machin".to_string(),
        provider_login: "test-login".to_string(),
        provider_email: "test@test.com".to_string(),
        avatar_url: "https://github.githubassets.com/assets/GitHub-Mark-ea2971cee799.png"
            .to_string(),
        github_user_tokens: None,
    };

    let persisted_user = repo.upsert_user(&user).await?;

    assert_matches!(
        persisted_user,
        User {
            provider_name,
            provider_login,
            provider_email,
            avatar_url,
            admin: false,
            teacher: false,
            first_name,
            last_name,
            school_group,
            school_email,
            ..
        }
    );

    assert_eq!(provider_name, "Jean Michel Machin");
    assert_eq!(provider_login, "test-login");
    assert_eq!(provider_email, "test@test.com");
    assert_eq!(
        avatar_url,
        "https://github.githubassets.com/assets/GitHub-Mark-ea2971cee799.png"
    );
    assert_eq!(first_name, "Jean");
    assert_eq!(last_name, "Michel Machin");
    assert_eq!(school_group, "");
    assert_eq!(school_email, "test@test.com");

    Ok(())
}
