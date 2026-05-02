//! Local file acquisition boundary.
//!
//! Concrete acquisition tools such as yt-dlp should be wrapped behind this
//! crate so Utagoe can track provenance and job state consistently.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AcquisitionJobId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionRequest {
    pub source_url: String,
    pub target_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedStorage {
    pub root: PathBuf,
}

impl ManagedStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn audio_artifact_path(&self, source: &str, extension: &str) -> PathBuf {
        self.managed_path(ManagedStorageKind::PreparedAudio, source, extension)
    }

    pub fn artwork_cache_path(&self, source: &str, extension: &str) -> PathBuf {
        self.managed_path(ManagedStorageKind::ArtworkCache, source, extension)
    }

    pub fn metadata_cache_path(&self, source: &str) -> PathBuf {
        self.managed_path(ManagedStorageKind::MetadataCache, source, "json")
    }

    fn managed_path(&self, kind: ManagedStorageKind, source: &str, extension: &str) -> PathBuf {
        let hash = fnv1a64(source.as_bytes());
        let hex = format!("{hash:016x}");
        let normalized_extension = extension.trim_start_matches('.');

        self.root
            .join(kind.directory())
            .join(&hex[0..2])
            .join(&hex[2..4])
            .join(format!("{hex}.{normalized_extension}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManagedStorageKind {
    PreparedAudio,
    ArtworkCache,
    MetadataCache,
}

impl ManagedStorageKind {
    fn directory(self) -> &'static str {
        match self {
            Self::PreparedAudio => "prepared-audio",
            Self::ArtworkCache => "artwork-cache",
            Self::MetadataCache => "metadata-cache",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcquisitionStatus {
    Queued,
    Running,
    Completed { path: PathBuf },
    Failed { message: String },
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325;

    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    hash
}

impl AcquisitionStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed { .. } | Self::Failed { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_completed_and_failed_statuses_are_terminal() {
        assert!(!AcquisitionStatus::Queued.is_terminal());
        assert!(!AcquisitionStatus::Running.is_terminal());
        assert!(AcquisitionStatus::Completed {
            path: PathBuf::from("song.flac"),
        }
        .is_terminal());
        assert!(AcquisitionStatus::Failed {
            message: "yt-dlp failed".to_owned(),
        }
        .is_terminal());
    }

    #[test]
    fn managed_storage_uses_stable_hashed_audio_paths() {
        let storage = ManagedStorage::new("utagoe");
        let source = "https://music.youtube.com/watch?v=abc";

        let path = storage.audio_artifact_path(source, ".opus");

        assert_eq!(path, storage.audio_artifact_path(source, "opus"));
        assert!(path.starts_with("utagoe/prepared-audio"));
        assert_eq!(
            path.extension().and_then(|extension| extension.to_str()),
            Some("opus")
        );
    }

    #[test]
    fn managed_storage_separates_artwork_and_metadata_cache() {
        let storage = ManagedStorage::new("utagoe");
        let source = "https://example.test/thumb.jpg";

        let artwork = storage.artwork_cache_path(source, "jpg");
        let metadata = storage.metadata_cache_path(source);

        assert!(artwork.starts_with("utagoe/artwork-cache"));
        assert!(metadata.starts_with("utagoe/metadata-cache"));
        assert_ne!(artwork, metadata);
    }
}
