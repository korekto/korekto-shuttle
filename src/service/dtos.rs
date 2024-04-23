use crate::entities::UserModuleDesc;
use time::serde::rfc3339 as time_serde;
use time::OffsetDateTime;

pub trait VecInto<D> {
    fn vec_into(self) -> Vec<D>;
}

impl<E, D> VecInto<D> for Vec<E>
where
    D: From<E>,
{
    fn vec_into(self) -> Vec<D> {
        self.into_iter().map(std::convert::Into::into).collect()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UserModuleResponse {
    pub id: String,
    pub name: String,
    #[serde(with = "time_serde")]
    pub start: OffsetDateTime,
    #[serde(with = "time_serde")]
    pub stop: OffsetDateTime,
    pub linked_repo_count: i32,
    pub assignment_count: i32,
    pub grade: f32,
    //#[serde(with = "time_serde")]
    pub latest_update: Option<OffsetDateTime>,
}

impl From<UserModuleDesc> for UserModuleResponse {
    fn from(value: UserModuleDesc) -> Self {
        Self {
            id: value.uuid,
            name: value.name,
            start: value.start,
            stop: value.stop,
            linked_repo_count: value.linked_repo_count,
            assignment_count: value.assignment_count,
            grade: value.grade,
            latest_update: value.latest_update,
        }
    }
}
