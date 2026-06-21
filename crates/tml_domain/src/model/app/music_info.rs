pub struct Model {
    pub id: Vec<u8>,
    pub artists: Vec<String>,
    pub album_title: String,
    pub title: String,
    pub track_number: i32,
    pub audio_bitrate: i32,
    pub sample_rate: i32,
    pub channels: i16,
    pub bit_depth: i16,
    pub storage_id: i64,
    pub file_path: String,
}
