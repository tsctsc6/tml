use crate::entity::auth::role;
use crate::entity::auth::user;
use moka::future::Cache;
use tml_application::usecase::auth::login;

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
impl login::repository::Trait for Repository {
    async fn find_user_by_username(
        &self,
        username: &str,
    ) -> Result<(tml_domain::model::auth::user::Model, Vec<String>), login::repository::Error> {
        let (user, roles) = user::Entity::find_by_username(username)
            .find_with_related(role::Entity)
            .all(&self.db)
            .await
            .map_err(|e| -> login::repository::Error {
                login::repository::Error::Unknown(e.to_string())
            })?
            .into_iter()
            .next()
            .ok_or(login::repository::Error::UserNotFound)?;
        let roles = roles.into_iter().map(|x| x.name).collect();
        self.cache.invalidate(&user.id).await;
        Ok((user.into(), roles))
    }
}
