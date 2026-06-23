use crate::{entity::app::music_list, tx_context::SeaOrmTxConnection};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, EntityTrait, QuerySelect, SqlErr, sea_query::LockType,
};
use tml_application::usecase::app::update_music_list;

#[derive(Clone)]
pub struct Repository {}

impl Repository {
    pub fn new() -> Self {
        Repository {}
    }
}

#[async_trait::async_trait]
impl update_music_list::repository::Trait for Repository {
    type Tx = SeaOrmTxConnection;

    async fn update_music_list(
        &self,
        tx_connection: &mut Self::Tx,
        id: i64,
        name: &str,
    ) -> Result<tml_domain::model::app::music_list::Model, update_music_list::repository::Error>
    {
        let music_list_to_update = music_list::Entity::find_by_id(id)
            .lock(LockType::Update)
            .one(&tx_connection.tx)
            .await
            .map_err(|e| -> update_music_list::repository::Error {
                update_music_list::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(update_music_list::repository::Error::MusicListNotFound)?;
        let mut music_list_to_update: music_list::ActiveModel = music_list_to_update.into();
        music_list_to_update.name = Set(name.to_string());
        let updated_music_list = match music_list_to_update.update(&tx_connection.tx).await {
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
        tx_connection: &mut Self::Tx,
        music_list_id: i64,
    ) -> Result<i64, update_music_list::repository::Error> {
        let music_list = music_list::Entity::find_by_id(music_list_id)
            .lock(LockType::Update)
            .one(&tx_connection.tx)
            .await
            .map_err(|e| -> update_music_list::repository::Error {
                update_music_list::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(update_music_list::repository::Error::MusicListNotFound)?;
        Ok(music_list.user_id)
    }
}
