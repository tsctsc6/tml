use crate::entity::app::{music_info_music_list, music_list};
use fractional_index::FractionalIndex;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, Order, QueryFilter as _,
    QueryOrder as _, SqlErr,
};
use tml_application::usecase::app::add_music_info_to_music_list;

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
impl add_music_info_to_music_list::repository::Trait for Repository {
    async fn add_music_info_to_music_list(
        &self,
        music_list_id: i64,
        music_info_id: &[u8],
    ) -> Result<
        tml_domain::model::app::music_info_music_list::Model,
        add_music_info_to_music_list::repository::Error,
    > {
        // 1. Get the last order value for this music_list
        let last_entry = music_info_music_list::Entity::find()
            .filter(music_info_music_list::Column::MusicListId.eq(music_list_id))
            .order_by(music_info_music_list::Column::Order, Order::Desc)
            .one(&self.db)
            .await
            .map_err(|e| add_music_info_to_music_list::repository::Error::Unknown(e.to_string()))?;

        // 2. Generate new order using fractional_index
        let new_order: Vec<u8> = match last_entry {
            Some(last) => {
                let last_index = FractionalIndex::from_bytes(last.order.clone()).map_err(|e| {
                    add_music_info_to_music_list::repository::Error::Unknown(e.to_string())
                })?;
                let new_index = FractionalIndex::new_after(&last_index);
                new_index.as_bytes().to_vec()
            }
            None => {
                let default_index = FractionalIndex::default();
                default_index.as_bytes().to_vec()
            }
        };

        // 3. Insert the new record — database constraints handle existence & uniqueness
        let new_entry = music_info_music_list::ActiveModel {
            music_info_id: Set(music_info_id.to_owned()),
            music_list_id: Set(music_list_id),
            order: Set(new_order),
        };
        let inserted = match new_entry.insert(&self.db).await {
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
        music_list_id: i64,
    ) -> Result<i64, add_music_info_to_music_list::repository::Error> {
        let music_list = music_list::Entity::find_by_id(music_list_id)
            .one(&self.db)
            .await
            .map_err(|e| -> add_music_info_to_music_list::repository::Error {
                add_music_info_to_music_list::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(add_music_info_to_music_list::repository::Error::MusicListNotFound)?;
        Ok(music_list.user_id)
    }
}
