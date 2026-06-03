use crate::app_trait;

pub mod repository {
    use async_trait::async_trait;
    use tml_domain::model::auth::user;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Unique index conflict: {0}")]
        UniqueIndex(String),
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait]
    pub trait Trait {
        async fn create_user(
            &self,
            username: &str,
            password_hash: &str,
        ) -> Result<user::Model, Error>;
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
    let hashed_password = password_hasher.hash_password(request.password)?;
    let new_user = repository
        .create_user(request.username, &hashed_password)
        .await?;
    Ok(Response { id: new_user.id })
}
