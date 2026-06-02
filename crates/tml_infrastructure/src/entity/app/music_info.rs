use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(schema_name = "app", table_name = "music_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    #[sea_orm(has_many, via = "music_info_performer")]
    pub performers: HasMany<super::performer::Entity>,
    pub album_id: i64,
    #[sea_orm(belongs_to, from = "album_id", to = "id")]
    pub album: HasOne<super::album::Entity>,
    pub album_index: i32,
    pub file_path: String,
    pub storage_id: i64,
    #[sea_orm(has_many, via = "music_info_music_list")]
    pub music_lists: HasMany<super::music_list::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
