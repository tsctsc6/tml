use crate::entity::auth::user;
use moka::future::Cache;
use tml_application::usecase::auth::login;

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
impl login::repository::Trait for Repository {
    async fn find_user_by_username(
        &self,
        username: &str,
    ) -> Result<tml_domain::model::auth::user::Model, login::repository::Error> {
        let user = user::Entity::find_by_username(username)
            .one(&self.db)
            .await
            .map_err(|e| -> login::repository::Error {
                login::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(login::repository::Error::UserNotFound)?;
        self.cache.insert(user.id, user.security_stamp).await;
        Ok(user.into())
    }
}
