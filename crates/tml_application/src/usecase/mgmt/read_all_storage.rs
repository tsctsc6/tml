pub mod repository {
    use tml_domain::model::mgmt::storage;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    pub struct PageResult {
        pub items: Vec<storage::Model>,
        pub total: u64,
    }

    #[async_trait::async_trait]
    pub trait Trait {
        async fn read_all_storage(&self, page: u64, page_size: u64) -> Result<PageResult, Error>;
    }
}

pub struct Request {
    pub page_index: u64,
    pub page_size: u64,
}

pub struct StorageItem {
    pub id: i64,
    pub name: String,
    pub path: String,
}

pub struct Response {
    pub total: u64,
    pub items: Vec<StorageItem>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
    #[error("Page size out of range")]
    PageSizeOutOfRange,
}

pub async fn handle(
    request: Request,
    repository: &impl repository::Trait,
) -> Result<Response, Error> {
    if request.page_size == 0 || request.page_size > 1000 {
        return Err(Error::PageSizeOutOfRange);
    }
    let result = repository
        .read_all_storage(request.page_index, request.page_size)
        .await?;
    Ok(Response {
        total: result.total,
        items: result
            .items
            .into_iter()
            .map(|m| StorageItem {
                id: m.id,
                name: m.name,
                path: m.path,
            })
            .collect(),
    })
}
