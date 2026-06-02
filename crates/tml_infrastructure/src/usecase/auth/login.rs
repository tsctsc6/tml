use crate::entity::auth::user;
use tml_application::usecase::auth::login;

pub struct Repository {
    db: sea_orm::DatabaseConnection,
}

impl Repository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Repository { db }
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
        Ok(user.into())
    }
}
