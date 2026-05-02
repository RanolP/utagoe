//! Music knowledge base and relationship graph.
//!
//! `wakatta` models tracks, external identities, annotations, and relationships.
//! It does not know about playback, Tauri, WebViews, or acquisition tools.

pub mod document;
pub mod graph;
pub mod identity;
pub mod track;

pub use document::{
    DocumentViewState, MetadataOrigin, ProviderExtractedMetadata, TrackDocument, TrackDocumentBody,
    TrackDocumentBodyFormat, TrackDocumentHeader, TrackDocumentState, TrackFileMetadata,
};
pub use graph::{Relation, RelationKind, Segment, SegmentError};
pub use identity::{ExternalIdentity, ProviderId};
pub use track::{Track, TrackId};
