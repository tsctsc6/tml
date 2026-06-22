use crate::{
    entity::app::{music_info_music_list, music_list},
    tx_context::SeaOrmTxConnection,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, Order, QueryFilter as _,
    QueryOrder as _,
};
use tml_application::usecase::app::change_music_info_order_in_music_list;

#[derive(Clone)]
pub struct Repository {}

impl Repository {
    pub fn new() -> Self {
        Repository {}
    }
}

#[async_trait::async_trait]
impl change_music_info_order_in_music_list::repository::Trait for Repository {
    type Tx = SeaOrmTxConnection;

    async fn get_music_list_owner_id(
        &self,
        tx_connection: &mut Self::Tx,
        music_list_id: i64,
    ) -> Result<i64, change_music_info_order_in_music_list::repository::Error> {
        let music_list = music_list::Entity::find_by_id(music_list_id)
            .one(&tx_connection.tx)
            .await
            .map_err(|e| {
                change_music_info_order_in_music_list::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(change_music_info_order_in_music_list::repository::Error::MusicListNotFound)?;
        Ok(music_list.user_id)
    }

    async fn get_prev_and_next_order(
        &self,
        tx_connection: &mut Self::Tx,
        music_list_id: i64,
        prev_music_info_id: Option<&[u8]>,
    ) -> Result<
        (Option<Vec<u8>>, Option<Vec<u8>>),
        change_music_info_order_in_music_list::repository::Error,
    > {
        let prev_order = match prev_music_info_id {
            Some(id) => {
                let entry =
                    music_info_music_list::Entity::find_by_id((id.to_owned(), music_list_id))
                        .one(&tx_connection.tx)
                        .await
                        .map_err(|e| {
                            change_music_info_order_in_music_list::repository::Error::Unknown(
                                e.to_string(),
                            )
                        })?
                        .ok_or(change_music_info_order_in_music_list::repository::Error::MusicInfoNotInMusicList)?;
                Some(entry.order)
            }
            None => None,
        };

        // get order of the next item of prev
        let next_order = match &prev_order {
            Some(prev) => {
                // find order > prev min item
                music_info_music_list::Entity::find()
                    .filter(music_info_music_list::Column::MusicListId.eq(music_list_id))
                    .filter(music_info_music_list::Column::Order.gt(prev.as_slice()))
                    .order_by(music_info_music_list::Column::Order, Order::Asc)
                    .one(&tx_connection.tx)
                    .await
                    .map_err(|e| {
                        change_music_info_order_in_music_list::repository::Error::Unknown(
                            e.to_string(),
                        )
                    })?
                    .map(|e| e.order)
            }
            None => {
                // there is no prev, find the min order
                music_info_music_list::Entity::find()
                    .filter(music_info_music_list::Column::MusicListId.eq(music_list_id))
                    .order_by(music_info_music_list::Column::Order, Order::Asc)
                    .one(&tx_connection.tx)
                    .await
                    .map_err(|e| {
                        change_music_info_order_in_music_list::repository::Error::Unknown(
                            e.to_string(),
                        )
                    })?
                    .map(|e| e.order)
            }
        };

        Ok((prev_order, next_order))
    }

    async fn update_order_of_music_info_in_music_list(
        &self,
        tx_connection: &mut Self::Tx,
        music_list_id: i64,
        music_info_id: &[u8],
        new_order: &[u8],
    ) -> Result<(), change_music_info_order_in_music_list::repository::Error> {
        let entry = music_info_music_list::Entity::find_by_id((
            music_info_id.to_owned(),
            music_list_id,
        ))
        .one(&tx_connection.tx)
        .await
        .map_err(|e| {
            change_music_info_order_in_music_list::repository::Error::Unknown(e.to_string())
        })?
        .ok_or(change_music_info_order_in_music_list::repository::Error::MusicInfoNotInMusicList)?;

        let mut active: music_info_music_list::ActiveModel = entry.into();
        active.order = Set(new_order.to_owned());
        active.update(&tx_connection.tx).await.map_err(|e| {
            change_music_info_order_in_music_list::repository::Error::Unknown(e.to_string())
        })?;
        Ok(())
    }
}
