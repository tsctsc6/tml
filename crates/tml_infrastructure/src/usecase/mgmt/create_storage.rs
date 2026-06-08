use crate::entity::mgmt::storage;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, SqlErr};
use tml_application::usecase::mgmt::create_storage;

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
impl create_storage::repository::Trait for Repository {
    async fn create_storage(
        &self,
        name: &str,
        path: &str,
    ) -> Result<tml_domain::model::mgmt::storage::Model, create_storage::repository::Error> {
        let storage_to_create = storage::ActiveModel {
            name: Set(name.into()),
            path: Set(path.into()),
            ..Default::default()
        };
        let new_storage = match storage_to_create.insert(&self.db).await {
            Ok(storage) => storage,
            Err(e) => match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(detail))
                    if detail.contains("storage_name_key") =>
                {
                    return Err(create_storage::repository::Error::NameDuplication);
                }
                Some(SqlErr::UniqueConstraintViolation(detail))
                    if detail.contains("storage_path_key") =>
                {
                    return Err(create_storage::repository::Error::PathDuplication);
                }
                _ => {
                    return Err(create_storage::repository::Error::Unknown(e.to_string()));
                }
            },
        };
        Ok(new_storage.into())
    }
}
