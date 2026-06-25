use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(schema_name = "auth", table_name = "role")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    /// kebab-lower-case
    #[sea_orm(unique)]
    pub name: String,
    #[sea_orm(has_many, via = "user_role")]
    pub users: HasMany<super::user::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for tml_domain::model::auth::role::Model {
    fn from(model: Model) -> Self {
        tml_domain::model::auth::role::Model {
            id: model.id,
            name: model.name,
        }
    }
}
