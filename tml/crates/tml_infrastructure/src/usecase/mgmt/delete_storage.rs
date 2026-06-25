use crate::entity::mgmt::storage;
use sea_orm::EntityTrait;
use tml_application::usecase::mgmt::delete_storage;

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
impl delete_storage::repository::Trait for Repository {
    async fn delete_storage(&self, id: i64) -> Result<(), delete_storage::repository::Error> {
        let result = storage::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| -> delete_storage::repository::Error {
                delete_storage::repository::Error::Unknown(e.to_string())
            })?;
        if result.rows_affected == 0 {
            return Err(delete_storage::repository::Error::StorageNotFound);
        }
        Ok(())
    }
}
