pub mod repository {
    use tml_domain::model::app::music_info_music_list;

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Music list not found")]
        MusicListNotFound,
        #[error("Music info not found")]
        MusicInfoNotFound,
        #[error("Permission denied")]
        PermissionDenied,
        #[error("Music info already in music list")]
        MusicInfoAlreadyInMusicList,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn add_music_info_to_music_list(
            &self,
            music_list_id: i64,
            music_info_id: &[u8],
            user_id: i64,
        ) -> Result<music_info_music_list::Model, Error>;
    }
}

pub struct Request<'a> {
    pub music_list_id: i64,
    pub music_info_id: &'a [u8],
    pub user_id: i64,
}

pub struct Response {
    pub order: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
}

pub async fn handle(
    request: Request<'_>,
    repository: &impl repository::Trait,
) -> Result<Response, Error> {
    let result = repository
        .add_music_info_to_music_list(
            request.music_list_id,
            request.music_info_id,
            request.user_id,
        )
        .await?;
    Ok(Response { order: result.order })
}
