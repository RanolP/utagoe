use std::collections::HashMap;

use wakatta::{ProviderExtractedMetadata, ProviderId, TrackDocument, TrackId};

use crate::{
    AppEvent, AppSnapshot, CoreId, LocalAsset, LocalAssetId, PlayPreparationJob, PlaybackContext,
    ProviderDescriptor, ProviderSessionState, ShadowSyncState, TrackHandle, TrackRole,
};

#[derive(Debug, Default)]
pub(crate) struct UtagoeState {
    active_track: Option<TrackHandle>,
    active_document: Option<TrackId>,
    active_asset: Option<LocalAssetId>,
    playback_context: Option<PlaybackContext>,
    documents: HashMap<TrackId, TrackDocument>,
    local_assets: HashMap<LocalAssetId, LocalAsset>,
    play_preparations: HashMap<CoreId, PlayPreparationJob>,
    shadow_sync: HashMap<ProviderId, ShadowSyncState>,
    provider_sessions: HashMap<ProviderId, ProviderSessionState>,
    providers: Vec<ProviderDescriptor>,
    events: Vec<AppEvent>,
}

impl UtagoeState {
    pub fn snapshot(&self) -> AppSnapshot {
        AppSnapshot {
            active_track: self.active_track.clone(),
            active_document: self
                .active_document
                .and_then(|track_id| self.documents.get(&track_id).cloned().map(Box::new)),
            active_asset: self.active_asset,
            playback_context: self.playback_context.clone(),
            play_preparations: self.play_preparations.values().cloned().collect(),
            shadow_sync: self.shadow_sync.values().cloned().collect(),
            providers: self.providers.clone(),
        }
    }

    pub fn drain_events(&mut self) -> Vec<AppEvent> {
        self.events.drain(..).collect()
    }

    pub fn add_local_asset(&mut self, asset: LocalAsset) -> Result<(), UtagoeError> {
        if self.local_assets.contains_key(&asset.id) {
            return Err(UtagoeError::DuplicateLocalAsset(asset.id));
        }

        let id = asset.id;
        let track_id = asset.track_id;
        self.local_assets.insert(id, asset.clone());

        if self.active_track.is_none() {
            self.active_track = Some(TrackHandle {
                track_id,
                role: TrackRole::LocalOnly,
            });
            self.active_asset = Some(id);
        }

        self.events.push(AppEvent::LocalAssetAdded(asset));
        self.push_snapshot();

        Ok(())
    }

    pub fn activate_track(
        &mut self,
        track_id: TrackId,
        role: TrackRole,
    ) -> Result<(), UtagoeError> {
        let active_asset = self
            .local_assets
            .values()
            .find(|asset| asset.track_id == track_id)
            .map(|asset| asset.id);

        if active_asset.is_none() && matches!(role, TrackRole::LocalOnly | TrackRole::Canonical) {
            return Err(UtagoeError::NoLocalAssetForTrack(track_id));
        }

        self.active_track = Some(TrackHandle { track_id, role });
        self.active_asset = active_asset;
        self.push_snapshot();

        Ok(())
    }

    pub fn update_shadow_sync(&mut self, sync: ShadowSyncState) {
        let provider = sync.provider.clone();
        self.shadow_sync.insert(provider, sync.clone());
        self.events.push(AppEvent::ShadowSyncChanged(sync));
        self.push_snapshot();
    }

    pub fn upsert_play_preparation(&mut self, job: PlayPreparationJob) {
        self.play_preparations.insert(job.id, job.clone());
        self.events.push(AppEvent::PlayPreparationChanged(job));
        self.push_snapshot();
    }

    pub fn persist_provider_track(&mut self, metadata: ProviderExtractedMetadata) -> TrackDocument {
        let document = TrackDocument::from_provider_metadata(metadata);
        let track_id = document.track_id;

        self.documents.insert(track_id, document.clone());
        self.active_document = Some(track_id);
        self.active_track = Some(TrackHandle {
            track_id,
            role: TrackRole::ExternalOnly,
        });
        self.events
            .push(AppEvent::DocumentPersisted(Box::new(document.clone())));
        self.push_snapshot();

        document
    }

    pub fn set_provider_attention(&mut self, provider: ProviderId, detail: String) {
        self.provider_sessions
            .insert(provider.clone(), ProviderSessionState::SessionExpired);
        self.events
            .push(AppEvent::ProviderAttention { provider, detail });
        self.push_snapshot();
    }

    fn push_snapshot(&mut self) {
        self.events.push(AppEvent::SnapshotChanged(self.snapshot()));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UtagoeError {
    #[error("local asset already exists: {0:?}")]
    DuplicateLocalAsset(LocalAssetId),
    #[error("track has no local asset: {0:?}")]
    NoLocalAssetForTrack(TrackId),
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use uuid::Uuid;
    use wakatta::{ProviderExtractedMetadata, ProviderId, TrackId};

    use super::*;
    use crate::{AcquisitionSource, PlayPreparationJob, ProviderIdentity, TrackRole};

    fn track_id() -> TrackId {
        TrackId(Uuid::now_v7())
    }

    fn local_asset(track_id: TrackId) -> LocalAsset {
        LocalAsset {
            id: LocalAssetId::new(),
            track_id,
            path: PathBuf::from("song.flac"),
            source: AcquisitionSource::UserImport,
        }
    }

    fn identity(provider: ProviderId) -> ProviderIdentity {
        ProviderIdentity {
            provider,
            external_id: "external-track".to_owned(),
            url: None,
        }
    }

    #[test]
    fn first_local_asset_becomes_active_track_and_asset() {
        let mut state = UtagoeState::default();
        let track_id = track_id();
        let asset = local_asset(track_id);
        let asset_id = asset.id;

        state.add_local_asset(asset.clone()).expect("asset insert");

        let snapshot = state.snapshot();
        assert_eq!(
            snapshot.active_track,
            Some(TrackHandle {
                track_id,
                role: TrackRole::LocalOnly
            })
        );
        assert_eq!(snapshot.active_asset, Some(asset_id));
        assert_eq!(
            state.drain_events(),
            vec![
                AppEvent::LocalAssetAdded(asset),
                AppEvent::SnapshotChanged(snapshot),
            ]
        );
    }

    #[test]
    fn duplicate_local_asset_is_rejected_without_event() {
        let mut state = UtagoeState::default();
        let asset = local_asset(track_id());

        state.add_local_asset(asset.clone()).expect("first insert");
        state.drain_events();

        let err = state
            .add_local_asset(asset)
            .expect_err("duplicate asset must fail");

        assert!(matches!(err, UtagoeError::DuplicateLocalAsset(_)));
        assert!(state.drain_events().is_empty());
    }

    #[test]
    fn activate_track_requires_local_asset_for_local_roles() {
        let mut state = UtagoeState::default();
        let missing = track_id();

        let err = state
            .activate_track(missing, TrackRole::LocalOnly)
            .expect_err("missing local asset must fail");

        assert_eq!(err, UtagoeError::NoLocalAssetForTrack(missing));
        assert!(state.snapshot().active_track.is_none());
    }

    #[test]
    fn shadow_sync_update_replaces_provider_state_and_emits_events() {
        let mut state = UtagoeState::default();
        let provider = ProviderId("ytm".to_owned());
        let track_id = track_id();
        let sync = ShadowSyncState {
            provider: provider.clone(),
            track_id,
            identity: identity(provider),
            provider_position: Some(Duration::from_secs(30)),
            drift: Some(Duration::from_secs(1)),
            muted: true,
            session_state: ProviderSessionState::SignedIn,
            issue: None,
        };

        state.update_shadow_sync(sync.clone());

        let snapshot = state.snapshot();
        assert_eq!(snapshot.shadow_sync, vec![sync.clone()]);
        assert_eq!(
            state.drain_events(),
            vec![
                AppEvent::ShadowSyncChanged(sync),
                AppEvent::SnapshotChanged(snapshot),
            ]
        );
    }

    #[test]
    fn play_preparation_update_is_visible_in_snapshot() {
        let mut state = UtagoeState::default();
        let track_id = track_id();
        let job =
            PlayPreparationJob::new(track_id, None, Some("https://example.test/song".to_owned()));

        state.upsert_play_preparation(job.clone());

        let snapshot = state.snapshot();
        assert_eq!(snapshot.play_preparations, vec![job.clone()]);
        assert_eq!(
            state.drain_events(),
            vec![
                AppEvent::PlayPreparationChanged(job),
                AppEvent::SnapshotChanged(snapshot),
            ]
        );
    }

    #[test]
    fn durable_provider_action_persists_track_document_and_activates_it() {
        let mut state = UtagoeState::default();
        let mut metadata =
            ProviderExtractedMetadata::from_source_url("https://music.youtube.com/watch?v=abc");
        metadata.title = Some("Provider Song".to_owned());

        let document = state.persist_provider_track(metadata);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.active_document.as_deref(), Some(&document));
        assert_eq!(
            snapshot.active_track,
            Some(TrackHandle {
                track_id: document.track_id,
                role: TrackRole::ExternalOnly,
            })
        );
        assert_eq!(
            state.drain_events(),
            vec![
                AppEvent::DocumentPersisted(Box::new(document)),
                AppEvent::SnapshotChanged(snapshot),
            ]
        );
    }

    #[test]
    fn provider_attention_emits_attention_event_and_snapshot() {
        let mut state = UtagoeState::default();
        let provider = ProviderId("soundcloud".to_owned());

        state.set_provider_attention(provider.clone(), "login required".to_owned());

        assert_eq!(
            state.drain_events(),
            vec![
                AppEvent::ProviderAttention {
                    provider,
                    detail: "login required".to_owned(),
                },
                AppEvent::SnapshotChanged(state.snapshot()),
            ]
        );
    }
}
