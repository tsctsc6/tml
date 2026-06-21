use crate::entity::auth::role;
use crate::entity::auth::user;
use crate::entity::auth::user_role;
use moka::future::Cache;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, SqlErr};
use tml_application::usecase::mgmt::update_normal_user;

#[derive(Clone)]
pub struct Repository {
    db: sea_orm::DatabaseConnection,
    cache: Cache<i64, Option<uuid::Uuid>>,
}

impl Repository {
    pub fn new(db: sea_orm::DatabaseConnection, cache: Cache<i64, Option<uuid::Uuid>>) -> Self {
        Repository { db, cache }
    }
}

#[async_trait::async_trait]
impl update_normal_user::repository::Trait for Repository {
    async fn update_normal_user(
        &self,
        id: i64,
        username: Option<&str>,
        password_hash: Option<&str>,
        enabled: Option<bool>,
    ) -> Result<tml_domain::model::auth::user::Model, update_normal_user::repository::Error> {
        let normal_user_role = role::Entity::find_by_name("normal-user")
            .one(&self.db)
            .await
            .map_err(|e| -> update_normal_user::repository::Error {
                update_normal_user::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(update_normal_user::repository::Error::Unknown(
                "role \"normal-user\" not found".to_string(),
            ))?;
        let _ = user_role::Entity::find_by_id((id, normal_user_role.id))
            .one(&self.db)
            .await
            .map_err(|e| -> update_normal_user::repository::Error {
                update_normal_user::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(update_normal_user::repository::Error::UserNotNormalUser)?;

        let user_to_update = user::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| -> update_normal_user::repository::Error {
                update_normal_user::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(update_normal_user::repository::Error::UserNotFound)?;

        let mut user_to_update: user::ActiveModel = user_to_update.into();
        let mut need_update_security_stamp = false;

        if let Some(username) = username {
            user_to_update.username = Set(username.to_string());
        }
        if let Some(password_hash) = password_hash {
            user_to_update.password_hash = Set(password_hash.to_string());
            need_update_security_stamp = true;
        }
        if let Some(enabled) = enabled {
            user_to_update.enabled = Set(enabled);
            need_update_security_stamp = true;
        }

        if need_update_security_stamp {
            user_to_update.security_stamp = Set(uuid::Uuid::new_v4());
        }

        let updated_user = match user_to_update.update(&self.db).await {
            Ok(user) => user,
            Err(e) => match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(_)) => {
                    return Err(update_normal_user::repository::Error::UsernameDuplication);
                }
                _ => {
                    return Err(update_normal_user::repository::Error::Unknown(
                        e.to_string(),
                    ));
                }
            },
        };

        if need_update_security_stamp {
            self.cache.invalidate(&id).await;
        }

        Ok(updated_user.into())
    }
}
