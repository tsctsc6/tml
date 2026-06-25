use crate::entity::mgmt::job;
use sea_orm::EntityTrait;
use tml_application::usecase::mgmt::read_job;

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
impl read_job::repository::Trait for Repository {
    async fn read_job(
        &self,
        id: i64,
    ) -> Result<tml_domain::model::mgmt::job::Model, read_job::repository::Error> {
        let result = job::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| read_job::repository::Error::Unknown(e.to_string()))?;
        match result {
            Some(job) => Ok(job.into()),
            None => Err(read_job::repository::Error::JobNotFound),
        }
    }
}
