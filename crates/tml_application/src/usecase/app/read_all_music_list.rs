pub mod repository {
    use tml_domain::model::app::music_list;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn read_all_music_list(
            &self,
            page_size: u64,
            cursor: i64,
            user_id: i64,
        ) -> Result<Vec<music_list::Model>, Error>;
    }
}

pub struct Request {
    pub cursor: Option<i64>,
    pub page_size: Option<u64>,
    pub user_id: i64,
}

pub struct MusicListItem {
    pub id: i64,
    pub name: String,
}

pub struct Response {
    pub items: Vec<MusicListItem>,
    pub next_cursor: Option<i64>,
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
    let page_size = request.page_size.unwrap_or(10);
    if page_size == 0 || page_size > 1000 {
        return Err(Error::PageSizeOutOfRange);
    }
    let cursor = request.cursor.unwrap_or(i64::MAX);

    let result = repository
        .read_all_music_list(page_size, cursor, request.user_id)
        .await?;

    let items: Vec<MusicListItem> = result
        .into_iter()
        .map(|m| MusicListItem {
            id: m.id,
            name: m.name,
        })
        .collect();
    let next_cursor = items.last().map(|x| x.id);

    Ok(Response { items, next_cursor })
}
