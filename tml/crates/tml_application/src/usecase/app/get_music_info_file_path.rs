pub mod repository {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Music info not found")]
        MusicInfoNotFound,
        #[error("Storage not found")]
        StorageNotFound,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn get_music_info_file_path(&self, music_info_id: Vec<u8>) -> Result<String, Error>;
    }
}

pub struct Request {
    pub music_info_id: Vec<u8>,
}

pub struct Response {
    pub file_path: String,
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
    let file_path = repository
        .get_music_info_file_path(request.music_info_id)
        .await?;
    Ok(Response { file_path })
}
