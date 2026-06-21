use crate::entity::app::music_list;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, SqlErr};
use tml_application::usecase::app::update_music_list;

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
impl update_music_list::repository::Trait for Repository {
    async fn update_music_list(
        &self,
        id: i64,
        name: &str,
    ) -> Result<tml_domain::model::app::music_list::Model, update_music_list::repository::Error>
    {
        let music_list_to_update = music_list::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| -> update_music_list::repository::Error {
                update_music_list::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(update_music_list::repository::Error::MusicListNotFound)?;
        let mut music_list_to_update: music_list::ActiveModel = music_list_to_update.into();
        music_list_to_update.name = Set(name.to_string());
        let updated_music_list = match music_list_to_update.update(&self.db).await {
            Ok(music_list) => music_list,
            Err(e) => match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(detail))
                    if detail.contains("music_list_name_key") =>
                {
                    return Err(update_music_list::repository::Error::NameDuplication);
                }
                _ => {
                    return Err(update_music_list::repository::Error::Unknown(e.to_string()));
                }
            },
        };
        Ok(updated_music_list.into())
    }

    async fn get_music_list_owner_id(
        &self,
        music_list_id: i64,
    ) -> Result<i64, update_music_list::repository::Error> {
        let music_list = music_list::Entity::find_by_id(music_list_id)
            .one(&self.db)
            .await
            .map_err(|e| -> update_music_list::repository::Error {
                update_music_list::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(update_music_list::repository::Error::MusicListNotFound)?;
        Ok(music_list.user_id)
    }
}
