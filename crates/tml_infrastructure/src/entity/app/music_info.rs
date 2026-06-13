use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(schema_name = "app", table_name = "music_info")]
pub struct Model {
    /// 128 bit hash value
    #[sea_orm(primary_key)]
    pub id: Vec<u8>,
    pub artists: Vec<String>,
    pub album_title: String,
    pub title: String,
    pub track_number: i32,
    pub audio_bitrate: i32,
    pub sample_rate: i32,
    pub channels: i16,
    pub bit_depth: i16,
    pub storage_id: i64,
    pub file_path: String,
    #[sea_orm(has_many, via = "music_info_music_list")]
    pub music_lists: HasMany<super::music_list::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for tml_domain::model::app::music_info::Model {
    fn from(model: Model) -> Self {
        tml_domain::model::app::music_info::Model {
            id: model.id,
            artists: model.artists,
            album_title: model.album_title,
            title: model.title,
            track_number: model.track_number,
            audio_bitrate: model.audio_bitrate,
            sample_rate: model.sample_rate,
            channels: model.channels,
            bit_depth: model.bit_depth,
            storage_id: model.storage_id,
            file_path: model.file_path,
        }
    }
}
