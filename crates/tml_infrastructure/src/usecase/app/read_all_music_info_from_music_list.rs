use crate::entity::app::{music_info, music_info_music_list, music_list};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter as _, QueryOrder as _, QuerySelect as _};
use tml_application::usecase::app::read_all_music_info_from_music_list;

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
impl read_all_music_info_from_music_list::repository::Trait for Repository {
    async fn read_all_music_info_from_music_list(
        &self,
        music_list_id: i64,
        page_size: u64,
        cursor: Vec<u8>,
    ) -> Result<
        Vec<(
            tml_domain::model::app::music_info_music_list::Model,
            tml_domain::model::app::music_info::Model,
        )>,
        read_all_music_info_from_music_list::repository::Error,
    > {
        let mut select = music_info_music_list::Entity::find()
            .filter(music_info_music_list::Column::MusicListId.eq(music_list_id))
            .order_by_asc(music_info_music_list::Column::Order)
            .limit(page_size);

        // Apply cursor: only fetch items with order > cursor
        if !cursor.is_empty() {
            select = select.filter(music_info_music_list::Column::Order.gt(cursor));
        }

        let result = select
            .find_also_related(music_info::Entity)
            .all(&self.db)
            .await
            .map_err(|e| {
                read_all_music_info_from_music_list::repository::Error::Unknown(e.to_string())
            })?
            .into_iter()
            .map(|(music_list_music_info, music_info)| {
                let music_info = music_info.expect("music_info should exist via foreign key");
                (
                    tml_domain::model::app::music_info_music_list::Model {
                        music_list_id: music_list_music_info.music_list_id,
                        music_info_id: music_list_music_info.music_info_id,
                        order: music_list_music_info.order,
                    },
                    music_info.into(),
                )
            })
            .collect();

        Ok(result)
    }

    async fn get_music_list_owner_id(
        &self,
        music_list_id: i64,
    ) -> Result<i64, read_all_music_info_from_music_list::repository::Error> {
        let music_list = music_list::Entity::find_by_id(music_list_id)
            .one(&self.db)
            .await
            .map_err(|e| {
                read_all_music_info_from_music_list::repository::Error::Unknown(e.to_string())
            })?
            .ok_or(read_all_music_info_from_music_list::repository::Error::MusicListNotFound)?;
        Ok(music_list.user_id)
    }
}
