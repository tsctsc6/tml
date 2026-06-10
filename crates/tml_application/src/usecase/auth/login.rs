use crate::app_trait;

pub mod repository {
    use tml_domain::model::auth::user;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("User not found")]
        UserNotFound,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn find_user_by_username(
            &self,
            username: &str,
        ) -> Result<(user::Model, Vec<String>), Error>;
    }
}

pub struct Request<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

pub struct Response {
    pub token: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Jwt error: {0}")]
    JwtError(#[from] app_trait::jwt_manager::Error),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
    #[error("Password Hasher error: {0}")]
    PasswordHasherError(#[from] app_trait::password_hasher::Error),
    #[error("User disabled")]
    UserDisabled,
}

pub async fn handle(
    request: Request<'_>,
    password_hasher: &impl app_trait::password_hasher::Trait,
    jwt_manager: &impl app_trait::jwt_manager::Trait,
    repository: &impl repository::Trait,
) -> Result<Response, Error> {
    let (user, roles) = repository.find_user_by_username(request.username).await?;
    if !user.enabled {
        return Err(Error::UserDisabled);
    }
    password_hasher.verify_password(request.password, &user.password_hash)?;
    let claims = app_trait::jwt_manager::Claims {
        sub: user.id,
        exp: 0, // exp will be set in create_token method
        roles: roles,
        security_stamp: user.security_stamp,
    };
    let token = jwt_manager.create_token(claims)?;
    Ok(Response { token: Some(token) })
}
