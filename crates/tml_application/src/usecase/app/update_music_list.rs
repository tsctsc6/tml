pub mod repository {
    use tml_domain::model::app::music_list;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Name duplication")]
        NameDuplication,
        #[error("Music list not found")]
        MusicListNotFound,
        #[error("Permission denied")]
        PermissionDenied,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn update_music_list(
            &self,
            id: i64,
            name: &str,
            user_id: i64,
        ) -> Result<music_list::Model, Error>;
    }
}

pub mod validation {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Name is empty")]
        NameEmpty,
        #[error("Name too long error")]
        NameTooLong,
    }

    pub fn validate(request: &super::Request<'_>) -> Result<(), Error> {
        if request.name.is_empty() {
            return Err(Error::NameEmpty);
        }
        if request.name.chars().count() > 50 {
            return Err(Error::NameTooLong);
        }
        Ok(())
    }
}

pub struct Request<'a> {
    pub id: i64,
    pub name: &'a str,
    pub user_id: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Validation error: {0}")]
    ValidationError(#[from] validation::Error),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
}

pub async fn handle(
    request: Request<'_>,
    repository: &impl repository::Trait,
) -> Result<(), Error> {
    validation::validate(&request)?;
    let _updated_music_list = repository
        .update_music_list(request.id, request.name, request.user_id)
        .await?;
    Ok(())
}
