use crate::entities;
use crate::service::Service;

impl Service {
    pub async fn find_user_by_id(&self, user_id: &i32) -> Option<entities::User> {
        self.repo.find_user_by_id(user_id).await.ok()
    }
}
