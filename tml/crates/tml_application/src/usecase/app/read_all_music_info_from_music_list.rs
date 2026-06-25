pub mod repository {
    use tml_domain::model::app::{music_info, music_info_music_list};

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("Music list not found")]
        MusicListNotFound,
        #[error("Unknown error: {0}")]
        Unknown(String),
    }

    #[async_trait::async_trait]
    pub trait Trait: Send + Sync + Clone + 'static {
        async fn read_all_music_info_from_music_list(
            &self,
            music_list_id: i64,
            page_size: u64,
            cursor: Vec<u8>,
        ) -> Result<Vec<(music_info_music_list::Model, music_info::Model)>, Error>;
        async fn get_music_list_owner_id(&self, music_list_id: i64) -> Result<i64, Error>;
    }
}

pub struct Request {
    pub music_list_id: i64,
    pub cursor: Option<Vec<u8>>,
    pub page_size: Option<u64>,
    pub user_id: i64,
}

pub struct MusicInfoItem {
    pub music_info_id: Vec<u8>,
    pub order: Vec<u8>,
    pub title: String,
    pub artists: Vec<String>,
    pub album_title: String,
    pub track_number: i32,
    pub audio_bitrate: i32,
    pub sample_rate: i32,
    pub channels: i16,
    pub bit_depth: i16,
    pub storage_id: i64,
    pub file_path: String,
}

pub struct Response {
    pub items: Vec<MusicInfoItem>,
    pub next_cursor: Option<Vec<u8>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] repository::Error),
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Page size out of range")]
    PageSizeOutOfRange,
}

pub async fn handle(
    request: Request,
    repository: &impl repository::Trait,
) -> Result<Response, Error> {
    // Verify the music list exists (and implicitly check permission separately if needed)
    let owner_id = repository
        .get_music_list_owner_id(request.music_list_id)
        .await?;
    if request.user_id != owner_id {
        return Err(Error::PermissionDenied);
    }

    let page_size = request.page_size.unwrap_or(10);
    if page_size == 0 || page_size > 1000 {
        return Err(Error::PageSizeOutOfRange);
    }
    let cursor = request.cursor.unwrap_or_default();

    let result = repository
        .read_all_music_info_from_music_list(request.music_list_id, page_size, cursor)
        .await?;

    let items: Vec<MusicInfoItem> = result
        .into_iter()
        .take(page_size as usize)
        .map(|(junction, info)| MusicInfoItem {
            music_info_id: junction.music_info_id,
            order: junction.order,
            title: info.title,
            artists: info.artists,
            album_title: info.album_title,
            track_number: info.track_number,
            audio_bitrate: info.audio_bitrate,
            sample_rate: info.sample_rate,
            channels: info.channels,
            bit_depth: info.bit_depth,
            storage_id: info.storage_id,
            file_path: info.file_path,
        })
        .collect();

    let next_cursor = items.last().map(|x| x.order.clone());

    Ok(Response { items, next_cursor })
}
