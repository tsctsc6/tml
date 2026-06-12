use crate::entity::app::music_list;
use sea_orm::EntityTrait;
use tml_application::usecase::app::delete_music_list;

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
impl delete_music_list::repository::Trait for Repository {
    async fn delete_music_list(&self, id: i64) -> Result<(), delete_music_list::repository::Error> {
        music_list::Entity::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| -> delete_music_list::repository::Error {
                delete_music_list::repository::Error::Unknown(e.to_string())
            })?;
        Ok(())
    }

    async fn get_music_list_owner_id(
        &self,
        music_list_id: i64,
    ) -> Result<i64, delete_music_list::repository::Error> {
        let music_list = music_list::Entity::find_by_id(music_list_id)
            .one(&self.db)
            .await
            .map_err(|e| -> delete_music_list::repository::Error {
                delete_music_list::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(delete_music_list::repository::Error::MusicListNotFound)?;
        Ok(music_list.user_id)
    }
}
