use assert_matches2::assert_matches;
use korekto::entities;

mod common;

#[tokio::test]
#[cfg_attr(not(feature = "tests-with-docker"), ignore)]
async fn upsert_user_insert_it_when_missing() -> anyhow::Result<()> {
    let repo = common::init_repo().await?;

    let user = entities::NewUserBuilder::default()
        .provider_name("Jean Michel Machin")
        .provider_login("test-login")
        .provider_email("test@test.com")
        .avatar_url("https://github.githubassets.com/assets/GitHub-Mark-ea2971cee799.png")
        .build()?;

    let persisted_user = repo.upsert_user(&user).await?;

    assert_matches!(
        persisted_user,
        entities::User {
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

    pretty_assertions::assert_eq!(provider_name, "Jean Michel Machin");
    pretty_assertions::assert_eq!(provider_login, "test-login");
    pretty_assertions::assert_eq!(provider_email, "test@test.com");
    pretty_assertions::assert_eq!(
        avatar_url,
        "https://github.githubassets.com/assets/GitHub-Mark-ea2971cee799.png"
    );
    pretty_assertions::assert_eq!(first_name, "Jean");
    pretty_assertions::assert_eq!(last_name, "Michel Machin");
    pretty_assertions::assert_eq!(school_group, "");
    pretty_assertions::assert_eq!(school_email, "test@test.com");

    Ok(())
}
