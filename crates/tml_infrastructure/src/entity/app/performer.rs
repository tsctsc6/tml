use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(schema_name = "app", table_name = "performer")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
}

impl ActiveModelBehavior for ActiveModel {}
