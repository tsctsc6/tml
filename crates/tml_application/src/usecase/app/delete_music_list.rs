pub mod repository {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Music list not found")]
        MusicListNotFound,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn delete_music_list(&self, id: i64) -> Result<(), Error>;
        async fn get_music_list_owner_id(&self, music_list_id: i64) -> Result<i64, Error>;
    }
}

pub struct Request {
    pub id: i64,
    pub user_id: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
    #[error("Permission denied")]
    PermissionDenied,
}

pub async fn handle(request: Request, repository: &impl repository::Trait) -> Result<(), Error> {
    let owner_id = repository.get_music_list_owner_id(request.id).await?;
    if request.user_id != owner_id {
        return Err(Error::PermissionDenied);
    }
    repository.delete_music_list(request.id).await?;
    Ok(())
}
