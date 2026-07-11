use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct MusicInfo {
    pub id: Vec<u8>,
    pub title: String,
    pub artists: Vec<String>,
    pub album_title: String,
    pub track_number: i32,
    pub audio_bitrate: i32,
    pub sample_rate: i32,
    pub channels: i16,
    pub bit_depth: i16,
    pub file_path: String,
}

#[derive(Serialize, Deserialize)]
pub struct MusicInfoMeiliSearch {
    /// hex-encoded, 128 bit
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub album_title: String,
}

pub trait Trait: Send + Sync + Clone + 'static {
    fn scan(
        &self,
        path: &str,
        file_extensions: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> impl Iterator<Item = MusicInfo> + Send;
}
