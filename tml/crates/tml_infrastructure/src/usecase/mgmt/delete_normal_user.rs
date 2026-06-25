use crate::entity::auth::role;
use crate::entity::auth::user;
use crate::entity::auth::user_role;
use sea_orm::EntityTrait;
use tml_application::usecase::mgmt::delete_normal_user;

#[derive(Clone)]
pub struct Repository {
    db: sea_orm::DatabaseConnection,
}

impl Repository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Repository { db }
    }
}

#[async_trait::async_trait]
impl delete_normal_user::repository::Trait for Repository {
    async fn delete_normal_user(
        &self,
        id: i64,
    ) -> Result<(), delete_normal_user::repository::Error> {
        let normal_user_role = role::Entity::find_by_name("normal-user")
            .one(&self.db)
            .await
            .map_err(|e| -> delete_normal_user::repository::Error {
                delete_normal_user::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(delete_normal_user::repository::Error::Unknown(
                "role \"normal-user\" not found".to_string(),
            ))?;

        let _ = user_role::Entity::find_by_id((id, normal_user_role.id))
            .one(&self.db)
            .await
            .map_err(|e| -> delete_normal_user::repository::Error {
                delete_normal_user::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(delete_normal_user::repository::Error::UserNotNormalUser)?;

        let result = user::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| -> delete_normal_user::repository::Error {
                delete_normal_user::repository::Error::Unknown(e.to_string())
            })?;

        if result.rows_affected == 0 {
            return Err(delete_normal_user::repository::Error::UserNotFound);
        }
        Ok(())
    }
}
