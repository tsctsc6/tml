pub mod repository {
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Music list not found")]
        MusicListNotFound,
        #[error("Music info not in music list")]
        MusicInfoNotInMusicList,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn remove_music_info_from_music_list(
            &self,
            music_list_id: i64,
            music_info_id: &[u8],
        ) -> Result<(), Error>;
        async fn get_music_list_owner_id(&self, music_list_id: i64) -> Result<i64, Error>;
    }
}

pub struct Request<'a> {
    pub music_list_id: i64,
    pub music_info_id: &'a [u8],
    pub user_id: i64,
}

pub struct Response;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
    #[error("Permission denied")]
    PermissionDenied,
}

pub async fn handle(
    request: Request<'_>,
    repository: &impl repository::Trait,
) -> Result<Response, Error> {
    let owner_id = repository
        .get_music_list_owner_id(request.music_list_id)
        .await?;
    if request.user_id != owner_id {
        return Err(Error::PermissionDenied);
    }
    repository
        .remove_music_info_from_music_list(request.music_list_id, request.music_info_id)
        .await?;
    Ok(Response)
}
