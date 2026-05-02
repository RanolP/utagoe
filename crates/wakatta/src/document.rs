use serde::{Deserialize, Serialize};

use crate::identity::ExternalIdentity;
use crate::track::TrackId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackDocument {
    pub track_id: TrackId,
    pub header: TrackDocumentHeader,
    pub body: TrackDocumentBody,
    pub state: TrackDocumentState,
}

impl TrackDocument {
    pub fn new(title: impl Into<String>, artists: Vec<String>) -> Self {
        Self {
            track_id: TrackId(uuid::Uuid::now_v7()),
            header: TrackDocumentHeader {
                title: title.into(),
                artists,
                ..TrackDocumentHeader::default()
            },
            body: TrackDocumentBody::empty_tiptap(),
            state: TrackDocumentState::Draft,
        }
    }

    pub fn attach_identity(&mut self, identity: ExternalIdentity) -> bool {
        if self.header.identities.contains(&identity) {
            return false;
        }

        self.header.identities.push(identity);
        true
    }

    pub fn mark_fragmentary(&mut self) {
        self.state = TrackDocumentState::Fragmentary;
    }

    pub fn from_file_metadata(metadata: TrackFileMetadata) -> Self {
        Self {
            track_id: TrackId(uuid::Uuid::now_v7()),
            header: TrackDocumentHeader::from_file_metadata(metadata),
            body: TrackDocumentBody::empty_tiptap(),
            state: TrackDocumentState::Active,
        }
    }

    pub fn from_provider_metadata(metadata: ProviderExtractedMetadata) -> Self {
        Self {
            track_id: TrackId(uuid::Uuid::now_v7()),
            header: TrackDocumentHeader::from_provider_metadata(metadata),
            body: TrackDocumentBody::empty_tiptap(),
            state: TrackDocumentState::Active,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackDocumentHeader {
    pub title: String,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub release_date: Option<String>,
    pub artwork_uri: Option<String>,
    pub duration_ms: Option<u64>,
    pub language: Option<String>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub identities: Vec<ExternalIdentity>,
    pub metadata_origin: MetadataOrigin,
}

impl TrackDocumentHeader {
    pub fn from_file_metadata(metadata: TrackFileMetadata) -> Self {
        Self {
            title: metadata.title.unwrap_or(metadata.file_stem),
            artists: metadata.artists,
            album: metadata.album,
            album_artist: metadata.album_artist,
            release_date: metadata.release_date,
            artwork_uri: metadata.artwork_uri,
            duration_ms: metadata.duration_ms,
            language: None,
            track_number: metadata.track_number,
            disc_number: metadata.disc_number,
            genres: metadata.genres,
            tags: Vec::new(),
            identities: Vec::new(),
            metadata_origin: MetadataOrigin::FileTags,
        }
    }

    pub fn from_provider_metadata(metadata: ProviderExtractedMetadata) -> Self {
        Self {
            title: metadata
                .title
                .unwrap_or_else(|| metadata.source_url.clone()),
            artists: metadata.artists,
            album: metadata.album,
            album_artist: metadata.uploader.or(metadata.channel),
            release_date: metadata.release_date.or(metadata.upload_date),
            artwork_uri: metadata.thumbnail_url,
            duration_ms: metadata.duration_ms,
            language: None,
            track_number: metadata.track_number.or(metadata.playlist_index),
            disc_number: None,
            genres: metadata.categories,
            tags: metadata.tags,
            identities: Vec::new(),
            metadata_origin: MetadataOrigin::Provider,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackFileMetadata {
    pub file_stem: String,
    pub title: Option<String>,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub release_date: Option<String>,
    pub artwork_uri: Option<String>,
    pub duration_ms: Option<u64>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,
    pub genres: Vec<String>,
}

impl TrackFileMetadata {
    pub fn from_file_stem(file_stem: impl Into<String>) -> Self {
        Self {
            file_stem: file_stem.into(),
            title: None,
            artists: Vec::new(),
            album: None,
            album_artist: None,
            release_date: None,
            artwork_uri: None,
            duration_ms: None,
            track_number: None,
            disc_number: None,
            genres: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderExtractedMetadata {
    pub source_url: String,
    pub webpage_url: Option<String>,
    pub original_url: Option<String>,
    pub extractor: Option<String>,
    pub extractor_key: Option<String>,
    pub title: Option<String>,
    pub uploader: Option<String>,
    pub channel: Option<String>,
    pub artists: Vec<String>,
    pub album: Option<String>,
    pub track: Option<String>,
    pub release_date: Option<String>,
    pub upload_date: Option<String>,
    pub duration_ms: Option<u64>,
    pub thumbnail_url: Option<String>,
    pub playlist: Option<String>,
    pub playlist_index: Option<u32>,
    pub track_number: Option<u32>,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub description: Option<String>,
}

impl ProviderExtractedMetadata {
    pub fn from_source_url(source_url: impl Into<String>) -> Self {
        Self {
            source_url: source_url.into(),
            webpage_url: None,
            original_url: None,
            extractor: None,
            extractor_key: None,
            title: None,
            uploader: None,
            channel: None,
            artists: Vec::new(),
            album: None,
            track: None,
            release_date: None,
            upload_date: None,
            duration_ms: None,
            thumbnail_url: None,
            playlist: None,
            playlist_index: None,
            track_number: None,
            tags: Vec::new(),
            categories: Vec::new(),
            description: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetadataOrigin {
    #[default]
    Manual,
    FileTags,
    Provider,
    Generated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackDocumentBody {
    pub format: TrackDocumentBodyFormat,
    pub content: String,
}

impl TrackDocumentBody {
    pub fn empty_tiptap() -> Self {
        Self {
            format: TrackDocumentBodyFormat::TiptapJson,
            content: r#"{"type":"doc","content":[]}"#.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackDocumentBodyFormat {
    TiptapJson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackDocumentState {
    Draft,
    Active,
    NeedsReview,
    Fragmentary,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentViewState {
    Expanded,
    Minimized,
    Focused,
    Background,
    Dirty,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::ProviderId;

    #[test]
    fn track_document_stores_common_metadata_in_header() {
        let document = TrackDocument::new("Plastic Love", vec!["Mariya Takeuchi".to_owned()]);

        assert_eq!(document.header.title, "Plastic Love");
        assert_eq!(document.header.artists, ["Mariya Takeuchi"]);
        assert_eq!(document.header.metadata_origin, MetadataOrigin::Manual);
        assert_eq!(document.body.format, TrackDocumentBodyFormat::TiptapJson);
        assert_eq!(document.state, TrackDocumentState::Draft);
    }

    #[test]
    fn track_document_can_be_created_from_file_tags() {
        let metadata = TrackFileMetadata {
            file_stem: "01 Plastic Love".to_owned(),
            title: Some("Plastic Love".to_owned()),
            artists: vec!["Mariya Takeuchi".to_owned()],
            album: Some("Variety".to_owned()),
            album_artist: Some("Mariya Takeuchi".to_owned()),
            release_date: Some("1984".to_owned()),
            artwork_uri: None,
            duration_ms: Some(295_000),
            track_number: Some(1),
            disc_number: Some(1),
            genres: vec!["City Pop".to_owned()],
        };

        let document = TrackDocument::from_file_metadata(metadata);

        assert_eq!(document.header.title, "Plastic Love");
        assert_eq!(document.header.album, Some("Variety".to_owned()));
        assert_eq!(document.header.track_number, Some(1));
        assert_eq!(document.header.genres, ["City Pop"]);
        assert_eq!(document.header.metadata_origin, MetadataOrigin::FileTags);
        assert_eq!(document.state, TrackDocumentState::Active);
    }

    #[test]
    fn file_stem_is_used_when_file_tag_title_is_missing() {
        let metadata = TrackFileMetadata::from_file_stem("untitled-recording");

        let document = TrackDocument::from_file_metadata(metadata);

        assert_eq!(document.header.title, "untitled-recording");
        assert_eq!(document.header.metadata_origin, MetadataOrigin::FileTags);
    }

    #[test]
    fn track_document_can_be_grounded_from_provider_extraction() {
        let mut metadata =
            ProviderExtractedMetadata::from_source_url("https://music.youtube.com/watch?v=abc");
        metadata.title = Some("Provider Song".to_owned());
        metadata.channel = Some("Official Channel".to_owned());
        metadata.album = Some("Provider Album".to_owned());
        metadata.upload_date = Some("20240101".to_owned());
        metadata.duration_ms = Some(180_000);
        metadata.thumbnail_url = Some("https://example.test/thumb.jpg".to_owned());
        metadata.playlist_index = Some(7);
        metadata.tags = vec!["live".to_owned()];
        metadata.categories = vec!["Music".to_owned()];

        let document = TrackDocument::from_provider_metadata(metadata);

        assert_eq!(document.header.title, "Provider Song");
        assert_eq!(
            document.header.album_artist,
            Some("Official Channel".to_owned())
        );
        assert_eq!(document.header.album, Some("Provider Album".to_owned()));
        assert_eq!(document.header.release_date, Some("20240101".to_owned()));
        assert_eq!(document.header.track_number, Some(7));
        assert_eq!(document.header.tags, ["live"]);
        assert_eq!(document.header.genres, ["Music"]);
        assert_eq!(document.header.metadata_origin, MetadataOrigin::Provider);
    }

    #[test]
    fn provider_source_url_is_fallback_title() {
        let metadata =
            ProviderExtractedMetadata::from_source_url("https://soundcloud.com/example/track");

        let document = TrackDocument::from_provider_metadata(metadata);

        assert_eq!(
            document.header.title,
            "https://soundcloud.com/example/track"
        );
        assert_eq!(document.header.metadata_origin, MetadataOrigin::Provider);
    }

    #[test]
    fn provider_identity_attach_is_idempotent() {
        let mut document = TrackDocument::new("Song", vec![]);
        let identity = ExternalIdentity {
            provider: ProviderId("ytm".to_owned()),
            external_id: "abc".to_owned(),
            url: Some("https://music.youtube.com/watch?v=abc".to_owned()),
        };

        assert!(document.attach_identity(identity.clone()));
        assert!(!document.attach_identity(identity));
        assert_eq!(document.header.identities.len(), 1);
    }

    #[test]
    fn fragmentary_document_state_can_feed_home() {
        let mut document = TrackDocument::new("Loose note", vec![]);

        document.mark_fragmentary();

        assert_eq!(document.state, TrackDocumentState::Fragmentary);
    }
}
