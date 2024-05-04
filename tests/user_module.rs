use korekto::entities::{Module, NewAssignmentBuilder, NewModuleBuilder, NewUserBuilder, User};
use korekto::service::dtos::{
    CompleteRunInfoResponseBuilder, DetailsResponseBuilder, NewGradeDetailRequest, NewGradeRequest,
    UserAssignmentDescResponse, UserAssignmentDescResponseBuilder, UserAssignmentResponse,
    UserAssignmentResponseBuilder, UserModuleDescResponse, UserModuleDescResponseBuilder,
    UserModuleResponse, UserModuleResponseBuilder,
};
use korekto::service::{ObfuscatedStr, Service};
use rust_decimal::Decimal;
use time::OffsetDateTime;

mod common;

struct AssignmentStateDef<'a> {
    pub name: &'a str,
    pub factor: i32,
    pub grades: &'a [(f32, Option<f32>)],
    pub normalized_grade: &'a str,
    pub repo_linked: bool,
}

static ASSIGNMENT_STATES: &[AssignmentStateDef] = &[
    AssignmentStateDef {
        name: "a1",
        factor: 20,
        grades: &[(4.0, Some(4.0)), (6.3, Some(8.0))],
        normalized_grade: "17.17",
        repo_linked: true,
    },
    AssignmentStateDef {
        name: "a2",
        factor: 20,
        // 4
        grades: &[(20.0, Some(20.0))],
        normalized_grade: "20",
        repo_linked: false,
    },
    AssignmentStateDef {
        name: "a3",
        factor: 40,
        // 5.82
        grades: &[(4.5, Some(6.0)), (12.05, Some(14.0)), (-3.0, None)],
        normalized_grade: "13.55",
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

        let grade = NewGradeRequest {
            time: Some(OffsetDateTime::UNIX_EPOCH),
            short_commit_id: "toto123".to_string(),
            commit_url: "githubmachin/commits/toto123".to_string(),
            grading_log_url: "githubmachin/job/log".to_string(),
            details: assignment_state
                .grades
                .iter()
                .enumerate()
                .map(|(index, grade)| NewGradeDetailRequest {
                    name: format!("step {index}"),
                    grade: grade.0,
                    max_grade: grade.1,
                    messages: vec![],
                })
                .collect(),
        };

        service
            .update_assignment_grade(&user.uuid, &assignment.uuid, grade)
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
        .redeem_module(&ObfuscatedStr::new("test"), &user)
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
            .id(&module.assignments[state_index].uuid)
            .name(ASSIGNMENT_STATES[state_index].name)
            .description("")
            .start(module.assignments[state_index].start.clone())
            .stop(module.assignments[state_index].stop.clone())
            .a_type("")
            .factor_percentage(ASSIGNMENT_STATES[state_index].factor)
            .locked(false)
            .grade(Decimal::from_str_exact(
                ASSIGNMENT_STATES[state_index].normalized_grade,
            )?)
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
        .redeem_module(&ObfuscatedStr::new("test"), &user)
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

    let computed_user_module: UserModuleDescResponse =
        user_modules.into_iter().next().unwrap().into();

    pretty_assertions::assert_eq!(
        computed_user_module,
        UserModuleDescResponseBuilder::default()
            .id(module.uuid)
            .name(module.name)
            .start(computed_user_module.start.clone())
            .stop(computed_user_module.stop.clone())
            .linked_repo_count(2)
            .assignment_count(4)
            .grade(Decimal::from_str_exact("12.85")?)
            .latest_update(computed_user_module.latest_update.unwrap().clone())
            .build()?
    );

    Ok(())
}

#[tokio::test]
#[cfg_attr(not(feature = "tests-with-docker"), ignore)]
async fn get_user_assignment_query() -> anyhow::Result<()> {
    let service: Service = common::init_repo().await?.into();

    let (user, module) = setup_data(&service).await?;

    service
        .redeem_module(&ObfuscatedStr::new("test"), &user)
        .await?;

    let assignment_uuid = &module.assignments[0].uuid;
    let assignment: UserAssignmentResponse = service
        .repo
        .get_assignment(&user, &module.uuid, assignment_uuid)
        .await?
        .unwrap()
        .into();

    pretty_assertions::assert_eq!(
        assignment,
        UserAssignmentResponseBuilder::default()
            .id(assignment_uuid)
            .a_type("")
            .name("a1")
            .description("")
            .start(OffsetDateTime::UNIX_EPOCH)
            .stop(OffsetDateTime::UNIX_EPOCH)
            .repo_linked(true)
            .repository_name("a1")
            .subject_url("")
            .grader_url("")
            .repository_url("https://github.com/test-login/a1")
            .factor_percentage(ASSIGNMENT_STATES[0].factor)
            .normalized_grade(17.17)
            .locked(false)
            .latest_run(
                CompleteRunInfoResponseBuilder::default()
                    .short_commit_id("toto123")
                    .commit_url("githubmachin/commits/toto123")
                    .grading_log_url("githubmachin/job/log")
                    .time(OffsetDateTime::UNIX_EPOCH)
                    .details(vec![
                        DetailsResponseBuilder::default()
                            .name("step 0")
                            .grade(4.0)
                            .max_grade(4.0)
                            .build()?,
                        DetailsResponseBuilder::default()
                            .name("step 1")
                            .grade(6.3)
                            .max_grade(8.0)
                            .build()?
                    ])
                    .build()?
            )
            .build()?
    );

    Ok(())
}
