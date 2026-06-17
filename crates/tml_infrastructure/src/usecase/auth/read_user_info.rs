use crate::entity::auth::role;
use crate::entity::auth::user;
use sea_orm::EntityTrait;
use tml_application::usecase::auth::read_user_info;

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
impl read_user_info::repository::Trait for Repository {
    async fn find_user_by_id(
        &self,
        id: i64,
    ) -> Result<
        (tml_domain::model::auth::user::Model, Vec<String>),
        read_user_info::repository::Error,
    > {
        let (user, roles) = user::Entity::find_by_id(id)
            .find_with_related(role::Entity)
            .all(&self.db)
            .await
            .map_err(|e| -> read_user_info::repository::Error {
                read_user_info::repository::Error::Unknown(e.to_string())
            })?
            .into_iter()
            .next()
            .ok_or(read_user_info::repository::Error::UserNotFound)?;
        let roles = roles.into_iter().map(|x| x.name).collect();
        Ok((user.into(), roles))
    }
}
