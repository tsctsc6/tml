pub mod repository {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Storage not found")]
        StorageNotFound,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait {
        async fn delete_storage(&self, id: i64) -> Result<(), Error>;
    }
}

pub struct Request {
    pub id: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
}

pub async fn handle(
    request: Request,
    repository: &impl repository::Trait,
) -> Result<(), Error> {
    repository.delete_storage(request.id).await?;
    Ok(())
}
