use crate::entity::auth::role;
use crate::entity::auth::user;
use crate::entity::auth::user_role;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use tml_application::usecase::mgmt::read_all_normal_user;

#[derive(Clone)]
pub struct Repository {
    db: sea_orm::DatabaseConnection,
}

impl Repository {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Repository { db }
    }
}

#[async_trait::async_trait]
impl read_all_normal_user::repository::Trait for Repository {
    async fn read_all_normal_user(
        &self,
        page_index: u64,
        page_size: u64,
    ) -> Result<
        read_all_normal_user::repository::PageResult,
        read_all_normal_user::repository::Error,
    > {
        let normal_user_role = role::Entity::find_by_name("normal-user")
            .one(&self.db)
            .await
            .map_err(|e| read_all_normal_user::repository::Error::Unknown(e.to_string()))?
            .ok_or(read_all_normal_user::repository::Error::Unknown(
                "role \"normal-user\" not found".to_string(),
            ))?;

        let normal_user_ids: Vec<i64> = user_role::Entity::find()
            .filter(user_role::Column::RoleId.eq(normal_user_role.id))
            .all(&self.db)
            .await
            .map_err(|e| read_all_normal_user::repository::Error::Unknown(e.to_string()))?
            .into_iter()
            .map(|ur| ur.user_id)
            .collect();

        let paginator = user::Entity::find()
            .filter(user::Column::Id.is_in(normal_user_ids))
            .order_by_asc(user::Column::Id)
            .paginate(&self.db, page_size);

        let total = paginator
            .num_items()
            .await
            .map_err(|e| read_all_normal_user::repository::Error::Unknown(e.to_string()))?;

        let items = paginator
            .fetch_page(page_index)
            .await
            .map_err(|e| read_all_normal_user::repository::Error::Unknown(e.to_string()))?;

        Ok(read_all_normal_user::repository::PageResult {
            items: items.into_iter().map(|m| m.into()).collect(),
            total,
        })
    }
}
