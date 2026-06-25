use crate::{entity::app::music_list, tx_context::SeaOrmTxConnection};
use sea_orm::{EntityTrait, QuerySelect as _, sea_query::LockType};
use tml_application::usecase::app::delete_music_list;

#[derive(Clone)]
pub struct Repository {}

impl Repository {
    pub fn new() -> Self {
        Repository {}
    }
}

#[async_trait::async_trait]
impl delete_music_list::repository::Trait for Repository {
    type Tx = SeaOrmTxConnection;

    async fn delete_music_list(
        &self,
        tx_connection: &mut Self::Tx,
        id: i64,
    ) -> Result<(), delete_music_list::repository::Error> {
        let result = music_list::Entity::delete_by_id(id)
            .exec(&tx_connection.tx)
            .await
            .map_err(|e| -> delete_music_list::repository::Error {
                delete_music_list::repository::Error::Unknown(e.to_string())
            })?;
        if result.rows_affected == 0 {
            Err(delete_music_list::repository::Error::MusicListNotFound)?;
        }
        Ok(())
    }

    async fn get_music_list_owner_id(
        &self,
        tx_connection: &mut Self::Tx,
        music_list_id: i64,
    ) -> Result<i64, delete_music_list::repository::Error> {
        let music_list = music_list::Entity::find_by_id(music_list_id)
            .lock(LockType::Update)
            .one(&tx_connection.tx)
            .await
            .map_err(|e| -> delete_music_list::repository::Error {
                delete_music_list::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(delete_music_list::repository::Error::MusicListNotFound)?;
        Ok(music_list.user_id)
    }
}
