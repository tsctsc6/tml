use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(schema_name = "app", table_name = "music_list")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    #[sea_orm(has_many, via = "music_info_music_list")]
    pub music_info: HasMany<super::music_info::Entity>,
    pub user_id: i64,
    #[sea_orm(belongs_to, from = "user_id", to = "id")]
    pub user: HasOne<super::super::auth::user::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for tml_domain::model::app::music_list::Model {
    fn from(model: Model) -> Self {
        tml_domain::model::app::music_list::Model {
            id: model.id,
            name: model.name,
            user_id: model.user_id,
        }
    }
}
