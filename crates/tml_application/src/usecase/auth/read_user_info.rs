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
        async fn find_user_by_id(
            &self,
            id: i64,
        ) -> Result<(user::Model, Vec<String>), Error>;
    }
}

pub struct Request {
    pub user_id: i64,
}

pub struct Response {
    pub id: i64,
    pub username: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub roles: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
}

pub async fn handle(
    request: Request,
    repository: &impl repository::Trait,
) -> Result<Response, Error> {
    let (user, roles) = repository.find_user_by_id(request.user_id).await?;
    Ok(Response {
        id: user.id,
        username: user.username,
        enabled: user.enabled,
        created_at: user.created_at,
        roles,
    })
}
