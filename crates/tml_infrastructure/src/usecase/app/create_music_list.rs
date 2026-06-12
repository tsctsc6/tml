use crate::entity::app::music_list;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, SqlErr};
use tml_application::usecase::app::create_music_list;

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
impl create_music_list::repository::Trait for Repository {
    async fn create_music_list(
        &self,
        name: &str,
        user_id: i64,
    ) -> Result<tml_domain::model::app::music_list::Model, create_music_list::repository::Error>
    {
        let music_list_to_create = music_list::ActiveModel {
            name: Set(name.into()),
            user_id: Set(user_id),
            ..Default::default()
        };
        let new_music_list = match music_list_to_create.insert(&self.db).await {
            Ok(music_list) => music_list,
            Err(e) => match e.sql_err() {
                Some(SqlErr::UniqueConstraintViolation(detail))
                    if detail.contains("music_list_name_key") =>
                {
                    return Err(create_music_list::repository::Error::NameDuplication);
                }
                _ => {
                    return Err(create_music_list::repository::Error::Unknown(e.to_string()));
                }
            },
        };
        Ok(new_music_list.into())
    }
}
