use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(schema_name = "app", table_name = "music_info")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub title: String,
    pub artists: serde_json::Value,
    pub album: String,
    pub track_number: i32,
    pub file_path: String,
    pub storage_id: i64,
    #[sea_orm(has_many, via = "music_info_music_list")]
    pub music_lists: HasMany<super::music_list::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
