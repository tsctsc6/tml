use crate::app_trait;

pub mod repository {
    use tml_domain::model::auth::user;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Username duplication")]
        UsernameDuplication,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn create_normal_user(
            &self,
            username: &str,
            password_hash: &str,
        ) -> Result<user::Model, Error>;
    }
}

pub mod validation {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Username too long")]
        UsernameTooLong,
        #[error("Username too short")]
        UsernameTooShort,
        #[error("Password too short")]
        PasswordTooShort,
    }

    pub fn validate(request: &super::Request<'_>) -> Result<(), Error> {
        if request.username.chars().count() > 50 {
            return Err(Error::UsernameTooLong);
        }
        if request.username.chars().count() < 1 {
            return Err(Error::UsernameTooShort);
        }
        if request.password.chars().count() < 6 {
            return Err(Error::PasswordTooShort);
        }
        Ok(())
    }
}

pub struct Request<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

pub struct Response {
    pub id: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Validation error: {0}")]
    ValidationError(#[from] validation::Error),
    #[error("Password hashing error: {0}")]
    HashingError(#[from] app_trait::password_hasher::Error),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
}

pub async fn handle(
    request: Request<'_>,
    password_hasher: &impl app_trait::password_hasher::Trait,
    repository: &impl repository::Trait,
) -> Result<Response, Error> {
    validation::validate(&request)?;
    let hashed_password = password_hasher.hash_password(request.password)?;
    let new_user = repository
        .create_normal_user(request.username, &hashed_password)
        .await?;
    Ok(Response { id: new_user.id })
}
