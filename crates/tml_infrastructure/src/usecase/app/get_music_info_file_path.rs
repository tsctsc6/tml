use std::path::Path;

use sea_orm::EntityTrait;

use tml_application::usecase::app::get_music_info_file_path;

use crate::entity::app::music_info;
use crate::entity::mgmt::storage;

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
impl get_music_info_file_path::repository::Trait for Repository {
    async fn get_music_info_file_path(
        &self,
        music_info_id: Vec<u8>,
    ) -> Result<String, get_music_info_file_path::repository::Error> {
        let result = music_info::Entity::find_by_id(music_info_id)
            .find_also_related(storage::Entity)
            .one(&self.db)
            .await
            .map_err(|e| get_music_info_file_path::repository::Error::Unknown(e.to_string()))?
            .ok_or(get_music_info_file_path::repository::Error::MusicInfoNotFound)?;

        let (music_info_model, storage_model) = result;
        let storage = storage_model
            .ok_or(get_music_info_file_path::repository::Error::StorageNotFound)?;

        let file_path = Path::new(&storage.path)
            .join(&music_info_model.file_path)
            .to_string_lossy()
            .to_string();

        Ok(file_path)
    }
}
