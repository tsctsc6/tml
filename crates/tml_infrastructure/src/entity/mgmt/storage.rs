use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(schema_name = "mgmt", table_name = "storage")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub name: String,
    #[sea_orm(unique)]
    /// use / as path separator
    pub path: String,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for tml_domain::model::mgmt::storage::Model {
    fn from(model: Model) -> Self {
        tml_domain::model::mgmt::storage::Model {
            id: model.id,
            name: model.name,
            path: model.path,
        }
    }
}
