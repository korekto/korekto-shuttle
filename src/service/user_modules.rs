use crate::entities::{ModuleId, User};
use crate::service::{ObfuscatedStr, Service};
use anyhow::anyhow;

impl Service {
    pub async fn redeem_module(
        &self,
        key: &ObfuscatedStr,
        user: &User,
    ) -> anyhow::Result<ModuleId> {
        let option = self.repo.find_module_by_key(key).await?;
        match option {
            Some(module_desc) => {
                self.repo.create_user_module(user, module_desc.id).await?;
                Ok(ModuleId {
                    uuid: module_desc.uuid,
                })
            }
            None => Err(anyhow!("No module matching the given key: {key}")),
        }
    }
}
