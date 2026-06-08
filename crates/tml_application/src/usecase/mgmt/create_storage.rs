pub mod repository {
    use tml_domain::model::mgmt::storage;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Name duplication")]
        NameDuplication,
        #[error("Path duplication")]
        PathDuplication,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn create_storage(&self, name: &str, path: &str) -> Result<storage::Model, Error>;
    }
}

pub mod validation {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Name too long error")]
        NameTooLong,
        #[error("Invalid path error, \\ / : * ? \"  < > | is not allowed")]
        InvalidPath,
    }

    pub fn validate(request: &super::Request<'_>) -> Result<(), Error> {
        if request.name.chars().count() > 50 {
            return Err(Error::NameTooLong);
        }
        if request
            .path
            .split("/")
            .any(|x| x.contains(['\\', '/', ':', '*', '?', '\"', '<', '>', '|']))
        {
            return Err(Error::InvalidPath);
        }
        Ok(())
    }
}

pub struct Request<'a> {
    pub name: &'a str,
    pub path: &'a str,
}

pub struct Response {
    pub id: i64,
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
) -> Result<Response, Error> {
    validation::validate(&request)?;
    let new_storage = repository
        .create_storage(request.name, request.path)
        .await?;
    Ok(Response { id: new_storage.id })
}
