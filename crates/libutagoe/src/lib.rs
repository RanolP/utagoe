//! Headless Utagoe application core.
//!
//! `libutagoe` coordinates playback, provider sync, acquisition, and the
//! knowledge base. Tauri commands should call this crate rather than wiring
//! lower-level crates directly into the desktop shell.

pub mod core_types;
mod state;

use saisei::SaiseiHandle;
use state::UtagoeState;
use utagoe_provider_runtime::RuntimeRegistry;
use wakatta::{TrackDocument, TrackId};

pub use core_types::{
    AcquisitionIntent, AcquisitionSource, AlbumEntityRef, AppEvent, AppSnapshot, ChannelEntityRef,
    CoreId, EntityRoute, EntitySurface, EntitySurfaceAction, GlobalSearchResults, InboxItem,
    InboxSeverity, ListeningEvent, ListeningEventKind, LocalAsset, LocalAssetId,
    PlayPreparationJob, PlayPreparationPlan, PlayPreparationState, PlaybackChromeState,
    PlaybackContext, PlaybackContextId, PlaybackContextKind, PlaylistEntityRef, PreparationTool,
    ProviderCapability, ProviderDescriptor, ProviderFollowUpIndicator, ProviderIdentity,
    ProviderPluginState, ProviderSessionState, Queue, QueueBehavior, QueueCursor, QueueItem,
    RankedSearchResult, RepeatMode, SearchQuery, SearchResult, SearchResultKind,
    SearchResultSource, SearchWarning, ShadowSyncState, ShuffleMode, TrackEntityRef, TrackHandle,
    TrackMatch, TrackMatchKind, TrackRole,
};
pub use state::UtagoeError;
pub use wakatta::ProviderExtractedMetadata;

#[derive(Debug)]
pub struct Utagoe {
    playback: SaiseiHandle,
    providers: RuntimeRegistry,
    state: UtagoeState,
}

impl Utagoe {
    pub fn new(playback: SaiseiHandle, providers: RuntimeRegistry) -> Self {
        Self {
            playback,
            providers,
            state: UtagoeState::default(),
        }
    }

    pub fn playback(&self) -> &SaiseiHandle {
        &self.playback
    }

    pub fn providers(&self) -> &RuntimeRegistry {
        &self.providers
    }

    pub fn snapshot(&self) -> AppSnapshot {
        self.state.snapshot()
    }

    pub fn drain_events(&mut self) -> Vec<AppEvent> {
        self.state.drain_events()
    }

    pub fn add_local_asset(&mut self, asset: LocalAsset) -> Result<(), UtagoeError> {
        self.state.add_local_asset(asset)
    }

    pub fn activate_track(
        &mut self,
        track_id: TrackId,
        role: TrackRole,
    ) -> Result<(), UtagoeError> {
        self.state.activate_track(track_id, role)
    }

    pub fn update_shadow_sync(&mut self, sync: ShadowSyncState) {
        self.state.update_shadow_sync(sync);
    }

    pub fn upsert_play_preparation(&mut self, job: PlayPreparationJob) {
        self.state.upsert_play_preparation(job);
    }

    pub fn persist_provider_track(&mut self, metadata: ProviderExtractedMetadata) -> TrackDocument {
        self.state.persist_provider_track(metadata)
    }

    pub fn set_provider_attention(&mut self, provider: wakatta::ProviderId, detail: String) {
        self.state.set_provider_attention(provider, detail);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use saisei::SaiseiHandle;
    use utagoe_provider_runtime::RuntimeRegistry;
    use uuid::Uuid;
    use wakatta::TrackId;

    use super::*;

    #[test]
    fn facade_add_local_asset_updates_snapshot_and_events() {
        let mut app = Utagoe::new(SaiseiHandle::mock(), RuntimeRegistry::default());
        let track_id = TrackId(Uuid::now_v7());
        let asset = LocalAsset {
            id: LocalAssetId::new(),
            track_id,
            path: PathBuf::from("song.flac"),
            source: AcquisitionSource::UserImport,
        };
        let asset_id = asset.id;

        app.add_local_asset(asset.clone()).expect("asset insert");

        assert_eq!(
            app.snapshot().active_track,
            Some(TrackHandle {
                track_id,
                role: TrackRole::LocalOnly
            })
        );
        assert_eq!(app.snapshot().active_asset, Some(asset_id));
        assert_eq!(
            app.drain_events(),
            vec![
                AppEvent::LocalAssetAdded(asset),
                AppEvent::SnapshotChanged(app.snapshot()),
            ]
        );
    }

    #[test]
    fn facade_persists_provider_track_document() {
        let mut app = Utagoe::new(SaiseiHandle::mock(), RuntimeRegistry::default());
        let mut metadata =
            ProviderExtractedMetadata::from_source_url("https://music.youtube.com/watch?v=abc");
        metadata.title = Some("Provider Song".to_owned());

        let document = app.persist_provider_track(metadata);

        assert_eq!(app.snapshot().active_document.as_deref(), Some(&document));
    }
}
