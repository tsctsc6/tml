use crate::entity::auth::user;
use moka::future::Cache;
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use tml_application::console_usecase::reset_password;

pub struct Repository {
    db: sea_orm::DatabaseConnection,
    cache: Cache<i64, uuid::Uuid>,
}

impl Repository {
    pub fn new(db: sea_orm::DatabaseConnection, cache: Cache<i64, uuid::Uuid>) -> Self {
        Repository { db, cache }
    }
}

#[async_trait::async_trait]
impl reset_password::repository::Trait for Repository {
    async fn set_password_hash(
        &self,
        username: &str,
        password_hash: &str,
    ) -> Result<(), reset_password::repository::Error> {
        let user = user::Entity::find_by_username(username)
            .one(&self.db)
            .await
            .map_err(|e| -> reset_password::repository::Error {
                reset_password::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(reset_password::repository::Error::UserNotFound)?;
        let mut user: user::ActiveModel = user.into();
        user.password_hash = Set(password_hash.to_string());
        user.security_stamp = Set(uuid::Uuid::new_v4());
        let user =
            user.update(&self.db)
                .await
                .map_err(|e| -> reset_password::repository::Error {
                    reset_password::repository::Error::Unknown(e.to_string())
                })?;
        self.cache.invalidate(&user.id).await;
        Ok(())
    }
}
