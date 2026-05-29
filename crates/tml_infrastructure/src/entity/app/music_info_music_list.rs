use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(schema_name = "app", table_name = "music_info_music_list")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub music_info_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub music_list_id: i64,
    #[sea_orm(belongs_to, from = "music_info_id", to = "id")]
    pub music_info: Option<super::music_info::Entity>,
    #[sea_orm(belongs_to, from = "music_list_id", to = "id")]
    pub music_list: Option<super::music_list::Entity>,
    pub order: String,
}

impl ActiveModelBehavior for ActiveModel {}
