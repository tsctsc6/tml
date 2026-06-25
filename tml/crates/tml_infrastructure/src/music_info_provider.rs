use std::{collections::BTreeMap, ffi::OsStr};

use lofty::{
    file::{AudioFile, TaggedFileExt},
    probe::Probe,
};
use path_slash::PathExt;
use serde::Serialize;
use tml_application::app_trait::music_info_provider::MusicInfo;
use walkdir::WalkDir;
use xxhash_rust::xxh3::xxh3_128;

#[derive(Serialize, Clone)]
pub struct MusicInfoHash {
    pub title: String,
    pub artists: Vec<String>,
    pub album_title: String,
    pub track_number: i32,
    pub audio_bitrate: i32,
    pub sample_rate: i32,
    pub channels: i16,
    pub bit_depth: i16,
}

#[derive(Clone)]
pub struct MusicInfoProvider;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Skiped")]
    Skiped,
    #[error("Lofty error: {0}")]
    LoftyError(#[from] lofty::error::LoftyError),
    #[error("Std IO error: {0}")]
    StdIOError(#[from] std::io::Error),
    #[error("Tag error: {0}")]
    TagError(#[from] TagError),
    #[error("Parse int error: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Try from int error: {0}")]
    TryFromIntError(#[from] std::num::TryFromIntError),
    #[error("Json error: {0}")]
    JsonError(#[from] serde_json::error::Error),
    #[error("Strip prefix error: {0}")]
    StripPrefixError(#[from] std::path::StripPrefixError),
}

#[derive(Debug, thiserror::Error)]
pub enum TagError {
    #[error("Get tag error")]
    GetTag,
    #[error("Get track artist error")]
    TrackArtist,
    #[error("Get album title error")]
    AlbumTitle,
    #[error("Get track title error")]
    TrackTitle,
    #[error("Get track number error")]
    TrackNumber,
}

impl MusicInfoProvider {
    fn map_to_music_info(
        &self,
        base_path: &str,
        entry: Result<walkdir::DirEntry, walkdir::Error>,
    ) -> Option<MusicInfo> {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("{}", e);
                return None;
            }
        };
        match self.map_to_music_info_result(base_path, &entry) {
            Ok(m) => {
                return Some(m);
            }
            Err(e) => {
                if let Error::Skiped = e {
                    return None;
                }
                tracing::error!("{} {}", &entry.path().to_string_lossy(), e);
                return None;
            }
        };
    }

    fn map_to_music_info_result(
        &self,
        base_path: &str,
        entry: &walkdir::DirEntry,
    ) -> Result<MusicInfo, Error> {
        let path = entry.path();
        tracing::debug!("Checking: {}", path.to_slash_lossy());
        if path.is_dir() {
            tracing::debug!("Skiped: {}", path.to_slash_lossy());
            Err(Error::Skiped)?;
        }
        let extension = path.extension().and_then(OsStr::to_str);
        if extension != Some("flac") && extension != Some("mp3") {
            tracing::debug!("Skiped: {}", path.to_slash_lossy());
            Err(Error::Skiped)?;
        }
        // base_path is the argument of WalkDir::new(path), so it shoud not failed.
        let relative_path = path.strip_prefix(base_path).unwrap().to_slash_lossy();

        tracing::trace!("Path: {}", entry.path().to_slash_lossy());
        let tagged_file = Probe::open(entry.path())?.guess_file_type()?.read()?;
        let tag = tagged_file.primary_tag().ok_or(TagError::GetTag)?;
        let artist = tag
            .get(lofty::tag::ItemKey::TrackArtist)
            .ok_or(TagError::TrackArtist)?
            .value()
            .text()
            .ok_or(TagError::TrackArtist)?;
        let album_title = tag
            .get(lofty::tag::ItemKey::AlbumTitle)
            .ok_or(TagError::AlbumTitle)?
            .value()
            .text()
            .ok_or(TagError::AlbumTitle)?;
        let title = tag
            .get(lofty::tag::ItemKey::TrackTitle)
            .ok_or(TagError::TrackTitle)?
            .value()
            .text()
            .ok_or(TagError::TrackTitle)?;
        let track_number = tag
            .get(lofty::tag::ItemKey::TrackNumber)
            .ok_or(TagError::TrackNumber)?
            .value()
            .text()
            .ok_or(TagError::TrackNumber)?;
        let properties = tagged_file.properties();
        let sample_rate = properties.sample_rate().unwrap_or(0);
        let channels = properties.channels().unwrap_or(0);
        let bit_depth = properties.bit_depth().unwrap_or(0);
        let audio_bitrate = properties.audio_bitrate().unwrap_or(0);
        let music_info_hash = MusicInfoHash {
            title: title.into(),
            artists: artist.split(" & ").map(|x| x.to_string()).collect(),
            album_title: album_title.into(),
            track_number: track_number.parse()?,
            audio_bitrate: i32::try_from(audio_bitrate)?,
            sample_rate: i32::try_from(sample_rate)?,
            channels: channels.into(),
            bit_depth: bit_depth.into(),
        };
        // order the keies
        let json_value = serde_json::to_value(&music_info_hash)?;
        // json_value is form last line, so it shoud not failed.
        let sorted_map: BTreeMap<String, serde_json::Value> =
            serde_json::from_value(json_value).unwrap();
        let json = serde_json::to_string(&sorted_map)?;
        tracing::trace!("{}", json);
        let hash_128 = xxh3_128(json.as_bytes());
        tracing::trace!("{:0>32X}", hash_128);
        return Ok(MusicInfo {
            id: hash_128.to_be_bytes().to_vec(),
            title: music_info_hash.title,
            artists: music_info_hash.artists,
            album_title: music_info_hash.album_title,
            track_number: music_info_hash.track_number,
            audio_bitrate: music_info_hash.audio_bitrate,
            sample_rate: music_info_hash.sample_rate,
            channels: music_info_hash.channels,
            bit_depth: music_info_hash.bit_depth,
            file_path: relative_path.into(),
        });
    }
}

impl tml_application::app_trait::music_info_provider::Trait for MusicInfoProvider {
    fn scan(&self, path: &str) -> impl Iterator<Item = MusicInfo> + Send {
        return WalkDir::new(path)
            .into_iter()
            .filter_map(|x| self.map_to_music_info(path, x));
    }
}
