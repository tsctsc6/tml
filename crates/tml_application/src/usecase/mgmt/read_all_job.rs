pub mod repository {
    use tml_domain::model::mgmt::job;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn read_all_job(
            &self,
            page_size: u64,
            created_after: chrono::DateTime<chrono::Utc>,
            created_before: chrono::DateTime<chrono::Utc>,
        ) -> Result<Vec<job::Model>, Error>;
    }
}

pub struct Request {
    pub cursor: Option<chrono::DateTime<chrono::Utc>>,
    pub page_size: u64,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct JobItem {
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

pub struct Response {
    pub items: Vec<JobItem>,
    pub next_cursor: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
    #[error("Page size out of range")]
    PageSizeOutOfRange,
    #[error("Datetime out of range")]
    DateTimeOutOfRange,
}

pub async fn handle(
    request: Request,
    repository: &impl repository::Trait,
) -> Result<Response, Error> {
    if request.page_size == 0 || request.page_size > 1000 {
        return Err(Error::PageSizeOutOfRange);
    }
    let created_before = request.created_before.unwrap_or(crate::SAFE_MAX_DATETIME);
    let created_after = request.created_after.unwrap_or(crate::SAFE_MIN_DATETIME);
    let created_before = if let Some(cursor) = request.cursor {
        if cursor < created_before {
            cursor
        } else {
            created_before
        }
    } else {
        created_before
    };

    if created_after >= created_before  {
        return Err(Error::DateTimeOutOfRange);
    }

    let result = repository
        .read_all_job(request.page_size, created_after, created_before)
        .await?;

    let items: Vec<JobItem> = result
        .into_iter()
        .map(|m| JobItem {
            id: m.id,
            job_type: m.job_type,
            job_args: m.job_args,
            status: m.status,
            description: m.description,
            error_message: m.error_message,
            success: m.success,
            created_by_id: m.created_by_id,
            created_at: m.created_at,
            completed_at: m.completed_at,
        })
        .collect();
    let next_cursor = items.last().map(|x| x.created_at);

    Ok(Response { items, next_cursor })
}
