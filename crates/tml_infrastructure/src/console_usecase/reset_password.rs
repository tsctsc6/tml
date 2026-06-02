use crate::entity::auth::role;
use crate::entity::auth::user;
use crate::entity::auth::user_role;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;
use sea_orm::QueryFilter;
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use tml_application::console_usecase::reset_password;

pub struct Repository {
    db: sea_orm::DatabaseConnection,
}

impl Repository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Repository { db }
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
        let _user =
            user.update(&self.db)
                .await
                .map_err(|e| -> reset_password::repository::Error {
                    reset_password::repository::Error::Unknown(e.to_string())
                })?;
        Ok(())
    }
}
