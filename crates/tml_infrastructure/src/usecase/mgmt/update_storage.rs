use crate::entity::mgmt::storage;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, SqlErr};
use tml_application::usecase::mgmt::update_storage;

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
impl update_storage::repository::Trait for Repository {
    async fn update_storage(
        &self,
        id: i64,
        name: &str,
        path: &str,
    ) -> Result<tml_domain::model::mgmt::storage::Model, update_storage::repository::Error> {
        let storage_to_update = storage::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| -> update_storage::repository::Error {
                update_storage::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(update_storage::repository::Error::StorageNotFound)?;
        let mut storage_to_update: storage::ActiveModel = storage_to_update.into();
        storage_to_update.name = Set(name.to_string());
        storage_to_update.path = Set(path.to_string());
        let updated_storage = match storage_to_update.update(&self.db).await {
            Ok(storage) => storage,
            Err(e) => match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(detail))
                    if detail.contains("storage_name_key") =>
                {
                    return Err(update_storage::repository::Error::NameDuplication);
                }
                Some(SqlErr::UniqueConstraintViolation(detail))
                    if detail.contains("storage_path_key") =>
                {
                    return Err(update_storage::repository::Error::PathDuplication);
                }
                _ => {
                    return Err(update_storage::repository::Error::Unknown(e.to_string()));
                }
            },
        };
        Ok(updated_storage.into())
    }
}
