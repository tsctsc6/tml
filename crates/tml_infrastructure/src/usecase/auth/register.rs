use crate::entity::auth::role;
use crate::entity::auth::user;
use crate::entity::auth::user_role;
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
            enabled: Set(true),
            created_at: Set(chrono::Utc::now()),
            secure_stamp: Set(uuid::Uuid::new_v4()),
            ..Default::default()
        };
        let new_user = match user_to_create.insert(&self.db).await {
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
        let normal_user_role = role::Entity::find_by_name("normal-user")
            .one(&self.db)
            .await
            .map_err(|e| -> register::repository::Error {
                register::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(register::repository::Error::Unknown(
                "role \"normal-user\" not found".to_string(),
            ))?;
        let user_role_to_create = user_role::ActiveModel {
            user_id: Set(new_user.id),
            role_id: Set(normal_user_role.id),
        };
        let _new_user_role = match user_role_to_create.insert(&self.db).await {
            Ok(user) => user,
            Err(e) => match e.sql_err() {
                _ => {
                    return Err(register::repository::Error::Unknown(e.to_string()));
                }
            },
        };
        Ok(new_user.into())
    }
}
