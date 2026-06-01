use crate::app_trait;

pub mod repository {
    use tml_domain::model::auth::user;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait {
        async fn is_no_admin(&self) -> Result<bool, Error>;
        async fn create_admin(
            &self,
            username: &str,
            password_hash: &str,
        ) -> Result<user::Model, Error>;
    }
}

pub struct Request {
    pub username: String,
    pub password: String,
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
    request: Request,
    password_hasher: &impl app_trait::password_hasher::Trait,
    repository: &impl repository::Trait,
) -> Result<(), Error> {
    if !repository.is_no_admin().await? {
        return Err(Error::HadAdminError);
    }
    let hashed_password = password_hasher.hash_password(&request.password)?;
    let _new_admin = repository
        .create_admin(&request.username, &hashed_password)
        .await?;
    Ok(())
}
