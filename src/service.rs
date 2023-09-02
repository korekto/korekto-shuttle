use sqlx::PgPool;

use crate::repository::Repository;

mod find_user_by_id;

#[derive(Clone)]
pub struct Service {
    pub repo: Repository,
}

impl Service {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        let repo = Repository::new(pool);
        Self { repo }
    }
}
