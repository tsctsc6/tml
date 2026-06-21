use crate::app_trait;

pub mod repository {
    use tml_domain::model::auth::user;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("User not found")]
        UserNotFound,
        #[error("User is not a normal user")]
        UserNotNormalUser,
        #[error("Username duplication")]
        UsernameDuplication,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn update_normal_user(
            &self,
            id: i64,
            username: Option<&str>,
            password_hash: Option<&str>,
            enabled: Option<bool>,
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
        #[error("No fields to update")]
        NoFieldsToUpdate,
    }

    pub fn validate(request: &super::Request<'_>) -> Result<(), Error> {
        if request.username.is_none() && request.password.is_none() && request.enabled.is_none() {
            return Err(Error::NoFieldsToUpdate);
        }
        if let Some(username) = request.username {
            if username.chars().count() > 50 {
                return Err(Error::UsernameTooLong);
            }
            if username.chars().count() < 1 {
                return Err(Error::UsernameTooShort);
            }
        }
        if let Some(password) = request.password {
            if password.chars().count() < 6 {
                return Err(Error::PasswordTooShort);
            }
        }
        Ok(())
    }
}

pub struct Request<'a> {
    pub id: i64,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
    pub enabled: Option<bool>,
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
) -> Result<(), Error> {
    validation::validate(&request)?;
    let hashed_password = match request.password {
        Some(password) => Some(password_hasher.hash_password(password)?),
        None => None,
    };
    repository
        .update_normal_user(
            request.id,
            request.username,
            hashed_password.as_deref(),
            request.enabled,
        )
        .await?;
    Ok(())
}
