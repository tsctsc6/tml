pub struct MusicInfo {
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub track_number: String,
    pub audio_bitrate: u32,
    pub sample_rate: u32,
    pub channels: u8,
    pub bit_depth: u8,
}

pub struct Chuck {
    pub data: Vec<MusicInfo>,
}

pub trait Trait: Send + Sync + Clone + Iterator {}
