pub mod repository {
    use tml_domain::model::mgmt::job;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Job not found")]
        JobNotFound,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn read_job(&self, id: i64) -> Result<job::Model, Error>;
    }
}

pub struct Request {
    pub id: i64,
}

pub struct Response {
    pub id: i64,
    pub job_type: tml_domain::model::mgmt::job::JobType,
    pub job_args: serde_json::Value,
    pub status: tml_domain::model::mgmt::job::JobStatus,
    pub description: String,
    pub error_message: String,
    pub success: bool,
    pub created_by_id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
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
    let job = repository.read_job(request.id).await?;
    Ok(Response {
        id: job.id,
        job_type: job.job_type,
        job_args: job.job_args,
        status: job.status,
        description: job.description,
        error_message: job.error_message,
        success: job.success,
        created_by_id: job.created_by_id,
        created_at: job.created_at,
        completed_at: job.completed_at,
    })
}
