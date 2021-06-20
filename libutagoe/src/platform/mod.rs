mod youtube;

use async_trait::async_trait;
use std::path::PathBuf;

pub use self::youtube::*;

pub trait MusicSource {
    fn title(&self) -> &str;
}

pub struct MusicDownloadTarget {
    pub audio: PathBuf,
    pub video: PathBuf,
    pub thumbnail: PathBuf,
}

#[async_trait]
pub trait MusicDownload {
    type MusicId;
    type Music: MusicSource;
    type DownloadError;

    fn audio_extension() -> &'static str;
    fn video_extension() -> &'static str;
    fn thumbnail_extension() -> &'static str;

    async fn download(
        id: &Self::MusicId,
        target: MusicDownloadTarget,
    ) -> Result<Self::Music, Self::DownloadError>;
}

pub trait MusicSearch {
    type Query;
    type SearchResult;

    fn search(query: Self::Query) -> Self::SearchResult;
}

pub trait Platform: MusicDownload + MusicSearch {
    fn id() -> &'static str;
}
