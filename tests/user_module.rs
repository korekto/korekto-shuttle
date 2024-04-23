use korekto::entities::{
    NewAssignmentBuilder, NewModuleBuilder, NewUserBuilder, UserModuleDescBuilder,
};
use korekto::service::{ObfuscatedStr, Service};
use time::OffsetDateTime;

mod common;

#[tokio::test]
#[cfg_attr(not(feature = "tests-with-docker"), ignore)]
async fn list_user_modules_gather_grades() -> anyhow::Result<()> {
    struct AssignmentStateDef {
        pub name: String,
        pub factor: i32,
        pub grade: f32,
        pub repo_linked: bool,
    }

    let assignment_states: Vec<AssignmentStateDef> = vec![
        AssignmentStateDef {
            name: "a1".to_string(),
            factor: 20,
            // 2.46
            grade: 12.3,
            repo_linked: true,
        },
        AssignmentStateDef {
            name: "a2".to_string(),
            factor: 20,
            // 4
            grade: 20.0,
            repo_linked: false,
        },
        AssignmentStateDef {
            name: "a3".to_string(),
            factor: 40,
            // 5.82
            grade: 14.55,
            repo_linked: true,
        },
    ];

    let service: Service = common::init_repo().await?.into();

    let user = service
        .repo
        .upsert_user(
            &NewUserBuilder::default()
                .provider_name("Jean Michel Machin")
                .provider_login("test-login")
                .provider_email("test@test.com")
                .avatar_url("https://github.githubassets.com/assets/GitHub-Mark-ea2971cee799.png")
                .build()?,
        )
        .await?;

    let now = OffsetDateTime::now_utc();

    let module = service
        .repo
        .create_module(
            &NewModuleBuilder::default()
                .name("test")
                .start(now.clone())
                .stop(now.clone())
                .unlock_key("test")
                .build()?,
        )
        .await?;

    for assignment_state in &assignment_states {
        let assignment = service
            .repo
            .create_assignment(
                &module.uuid,
                &NewAssignmentBuilder::default()
                    .name(&assignment_state.name)
                    .factor_percentage(assignment_state.factor)
                    .repository_name(&assignment_state.name)
                    .build()?,
            )
            .await?;

        service
            .repo
            .upsert_user_assignments(
                &user.provider_login,
                &[&assignment_state.name],
                assignment_state.repo_linked,
            )
            .await?;

        service
            .repo
            .update_assignment_grade(user.id, assignment.id, assignment_state.grade, &now)
            .await?;
    }

    service
        .redeem_module(&ObfuscatedStr("test".to_string()), &user)
        .await?;

    service
        .repo
        .create_assignment(
            &module.uuid,
            &NewAssignmentBuilder::default()
                .name("a0")
                .factor_percentage(20)
                .build()?,
        )
        .await?;

    let user_modules = service.repo.list_modules(&user).await?;

    let computed_user_module_desc = user_modules.into_iter().next().unwrap();

    pretty_assertions::assert_eq!(
        computed_user_module_desc,
        UserModuleDescBuilder::default()
            .id(module.id)
            .uuid(module.uuid)
            .name(module.name)
            .start(computed_user_module_desc.start.clone())
            .stop(computed_user_module_desc.stop.clone())
            .linked_repo_count(2)
            .assignment_count(4)
            .grade(12.28)
            .latest_update(computed_user_module_desc.latest_update.unwrap().clone())
            .build()?
    );

    Ok(())
}
