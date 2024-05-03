use korekto::entities::{
    Module, NewAssignmentBuilder, NewModuleBuilder, NewUserBuilder, User, UserModuleDescBuilder,
};
use korekto::service::dtos::{
    UserAssignmentDescResponse, UserAssignmentDescResponseBuilder, UserModuleResponse,
    UserModuleResponseBuilder,
};
use korekto::service::{ObfuscatedStr, Service};
use time::OffsetDateTime;

mod common;

struct AssignmentStateDef<'a> {
    pub name: &'a str,
    pub factor: i32,
    pub grade: f32,
    pub repo_linked: bool,
}

static ASSIGNMENT_STATES: &[AssignmentStateDef] = &[
    AssignmentStateDef {
        name: "a1",
        factor: 20,
        // 2.46
        grade: 12.3,
        repo_linked: true,
    },
    AssignmentStateDef {
        name: "a2",
        factor: 20,
        // 4
        grade: 20.0,
        repo_linked: false,
    },
    AssignmentStateDef {
        name: "a3",
        factor: 40,
        // 5.82
        grade: 14.55,
        repo_linked: true,
    },
];

async fn setup_data(service: &Service) -> anyhow::Result<(User, Module)> {
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

    let module_v0 = service
        .repo
        .create_module(
            &NewModuleBuilder::default()
                .name("test")
                .description("test")
                .start(OffsetDateTime::UNIX_EPOCH)
                .stop(OffsetDateTime::UNIX_EPOCH)
                .unlock_key("test")
                .source_url("test")
                .build()?,
        )
        .await?;

    for assignment_state in ASSIGNMENT_STATES {
        let assignment = service
            .repo
            .create_assignment(
                &module_v0.uuid,
                &NewAssignmentBuilder::default()
                    .name(assignment_state.name)
                    .factor_percentage(assignment_state.factor)
                    .repository_name(assignment_state.name)
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
            .update_assignment_grade(
                user.id,
                assignment.id,
                assignment_state.grade,
                &OffsetDateTime::UNIX_EPOCH,
            )
            .await?;
    }

    let module = service.repo.find_module(&module_v0.uuid).await?;

    Ok((user, module))
}

#[tokio::test]
#[cfg_attr(not(feature = "tests-with-docker"), ignore)]
async fn get_user_module_gather_grades() -> anyhow::Result<()> {
    let service: Service = common::init_repo().await?.into();

    let (user, module) = setup_data(&service).await?;

    service
        .redeem_module(&ObfuscatedStr("test".to_string()), &user)
        .await?;

    let user_module: UserModuleResponse = service
        .repo
        .get_module(&user, &module.uuid)
        .await?
        .unwrap()
        .into();

    fn build_assignment_resp(
        module: &Module,
        state_index: usize,
    ) -> anyhow::Result<UserAssignmentDescResponse> {
        Ok(UserAssignmentDescResponseBuilder::default()
            .id(&module.assignments[state_index].id)
            .name(ASSIGNMENT_STATES[state_index].name)
            .description("")
            .start(module.assignments[state_index].start.clone())
            .stop(module.assignments[state_index].stop.clone())
            .a_type("")
            .factor_percentage(ASSIGNMENT_STATES[state_index].factor)
            .locked(false)
            .grade(ASSIGNMENT_STATES[state_index].grade)
            .repo_linked(ASSIGNMENT_STATES[state_index].repo_linked)
            .repository_name(ASSIGNMENT_STATES[state_index].name)
            .build()?)
    }

    pretty_assertions::assert_eq!(
        user_module,
        UserModuleResponseBuilder::default()
            .id(&module.uuid)
            .name(&module.name)
            .description(&module.description)
            .start(user_module.start.clone())
            .stop(user_module.stop.clone())
            .latest_update(user_module.latest_update.unwrap().clone())
            .source_url(&module.source_url)
            .locked(false)
            .assignments(vec![
                build_assignment_resp(&module, 0)?,
                build_assignment_resp(&module, 1)?,
                build_assignment_resp(&module, 2)?
            ])
            .build()?
    );

    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "tests-with-docker"), ignore)]
async fn list_user_modules_gather_grades() -> anyhow::Result<()> {
    let service: Service = common::init_repo().await?.into();

    let (user, module) = setup_data(&service).await?;

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
