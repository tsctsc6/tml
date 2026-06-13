use crate::entity::app::{music_info_music_list, music_list};
use sea_orm::EntityTrait;
use tml_application::usecase::app::remove_music_info_from_music_list;

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
impl remove_music_info_from_music_list::repository::Trait for Repository {
    async fn remove_music_info_from_music_list(
        &self,
        music_list_id: i64,
        music_info_id: &[u8],
    ) -> Result<(), remove_music_info_from_music_list::repository::Error> {
        let result =
            music_info_music_list::Entity::delete_by_id((music_info_id.to_owned(), music_list_id))
                .exec(&self.db)
                .await
                .map_err(
                    |e| -> remove_music_info_from_music_list::repository::Error {
                        remove_music_info_from_music_list::repository::Error::Unknown(e.to_string())
                    },
                )?;
        if result.rows_affected == 0 {
            Err(remove_music_info_from_music_list::repository::Error::MusicInfoNotInMusicList)?;
        }
        Ok(())
    }

    async fn get_music_list_owner_id(
        &self,
        music_list_id: i64,
    ) -> Result<i64, remove_music_info_from_music_list::repository::Error> {
        let music_list = music_list::Entity::find_by_id(music_list_id)
            .one(&self.db)
            .await
            .map_err(
                |e| -> remove_music_info_from_music_list::repository::Error {
                    remove_music_info_from_music_list::repository::Error::Unknown(e.to_string())
                },
            )?
            .ok_or(remove_music_info_from_music_list::repository::Error::MusicListNotFound)?;
        Ok(music_list.user_id)
    }
}
