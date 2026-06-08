use crate::entity::mgmt::storage;
use sea_orm::{EntityTrait, PaginatorTrait, QueryOrder};
use tml_application::usecase::mgmt::read_all_storage;

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
impl read_all_storage::repository::Trait for Repository {
    async fn read_all_storage(
        &self,
        page_index: u64,
        page_size: u64,
    ) -> Result<read_all_storage::repository::PageResult, read_all_storage::repository::Error> {
        let paginator = storage::Entity::find()
            .order_by_asc(storage::Column::Id)
            .paginate(&self.db, page_size);

        let total = paginator
            .num_items()
            .await
            .map_err(|e| read_all_storage::repository::Error::Unknown(e.to_string()))?;

        let items = paginator
            .fetch_page(page_index)
            .await
            .map_err(|e| read_all_storage::repository::Error::Unknown(e.to_string()))?;

        Ok(read_all_storage::repository::PageResult {
            items: items.into_iter().map(|m| m.into()).collect(),
            total,
        })
    }
}
