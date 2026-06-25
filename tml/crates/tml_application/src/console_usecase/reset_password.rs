use crate::app_trait;

pub mod repository {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("User not found")]
        UserNotFound,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn set_password_hash(&self, username: &str, password_hash: &str)
        -> Result<(), Error>;
    }
}

pub struct Request<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
    #[error("Password Hasher error: {0}")]
    PasswordHasherError(#[from] app_trait::password_hasher::Error),
    #[error("There is already an admin")]
    HadAdminError,
}

pub async fn handle(
    request: Request<'_>,
    password_hasher: &impl app_trait::password_hasher::Trait,
    repository: &impl repository::Trait,
) -> Result<(), Error> {
    let hashed_password = password_hasher.hash_password(request.password)?;
    repository
        .set_password_hash(request.username, &hashed_password)
        .await?;
    Ok(())
}
