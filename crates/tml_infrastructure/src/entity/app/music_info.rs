use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(schema_name = "app", table_name = "music_info")]
pub struct Model {
    /// 128 bit hash value
    #[sea_orm(primary_key)]
    pub id: Vec<u8>,
    pub artists: Vec<String>,
    pub album: String,
    pub title: String,
    pub track_number: i32,
    pub storage_id: i64,
    pub file_path: String,
    #[sea_orm(has_many, via = "music_info_music_list")]
    pub music_lists: HasMany<super::music_list::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
