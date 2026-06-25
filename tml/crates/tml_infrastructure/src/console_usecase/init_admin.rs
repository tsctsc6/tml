use crate::entity::auth::role;
use crate::entity::auth::user;
use crate::entity::auth::user_role;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use tml_application::console_usecase::init_admin;

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
impl init_admin::repository::Trait for Repository {
    async fn is_no_admin(&self) -> Result<bool, init_admin::repository::Error> {
        let admin_role = role::Entity::find_by_name("admin")
            .one(&self.db)
            .await
            .map_err(|e| -> init_admin::repository::Error {
                init_admin::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(init_admin::repository::Error::Unknown(
                "role \"admin\" not found".to_string(),
            ))?;
        let count = user_role::Entity::find()
            .filter(user_role::Column::RoleId.eq(admin_role.id))
            .count(&self.db)
            .await
            .map_err(|e| -> init_admin::repository::Error {
                init_admin::repository::Error::Unknown(e.to_string())
            })?;
        Ok(count == 0)
    }

    async fn create_admin(
        &self,
        username: &str,
        password_hash: &str,
    ) -> Result<tml_domain::model::auth::user::Model, init_admin::repository::Error> {
        let user_to_create = user::ActiveModel {
            username: Set(username.into()),
            password_hash: Set(password_hash.into()),
            enabled: Set(true),
            created_at: Set(chrono::Utc::now()),
            security_stamp: Set(uuid::Uuid::new_v4()),
            ..Default::default()
        };
        let new_user = match user_to_create.insert(&self.db).await {
            Ok(user) => user,
            Err(e) => match e.sql_err() {
                _ => {
                    return Err(init_admin::repository::Error::Unknown(e.to_string()));
                }
            },
        };
        let admin_role = role::Entity::find_by_name("admin")
            .one(&self.db)
            .await
            .map_err(|e| -> init_admin::repository::Error {
                init_admin::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(init_admin::repository::Error::Unknown(
                "role \"admin\" not found".to_string(),
            ))?;
        let user_role_to_create = user_role::ActiveModel {
            user_id: Set(new_user.id),
            role_id: Set(admin_role.id),
        };
        let _new_user_role = match user_role_to_create.insert(&self.db).await {
            Ok(user) => user,
            Err(e) => match e.sql_err() {
                _ => {
                    return Err(init_admin::repository::Error::Unknown(e.to_string()));
                }
            },
        };
        Ok(new_user.into())
    }
}
