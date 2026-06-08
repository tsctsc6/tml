pub struct MusicInfo {
    pub title: String,
    pub artists: Vec<String>,
    pub album_title: String,
    pub track_number: i32,
    pub audio_bitrate: i32,
    pub sample_rate: i32,
    pub channels: i16,
    pub bit_depth: i16,
}

pub trait Trait: Send + Sync + Clone + 'static {
    fn scan(&self, path: &str) -> impl Iterator<Item = MusicInfo>;
}
