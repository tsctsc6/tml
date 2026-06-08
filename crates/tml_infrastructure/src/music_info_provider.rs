use std::ffi::OsStr;

use lofty::{
    file::{AudioFile, TaggedFileExt},
    probe::Probe,
};
use tml_application::app_trait::music_info_provider::MusicInfo;
use walkdir::WalkDir;

#[derive(Clone)]
pub struct MusicInfoProvider;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Skiped")]
    Skiped,
    #[error("WalkDir error: {0}")]
    WalkDirError(#[from] walkdir::Error),
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
        entry: Result<walkdir::DirEntry, walkdir::Error>,
    ) -> Option<tml_application::app_trait::music_info_provider::MusicInfo> {
        match self.map_to_music_info_result(entry) {
            Ok(m) => {
                return Some(m);
            }
            Err(e) => {
                tracing::warn!("{}", e);
                return None;
            }
        };
    }

    fn map_to_music_info_result(
        &self,
        entry: Result<walkdir::DirEntry, walkdir::Error>,
    ) -> Result<tml_application::app_trait::music_info_provider::MusicInfo, Error> {
        let entry = entry?;
        if entry.path().is_dir() {
            return Err(Error::Skiped);
        }
        let extension = entry.path().extension().and_then(OsStr::to_str);
        if extension != Some("flac") && extension != Some("mp3") {
            return Err(Error::Skiped);
        }
        tracing::trace!("Path: {}", entry.path().display());
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
        let music_info = MusicInfo {
            title: title.into(),
            artists: artist.split(" & ").map(|x| x.to_string()).collect(),
            album_title: album_title.into(),
            track_number: track_number.parse()?,
            audio_bitrate: i32::try_from(audio_bitrate)?,
            sample_rate: i32::try_from(sample_rate)?,
            channels: channels.into(),
            bit_depth: bit_depth.into(),
        };
        return Ok(music_info);
    }
}

impl tml_application::app_trait::music_info_provider::Trait for MusicInfoProvider {
    fn scan(
        &self,
        path: &str,
    ) -> impl Iterator<Item = tml_application::app_trait::music_info_provider::MusicInfo> {
        return WalkDir::new(path)
            .into_iter()
            .filter_map(|x| self.map_to_music_info(x));
    }
}
