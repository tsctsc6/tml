use crate::{
    entity::app::{music_info_music_list, music_list},
    tx_context::SeaOrmTxConnection,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, Order, QueryFilter as _,
    QueryOrder as _, QuerySelect as _, SqlErr, sea_query::LockType,
};
use tml_application::usecase::app::add_music_info_to_music_list;

#[derive(Clone)]
pub struct Repository {}

impl Repository {
    pub fn new() -> Self {
        Repository {}
    }
}

#[async_trait::async_trait]
impl add_music_info_to_music_list::repository::Trait for Repository {
    type Tx = SeaOrmTxConnection;

    async fn get_last_order(
        &self,
        tx_connection: &mut Self::Tx,
        music_list_id: i64,
    ) -> Result<Vec<u8>, add_music_info_to_music_list::repository::Error> {
        let last_entry = music_info_music_list::Entity::find()
            .filter(music_info_music_list::Column::MusicListId.eq(music_list_id))
            .order_by(music_info_music_list::Column::Order, Order::Desc)
            .lock(LockType::Update)
            .one(&tx_connection.tx)
            .await
            .map_err(|e| add_music_info_to_music_list::repository::Error::Unknown(e.to_string()))?;
        let last_order = last_entry.map(|x| x.order);
        match last_order {
            Some(o) => Ok(o),
            None => Ok(vec![]),
        }
    }

    async fn add_music_info_to_music_list(
        &self,
        tx_connection: &mut Self::Tx,
        music_list_id: i64,
        music_info_id: &[u8],
        order: &[u8],
    ) -> Result<
        tml_domain::model::app::music_info_music_list::Model,
        add_music_info_to_music_list::repository::Error,
    > {
        let new_entry = music_info_music_list::ActiveModel {
            music_info_id: Set(music_info_id.to_owned()),
            music_list_id: Set(music_list_id),
            order: Set(order.to_owned()),
        };
        let inserted = match new_entry.insert(&tx_connection.tx).await {
            Ok(record) => record,
            Err(e) => match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(_)) => {
                    return Err(
                        add_music_info_to_music_list::repository::Error::MusicInfoAlreadyInMusicList,
                    );
                }
                Some(SqlErr::ForeignKeyConstraintViolation(detail)) => {
                    if detail.contains("music_list_id") {
                        return Err(
                            add_music_info_to_music_list::repository::Error::MusicListNotFound,
                        );
                    }
                    if detail.contains("music_info_id") {
                        return Err(
                            add_music_info_to_music_list::repository::Error::MusicInfoNotFound,
                        );
                    }
                    return Err(add_music_info_to_music_list::repository::Error::Unknown(
                        e.to_string(),
                    ));
                }
                _ => {
                    return Err(add_music_info_to_music_list::repository::Error::Unknown(
                        e.to_string(),
                    ));
                }
            },
        };

        Ok(tml_domain::model::app::music_info_music_list::Model {
            music_list_id: inserted.music_list_id,
            music_info_id: inserted.music_info_id,
            order: inserted.order,
        })
    }

    async fn get_music_list_owner_id(
        &self,
        tx_connection: &mut Self::Tx,
        music_list_id: i64,
    ) -> Result<i64, add_music_info_to_music_list::repository::Error> {
        let music_list = music_list::Entity::find_by_id(music_list_id)
            .lock(LockType::Update)
            .one(&tx_connection.tx)
            .await
            .map_err(|e| -> add_music_info_to_music_list::repository::Error {
                add_music_info_to_music_list::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(add_music_info_to_music_list::repository::Error::MusicListNotFound)?;
        Ok(music_list.user_id)
    }
}
