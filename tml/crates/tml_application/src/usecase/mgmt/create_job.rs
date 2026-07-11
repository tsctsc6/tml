use tml_domain::model::mgmt::job;

pub mod repository {
    use tml_domain::model::mgmt::job;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn create_job(
            &self,
            job_type: &job::JobType,
            job_args: &serde_json::Value,
            description: &str,
            created_by_id: i64,
        ) -> Result<job::Model, Error>;
    }
}

pub mod validation {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Description too long")]
        DescriptionTooLong,
    }

    pub fn validate(request: &super::Request<'_>) -> Result<(), Error> {
        if request.description.chars().count() > 200 {
            return Err(Error::DescriptionTooLong);
        }
        Ok(())
    }
}

pub struct Request<'a> {
    pub job_type: &'a job::JobType,
    pub job_args: &'a serde_json::Value,
    pub description: &'a str,
    pub created_by_id: i64,
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
    job_handler: &impl crate::app_trait::job_handler::Trait,
    meilisearch_index_name: &str,
    file_extensions: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<Response, Error> {
    validation::validate(&request)?;
    let new_job = repository
        .create_job(
            request.job_type,
            request.job_args,
            request.description,
            request.created_by_id,
        )
        .await?;
    let job_handler2 = job_handler.clone();
    let job_type = request.job_type.clone();
    let job_args = request.job_args.clone();
    let meilisearch_index_name = meilisearch_index_name.to_string();
    let file_extensions: Vec<_> = file_extensions
        .into_iter()
        .map(|item| item.as_ref().to_string())
        .collect();
    tokio::spawn(async move {
        job_handler2
            .handle(
                new_job.id,
                job_type,
                job_args,
                &meilisearch_index_name,
                file_extensions,
            )
            .await
    });
    Ok(Response { id: new_job.id })
}
