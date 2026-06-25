use crate::entity::app::music_list;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter as _};
use tml_application::usecase::app::read_all_music_list;

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
impl read_all_music_list::repository::Trait for Repository {
    async fn read_all_music_list(
        &self,
        page_size: u64,
        cursor: i64,
        user_id: i64,
    ) -> Result<
        Vec<tml_domain::model::app::music_list::Model>,
        read_all_music_list::repository::Error,
    > {
        let result = music_list::Entity::find()
            .filter(music_list::Column::UserId.eq(user_id))
            .cursor_by(music_list::Column::Id)
            .desc()
            .after(cursor)
            .first(page_size)
            .all(&self.db)
            .await
            .map_err(|e| read_all_music_list::repository::Error::Unknown(e.to_string()))?
            .into_iter()
            .map(|x| x.into())
            .collect();
        Ok(result)
    }
}
