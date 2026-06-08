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
    music_info_provider: &impl crate::app_trait::music_info_provider::Trait,
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
    let repository2 = repository.clone();
    let music_info_provider2 = music_info_provider.clone();
    let _x = match new_job.job_type {
        job::JobType::Undefined => tokio::spawn(async {}),
        job::JobType::ScanIncremental => tokio::spawn(async move {
            handle_scan_incremental_job("", repository2, music_info_provider2).await;
        }),
        job::JobType::BuildIndex => tokio::spawn(async {}),
        job::JobType::UpdateIndex => tokio::spawn(async {}),
    };
    Ok(Response { id: new_job.id })
}

async fn handle_scan_incremental_job(
    path: &str,
    repository: impl repository::Trait,
    music_info_provider: impl crate::app_trait::music_info_provider::Trait,
) -> () {
    //let itor = music_info_provider.scan(path);
    //let x = music_info_provider.clone();
    //let _res = tokio::task::spawn_blocking(|| handle_job(x)).await.unwrap();
}
