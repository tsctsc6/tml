use crate::entity::mgmt::job;
use sea_orm::EntityTrait;
use tml_application::usecase::mgmt::delete_job;

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
impl delete_job::repository::Trait for Repository {
    async fn delete_job(&self, id: i64) -> Result<(), delete_job::repository::Error> {
        let result = job::Entity::delete_by_id(id).exec(&self.db).await.map_err(
            |e| -> delete_job::repository::Error {
                delete_job::repository::Error::Unknown(e.to_string())
            },
        )?;
        if result.rows_affected == 0 {
            return Err(delete_job::repository::Error::JobNotFound);
        }
        Ok(())
    }
}
