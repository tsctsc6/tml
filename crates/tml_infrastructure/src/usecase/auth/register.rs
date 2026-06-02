use crate::entity::auth::user;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, SqlErr};
use tml_application::usecase::auth::register;

pub struct Repository {
    db: sea_orm::DatabaseConnection,
}

impl Repository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Repository { db }
    }
}

#[async_trait::async_trait]
impl register::repository::Trait for Repository {
    async fn create_user(
        &self,
        username: &str,
        password_hash: &str,
    ) -> Result<tml_domain::model::auth::user::Model, register::repository::Error> {
        let user_to_create = user::ActiveModel {
            username: Set(username.into()),
            password_hash: Set(password_hash.into()),
            ..Default::default()
        };
        let user_to_create = match user_to_create.insert(&self.db).await {
            Ok(user) => user,
            Err(e) => match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(detail)) => {
                    return Err(register::repository::Error::UniqueIndex(detail));
                }
                _ => {
                    return Err(register::repository::Error::Unknown(e.to_string()));
                }
            },
        };
        Ok(user_to_create.into())
    }
}
