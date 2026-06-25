use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(schema_name = "auth", table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub username: String,
    pub password_hash: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub security_stamp: uuid::Uuid,
    #[sea_orm(has_many, via = "user_role")]
    pub roles: HasMany<super::role::Entity>,
    #[sea_orm(has_many)]
    pub jobs: HasMany<super::super::mgmt::job::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for tml_domain::model::auth::user::Model {
    fn from(model: Model) -> Self {
        tml_domain::model::auth::user::Model {
            id: model.id,
            username: model.username,
            password_hash: model.password_hash,
            enabled: model.enabled,
            created_at: model.created_at,
            security_stamp: model.security_stamp,
        }
    }
}
