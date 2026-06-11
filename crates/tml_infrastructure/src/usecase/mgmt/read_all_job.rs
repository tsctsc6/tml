use crate::entity::mgmt::job;
use sea_orm::EntityTrait;
use tml_application::usecase::mgmt::read_all_job;

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
impl read_all_job::repository::Trait for Repository {
    async fn read_all_job(
        &self,
        page_size: u64,
        created_after: chrono::DateTime<chrono::Utc>,
        created_before: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<tml_domain::model::mgmt::job::Model>, read_all_job::repository::Error> {
        tracing::debug!("{}, {}, {}", page_size, created_after, created_before);
        let result = job::Entity::find()
            .cursor_by(job::Column::CreatedAt)
            .desc()
            // because of desc, before and after swap
            .after(created_before)
            .before(created_after)
            .first(page_size)
            .all(&self.db)
            .await
            .map_err(|e| read_all_job::repository::Error::Unknown(e.to_string()))?
            .into_iter()
            .map(|x| x.into())
            .collect();
        Ok(result)
    }
}
