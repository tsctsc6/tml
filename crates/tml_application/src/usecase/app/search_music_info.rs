use crate::app_trait::search_engine::{self, SearchResults};

pub struct Request<'a> {
    pub query: &'a str,
    pub hits_per_page: Option<usize>,
    pub page: Option<usize>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct MusicInfoSearchItem {
    /// hex-encoded, 128 bit
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album_title: String,
}

pub struct Response {
    pub results: SearchResults<MusicInfoSearchItem>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Search engine error: {0}")]
    SearchEngineError(#[from] search_engine::Error),
}

pub async fn handle(
    request: Request<'_>,
    search_engine: &impl search_engine::Trait,
) -> Result<Response, Error> {
    let hits_per_page = request.hits_per_page.unwrap_or(10);
    let page = request.page.unwrap_or(1);
    let results = search_engine
        .search_music_info(request.query, hits_per_page, page)
        .await?;
    Ok(Response { results })
}
