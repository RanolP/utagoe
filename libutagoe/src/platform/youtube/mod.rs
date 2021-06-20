use std::time::Duration;

use async_trait::async_trait;
use rustube::{url::Url, Video};
use thiserror::Error;

use super::*;
pub struct YouTube;

pub struct YouTubeUploader {
    pub id: String,
    pub name: String,
}

pub struct YouTubeMusic {
    pub title: String,
    pub uploader: YouTubeUploader,
    pub description: String,
    pub length: Duration,
}

impl MusicSource for YouTubeMusic {
    fn title(&self) -> &str {
        &self.title
    }
}

#[derive(Error, Debug)]
pub enum YouTubeMusicDownloadError {
    #[error("Error from rustube")]
    Rustube(rustube::Error),
}

impl From<rustube::Error> for YouTubeMusicDownloadError {
    fn from(err: rustube::Error) -> Self {
        YouTubeMusicDownloadError::Rustube(err)
    }
}

#[async_trait]
impl MusicDownload for YouTube {
    type MusicId = Url;
    type Music = YouTubeMusic;
    type DownloadError = YouTubeMusicDownloadError;

    async fn download(
        id: &Self::MusicId,
        target: MusicDownloadTarget,
    ) -> Result<Self::Music, Self::DownloadError> {
        let video = Video::from_url(id).await?;
        if let Some(audio) = video.best_audio() {
            audio.download_to(target.audio).await?;
        }
        if let Some(video) = video.best_video() {
            video.download_to(target.video).await?;
        }

        let details = video.video_details();

        if let Some(thumbnail) = details.thumbnails.first() {
            // TODO: download `thumbnail.url`
        }

        let music = YouTubeMusic {
            title: details.title.clone(),
            uploader: YouTubeUploader {
                id: details.channel_id.clone(),
                name: details.author.clone(),
            },
            description: details.short_description.clone(),
            length: Duration::from_secs(details.length_seconds),
        };

        Ok(music)
    }

    fn audio_extension() -> &'static str {
        "mp4"
    }

    fn video_extension() -> &'static str {
        "mp4"
    }

    fn thumbnail_extension() -> &'static str {
        ""
    }
}

impl MusicSearch for YouTube {
    type Query = ();
    type SearchResult = ();

    fn search(query: Self::Query) -> Self::SearchResult {
        ()
    }
}

impl Platform for YouTube {
    fn id() -> &'static str {
        "youtube"
    }
}
