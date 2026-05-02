//! Core Utagoe domain types.
//!
//! These types describe the language shared by the app core, UI, provider
//! runtime, acquisition pipeline, playback runtime, and knowledge base.

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use wakatta::{ProviderId, TrackDocument, TrackId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoreId(pub Uuid);

impl CoreId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for CoreId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackHandle {
    pub track_id: TrackId,
    pub role: TrackRole,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackRole {
    Canonical,
    Version,
    LocalOnly,
    ExternalOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderIdentity {
    pub provider: ProviderId,
    pub external_id: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
}

impl SearchQuery {
    pub fn fuzzy(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub kind: SearchResultKind,
    pub provider_identity: Option<ProviderIdentity>,
    pub route: EntityRoute,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankedSearchResult {
    pub result: SearchResult,
    pub source: SearchResultSource,
    pub relevance: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchResultSource {
    LocalKnowledgeBase,
    KnownProviderIdentity,
    RemoteProvider,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchWarning {
    pub provider: Option<ProviderId>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalSearchResults {
    pub results: Vec<RankedSearchResult>,
    pub warnings: Vec<SearchWarning>,
}

impl GlobalSearchResults {
    pub fn aggregate(mut results: Vec<RankedSearchResult>, warnings: Vec<SearchWarning>) -> Self {
        results.sort_by(|left, right| {
            right
                .relevance
                .total_cmp(&left.relevance)
                .then_with(|| left.result.title.cmp(&right.result.title))
        });

        Self { results, warnings }
    }

    pub fn has_partial_provider_failure(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchResultKind {
    Track,
    Artist,
    Channel,
    Album,
    Playlist,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityRoute {
    Track(TrackEntityRef),
    Channel(ChannelEntityRef),
    Album(AlbumEntityRef),
    Playlist(PlaylistEntityRef),
    SearchFallback { query: String },
}

impl EntityRoute {
    pub fn resolve_youtube_music_url(url: &str) -> Self {
        let provider = ProviderId("ytm".to_owned());

        if let Some(video_id) = query_value(url, "v") {
            return Self::Track(TrackEntityRef {
                track_id: None,
                provider_identity: Some(ProviderIdentity {
                    provider,
                    external_id: video_id,
                    url: Some(url.to_owned()),
                }),
                source_url: Some(url.to_owned()),
            });
        }

        if let Some(playlist_id) = query_value(url, "list") {
            return Self::Playlist(PlaylistEntityRef {
                provider: Some(provider),
                external_id: Some(playlist_id),
                source_url: Some(url.to_owned()),
                title: None,
            });
        }

        if let Some(channel_id) = path_after(url, "/channel/").or_else(|| path_after(url, "/@")) {
            return Self::Channel(ChannelEntityRef {
                provider: Some(provider),
                external_id: Some(channel_id),
                source_url: Some(url.to_owned()),
                display_name: None,
            });
        }

        if let Some(browse_id) = path_after(url, "/browse/") {
            let route_id = Some(browse_id.clone());
            let source_url = Some(url.to_owned());

            if browse_id.starts_with("MPRE") {
                return Self::Album(AlbumEntityRef {
                    provider: Some(provider),
                    external_id: route_id,
                    source_url,
                    title: None,
                });
            }

            return Self::Channel(ChannelEntityRef {
                provider: Some(provider),
                external_id: route_id,
                source_url,
                display_name: None,
            });
        }

        Self::SearchFallback {
            query: url.to_owned(),
        }
    }
}

fn query_value(url: &str, key: &str) -> Option<String> {
    let query = url.split_once('?')?.1.split('#').next().unwrap_or_default();

    query.split('&').find_map(|pair| {
        let (candidate_key, value) = pair.split_once('=')?;
        (candidate_key == key && !value.is_empty()).then(|| value.to_owned())
    })
}

fn path_after(url: &str, marker: &str) -> Option<String> {
    let (_, tail) = url.split_once(marker)?;
    let value = tail
        .split(['?', '#', '/'])
        .next()
        .unwrap_or_default()
        .to_owned();

    (!value.is_empty()).then_some(value)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackEntityRef {
    pub track_id: Option<TrackId>,
    pub provider_identity: Option<ProviderIdentity>,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelEntityRef {
    pub provider: Option<ProviderId>,
    pub external_id: Option<String>,
    pub source_url: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlbumEntityRef {
    pub provider: Option<ProviderId>,
    pub external_id: Option<String>,
    pub source_url: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaylistEntityRef {
    pub provider: Option<ProviderId>,
    pub external_id: Option<String>,
    pub source_url: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntitySurface {
    pub title: String,
    pub kind: SearchResultKind,
    pub route: EntityRoute,
    pub actions: Vec<EntitySurfaceAction>,
}

impl EntitySurface {
    pub fn minimal(title: impl Into<String>, kind: SearchResultKind, route: EntityRoute) -> Self {
        Self {
            title: title.into(),
            kind,
            route,
            actions: vec![EntitySurfaceAction::Play, EntitySurfaceAction::Organize],
        }
    }

    pub fn supports_listening_and_organizing(&self) -> bool {
        self.actions.contains(&EntitySurfaceAction::Play)
            && self.actions.contains(&EntitySurfaceAction::Organize)
    }

    pub fn track_screen(title: impl Into<String>, route: EntityRoute) -> Self {
        Self {
            title: title.into(),
            kind: SearchResultKind::Track,
            route,
            actions: vec![
                EntitySurfaceAction::Play,
                EntitySurfaceAction::AddToQueue,
                EntitySurfaceAction::Organize,
            ],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntitySurfaceAction {
    Play,
    AddToQueue,
    Organize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackMatch {
    pub track_id: TrackId,
    pub identity: ProviderIdentity,
    pub kind: TrackMatchKind,
    pub confidence: f32,
    pub duration_delta: Option<Duration>,
}

impl TrackMatch {
    pub fn is_accepted(&self) -> bool {
        !matches!(self.kind, TrackMatchKind::Rejected) && self.confidence > 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackMatchKind {
    Exact,
    LikelySameRecording,
    AlternateVersion,
    RemixOrDerivative,
    UserConfirmed,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocalAssetId(pub Uuid);

impl LocalAssetId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for LocalAssetId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalAsset {
    pub id: LocalAssetId,
    pub track_id: TrackId,
    pub path: PathBuf,
    pub source: AcquisitionSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlaybackContextId(pub Uuid);

impl PlaybackContextId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for PlaybackContextId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackContext {
    pub id: PlaybackContextId,
    pub label: String,
    pub kind: PlaybackContextKind,
    pub queue: Queue,
}

impl PlaybackContext {
    pub fn manual(label: impl Into<String>, items: Vec<QueueItem>) -> Self {
        Self {
            id: PlaybackContextId::new(),
            label: label.into(),
            kind: PlaybackContextKind::Manual,
            queue: Queue::finite(items),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackContextKind {
    Manual,
    ProviderCollection { provider: ProviderId },
    Album { provider: Option<ProviderId> },
    Playlist { provider: Option<ProviderId> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Queue {
    pub behavior: QueueBehavior,
    pub items: Vec<QueueItem>,
}

impl Queue {
    pub fn finite(items: Vec<QueueItem>) -> Self {
        Self {
            behavior: QueueBehavior::Finite,
            items,
        }
    }

    pub fn infinite(items: Vec<QueueItem>) -> Self {
        Self {
            behavior: QueueBehavior::Infinite,
            items,
        }
    }

    pub fn is_infinite(&self) -> bool {
        self.behavior == QueueBehavior::Infinite
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueCursor {
    pub queue: Queue,
    pub current_index: Option<usize>,
    pub repeat: RepeatMode,
    pub shuffle: ShuffleMode,
}

impl QueueCursor {
    pub fn new(queue: Queue) -> Self {
        Self {
            current_index: (!queue.items.is_empty()).then_some(0),
            queue,
            repeat: RepeatMode::Off,
            shuffle: ShuffleMode::Off,
        }
    }

    pub fn current(&self) -> Option<&QueueItem> {
        self.current_index
            .and_then(|index| self.queue.items.get(index))
    }

    pub fn advance_next(&mut self) -> Option<&QueueItem> {
        let current = self.current_index?;
        let next = current + 1;

        self.current_index = if next < self.queue.items.len() {
            Some(next)
        } else if self.repeat == RepeatMode::All || self.queue.is_infinite() {
            Some(0)
        } else {
            None
        };

        self.current()
    }

    pub fn previous(&mut self) -> Option<&QueueItem> {
        let current = self.current_index?;

        self.current_index = if current > 0 {
            Some(current - 1)
        } else if self.repeat == RepeatMode::All {
            self.queue.items.len().checked_sub(1)
        } else {
            Some(0)
        };

        self.current()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    Off,
    One,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShuffleMode {
    Off,
    FutureQueue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueBehavior {
    Finite,
    Infinite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueItem {
    pub track_id: Option<TrackId>,
    pub provider_identity: Option<ProviderIdentity>,
    pub title: String,
    pub artist: Option<String>,
    pub relation_hint: Option<String>,
}

impl QueueItem {
    pub fn local(track_id: TrackId, title: impl Into<String>) -> Self {
        Self {
            track_id: Some(track_id),
            provider_identity: None,
            title: title.into(),
            artist: None,
            relation_hint: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcquisitionSource {
    UserImport,
    ProviderDownload {
        provider: ProviderId,
        external_id: String,
    },
    Tool {
        name: String,
        source_url: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquisitionIntent {
    pub id: CoreId,
    pub track_id: Option<TrackId>,
    pub provider_identity: Option<ProviderIdentity>,
    pub source_url: Option<String>,
    pub target_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayPreparationJob {
    pub id: CoreId,
    pub track_id: TrackId,
    pub provider_identity: Option<ProviderIdentity>,
    pub source_url: Option<String>,
    pub state: PlayPreparationState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayPreparationPlan {
    pub job: PlayPreparationJob,
    pub tool: PreparationTool,
    pub target_path: PathBuf,
    pub allow_progressive_playback: bool,
}

impl PlayPreparationPlan {
    pub fn ytdlp(job: PlayPreparationJob, target_path: PathBuf) -> Self {
        Self {
            job,
            tool: PreparationTool::YtDlp,
            target_path,
            allow_progressive_playback: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PreparationTool {
    YtDlp,
}

impl PlayPreparationJob {
    pub fn new(
        track_id: TrackId,
        provider_identity: Option<ProviderIdentity>,
        source_url: Option<String>,
    ) -> Self {
        Self {
            id: CoreId::new(),
            track_id,
            provider_identity,
            source_url,
            state: PlayPreparationState::Preparing,
        }
    }

    pub fn can_start_playback(&self) -> bool {
        matches!(
            self.state,
            PlayPreparationState::ReadyToPlay | PlayPreparationState::PlayingWhilePreparing
        )
    }

    pub fn complete(&mut self, asset_id: LocalAssetId) {
        self.state = PlayPreparationState::Completed { asset_id };
    }

    pub fn fail(&mut self, reason: impl Into<String>) {
        self.state = PlayPreparationState::Failed {
            reason: reason.into(),
        };
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayPreparationState {
    Preparing,
    Buffering,
    ReadyToPlay,
    PlayingWhilePreparing,
    Completed { asset_id: LocalAssetId },
    Failed { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    pub id: ProviderId,
    pub display_name: String,
    pub version: String,
    pub capabilities: Vec<ProviderCapability>,
}

impl ProviderDescriptor {
    pub fn supports(&self, capability: &ProviderCapability) -> bool {
        self.capabilities
            .iter()
            .any(|candidate| candidate == capability)
    }

    pub fn supports_label(&self, label: &str) -> bool {
        self.capabilities
            .iter()
            .any(|capability| capability.label == label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderCapability {
    pub label: String,
}

impl ProviderCapability {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderSessionState {
    SignedOut,
    SignedIn,
    SessionExpired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderPluginState {
    NotInstalled,
    Enabled,
    Incompatible,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowSyncState {
    pub provider: ProviderId,
    pub track_id: TrackId,
    pub identity: ProviderIdentity,
    pub provider_position: Option<Duration>,
    pub drift: Option<Duration>,
    pub muted: bool,
    pub session_state: ProviderSessionState,
    pub issue: Option<String>,
}

impl ShadowSyncState {
    pub fn has_follow_up_issue(&self) -> bool {
        self.session_state == ProviderSessionState::SessionExpired || self.issue.is_some()
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboxItem {
    pub provider: Option<ProviderId>,
    pub severity: InboxSeverity,
    pub message: String,
    pub recoverable: bool,
}

impl InboxItem {
    pub fn provider_attention(provider: ProviderId, message: impl Into<String>) -> Self {
        Self {
            provider: Some(provider),
            severity: InboxSeverity::Attention,
            message: message.into(),
            recoverable: true,
        }
    }

    pub fn play_preparation_failure(message: impl Into<String>) -> Self {
        Self {
            provider: None,
            severity: InboxSeverity::Error,
            message: message.into(),
            recoverable: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InboxSeverity {
    Info,
    Attention,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackChromeState {
    pub current_track: Option<String>,
    pub can_control: bool,
    pub management_entry_visible: bool,
}

impl PlaybackChromeState {
    pub fn idle() -> Self {
        Self {
            current_track: None,
            can_control: true,
            management_entry_visible: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderFollowUpIndicator {
    pub provider: ProviderId,
    pub visible: bool,
    pub detail_required: bool,
}

impl ProviderFollowUpIndicator {
    pub fn from_shadow_sync(sync: &ShadowSyncState) -> Self {
        Self {
            provider: sync.provider.clone(),
            visible: true,
            detail_required: sync.has_follow_up_issue(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListeningEvent {
    pub id: CoreId,
    pub track_id: TrackId,
    pub at: SystemTime,
    pub position: Duration,
    pub kind: ListeningEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListeningEventKind {
    Started,
    Paused,
    Resumed,
    Seeked {
        from: Duration,
        to: Duration,
    },
    Completed,
    RepeatedSegment {
        start: Duration,
        end: Duration,
        count: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSnapshot {
    pub active_track: Option<TrackHandle>,
    pub active_document: Option<Box<TrackDocument>>,
    pub active_asset: Option<LocalAssetId>,
    pub playback_context: Option<PlaybackContext>,
    pub play_preparations: Vec<PlayPreparationJob>,
    pub shadow_sync: Vec<ShadowSyncState>,
    pub providers: Vec<ProviderDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AppEvent {
    SnapshotChanged(AppSnapshot),
    DocumentPersisted(Box<TrackDocument>),
    Listening(ListeningEvent),
    ProviderAttention {
        provider: ProviderId,
        detail: String,
    },
    ShadowSyncChanged(ShadowSyncState),
    LocalAssetAdded(LocalAsset),
    PlayPreparationChanged(PlayPreparationJob),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider_id() -> ProviderId {
        ProviderId("ytm".to_owned())
    }

    fn track_id() -> TrackId {
        TrackId(Uuid::now_v7())
    }

    fn identity() -> ProviderIdentity {
        ProviderIdentity {
            provider: provider_id(),
            external_id: "external-track".to_owned(),
            url: Some("https://music.example/track".to_owned()),
        }
    }

    #[test]
    fn provider_descriptor_reports_supported_capabilities() {
        let descriptor = ProviderDescriptor {
            id: provider_id(),
            display_name: "YouTube Music".to_owned(),
            version: "0.1.0".to_owned(),
            capabilities: vec![
                ProviderCapability::new("search"),
                ProviderCapability::new("shadow-sync"),
            ],
        };

        assert!(descriptor.supports(&ProviderCapability::new("search")));
        assert!(descriptor.supports(&ProviderCapability::new("shadow-sync")));
        assert!(!descriptor.supports(&ProviderCapability::new("relation-hints")));
        assert!(descriptor.supports_label("search"));
    }

    #[test]
    fn rejected_track_match_is_not_accepted() {
        let accepted = TrackMatch {
            track_id: track_id(),
            identity: identity(),
            kind: TrackMatchKind::UserConfirmed,
            confidence: 1.0,
            duration_delta: None,
        };

        let rejected = TrackMatch {
            kind: TrackMatchKind::Rejected,
            confidence: 1.0,
            ..accepted.clone()
        };

        assert!(accepted.is_accepted());
        assert!(!rejected.is_accepted());
    }

    #[test]
    fn shadow_sync_muted_without_issue_is_ready_for_statistics_sync() {
        let state = ShadowSyncState {
            provider: provider_id(),
            track_id: track_id(),
            identity: identity(),
            provider_position: Some(Duration::from_secs(42)),
            drift: Some(Duration::from_millis(900)),
            muted: true,
            session_state: ProviderSessionState::SignedIn,
            issue: None,
        };

        assert!(state.is_muted());
        assert!(!state.has_follow_up_issue());
    }

    #[test]
    fn shadow_sync_issue_is_follow_up_not_playback_blocker() {
        let state = ShadowSyncState {
            provider: provider_id(),
            track_id: track_id(),
            identity: identity(),
            provider_position: None,
            drift: None,
            muted: true,
            session_state: ProviderSessionState::SessionExpired,
            issue: None,
        };

        assert!(state.has_follow_up_issue());
        assert!(state.is_muted());
    }

    #[test]
    fn provider_plugin_state_has_no_disabled_variant() {
        let states = [
            ProviderPluginState::NotInstalled,
            ProviderPluginState::Enabled,
            ProviderPluginState::Incompatible,
            ProviderPluginState::Error,
        ];

        assert_eq!(states.len(), 4);
    }

    #[test]
    fn play_preparation_can_start_before_completion() {
        let mut job = PlayPreparationJob::new(track_id(), Some(identity()), None);

        assert!(!job.can_start_playback());
        job.state = PlayPreparationState::ReadyToPlay;
        assert!(job.can_start_playback());
        job.state = PlayPreparationState::PlayingWhilePreparing;
        assert!(job.can_start_playback());
        job.complete(LocalAssetId::new());
        assert!(!job.can_start_playback());
    }

    #[test]
    fn playback_context_manual_queue_is_finite() {
        let item = QueueItem::local(track_id(), "Song");
        let context = PlaybackContext::manual("Manual Queue", vec![item.clone()]);

        assert_eq!(context.kind, PlaybackContextKind::Manual);
        assert_eq!(context.queue.items, vec![item]);
        assert!(!context.queue.is_infinite());
    }

    #[test]
    fn queue_can_represent_logically_infinite_context() {
        let queue = Queue::infinite(vec![QueueItem::local(track_id(), "Seed")]);

        assert!(queue.is_infinite());
    }

    #[test]
    fn queue_cursor_moves_previous_and_next() {
        let mut cursor = QueueCursor::new(Queue::finite(vec![
            QueueItem::local(track_id(), "First"),
            QueueItem::local(track_id(), "Second"),
        ]));

        assert_eq!(
            cursor.current().map(|item| item.title.as_str()),
            Some("First")
        );
        assert_eq!(
            cursor.advance_next().map(|item| item.title.as_str()),
            Some("Second")
        );
        assert_eq!(
            cursor.previous().map(|item| item.title.as_str()),
            Some("First")
        );
    }

    #[test]
    fn queue_cursor_repeat_all_wraps_to_first_track() {
        let mut cursor =
            QueueCursor::new(Queue::finite(vec![QueueItem::local(track_id(), "Only")]));
        cursor.repeat = RepeatMode::All;

        assert_eq!(
            cursor.advance_next().map(|item| item.title.as_str()),
            Some("Only")
        );
    }

    #[test]
    fn song_search_result_routes_to_track_screen() {
        let identity = identity();
        let result = SearchResult {
            title: "Song".to_owned(),
            kind: SearchResultKind::Track,
            provider_identity: Some(identity.clone()),
            route: EntityRoute::Track(TrackEntityRef {
                track_id: None,
                provider_identity: Some(identity),
                source_url: None,
            }),
        };

        assert!(matches!(result.route, EntityRoute::Track(_)));
    }

    #[test]
    fn global_search_aggregates_mixed_sources_by_relevance() {
        let local = RankedSearchResult {
            result: SearchResult {
                title: "Local Song".to_owned(),
                kind: SearchResultKind::Track,
                provider_identity: None,
                route: EntityRoute::Track(TrackEntityRef {
                    track_id: Some(track_id()),
                    provider_identity: None,
                    source_url: None,
                }),
            },
            source: SearchResultSource::LocalKnowledgeBase,
            relevance: 0.7,
        };
        let remote = RankedSearchResult {
            result: SearchResult {
                title: "Remote Song".to_owned(),
                kind: SearchResultKind::Track,
                provider_identity: Some(identity()),
                route: EntityRoute::Track(TrackEntityRef {
                    track_id: None,
                    provider_identity: Some(identity()),
                    source_url: None,
                }),
            },
            source: SearchResultSource::RemoteProvider,
            relevance: 0.9,
        };

        let aggregate = GlobalSearchResults::aggregate(vec![local, remote], Vec::new());

        assert_eq!(
            aggregate.results[0].source,
            SearchResultSource::RemoteProvider
        );
        assert_eq!(
            aggregate.results[1].source,
            SearchResultSource::LocalKnowledgeBase
        );
    }

    #[test]
    fn global_search_warnings_do_not_block_partial_results() {
        let aggregate = GlobalSearchResults::aggregate(
            vec![RankedSearchResult {
                result: SearchResult {
                    title: "Known Song".to_owned(),
                    kind: SearchResultKind::Track,
                    provider_identity: None,
                    route: EntityRoute::Track(TrackEntityRef {
                        track_id: Some(track_id()),
                        provider_identity: None,
                        source_url: None,
                    }),
                },
                source: SearchResultSource::LocalKnowledgeBase,
                relevance: 1.0,
            }],
            vec![SearchWarning {
                provider: Some(provider_id()),
                message: "remote search failed".to_owned(),
            }],
        );

        assert_eq!(aggregate.results.len(), 1);
        assert!(aggregate.has_partial_provider_failure());
    }

    #[test]
    fn unresolved_provider_url_falls_back_to_search_route() {
        let route = EntityRoute::SearchFallback {
            query: "https://example.test/unresolved".to_owned(),
        };

        assert_eq!(
            route,
            EntityRoute::SearchFallback {
                query: "https://example.test/unresolved".to_owned()
            }
        );
    }

    #[test]
    fn collection_urls_can_route_to_album_or_playlist_screen() {
        let album = EntityRoute::Album(AlbumEntityRef {
            provider: Some(provider_id()),
            external_id: Some("album-1".to_owned()),
            source_url: None,
            title: Some("Album".to_owned()),
        });
        let playlist = EntityRoute::Playlist(PlaylistEntityRef {
            provider: Some(provider_id()),
            external_id: Some("playlist-1".to_owned()),
            source_url: None,
            title: Some("Playlist".to_owned()),
        });

        assert!(matches!(album, EntityRoute::Album(_)));
        assert!(matches!(playlist, EntityRoute::Playlist(_)));
    }

    #[test]
    fn minimal_entity_surfaces_support_listening_and_organizing() {
        let surfaces = [
            EntitySurface::minimal(
                "Track",
                SearchResultKind::Track,
                EntityRoute::Track(TrackEntityRef {
                    track_id: Some(track_id()),
                    provider_identity: None,
                    source_url: None,
                }),
            ),
            EntitySurface::minimal(
                "Artist",
                SearchResultKind::Artist,
                EntityRoute::Channel(ChannelEntityRef {
                    provider: Some(provider_id()),
                    external_id: Some("artist".to_owned()),
                    source_url: None,
                    display_name: Some("Artist".to_owned()),
                }),
            ),
            EntitySurface::minimal(
                "Channel",
                SearchResultKind::Channel,
                EntityRoute::Channel(ChannelEntityRef {
                    provider: Some(provider_id()),
                    external_id: Some("channel".to_owned()),
                    source_url: None,
                    display_name: Some("Channel".to_owned()),
                }),
            ),
            EntitySurface::minimal(
                "Album",
                SearchResultKind::Album,
                EntityRoute::Album(AlbumEntityRef {
                    provider: Some(provider_id()),
                    external_id: Some("album".to_owned()),
                    source_url: None,
                    title: Some("Album".to_owned()),
                }),
            ),
            EntitySurface::minimal(
                "Playlist",
                SearchResultKind::Playlist,
                EntityRoute::Playlist(PlaylistEntityRef {
                    provider: Some(provider_id()),
                    external_id: Some("playlist".to_owned()),
                    source_url: None,
                    title: Some("Playlist".to_owned()),
                }),
            ),
        ];

        assert!(surfaces
            .iter()
            .all(EntitySurface::supports_listening_and_organizing));
    }

    #[test]
    fn ytdlp_play_preparation_plan_allows_progressive_playback() {
        let job = PlayPreparationJob::new(
            track_id(),
            Some(identity()),
            Some("https://music.youtube.com/watch?v=abc".to_owned()),
        );

        let plan = PlayPreparationPlan::ytdlp(job, PathBuf::from("prepared/song.opus"));

        assert_eq!(plan.tool, PreparationTool::YtDlp);
        assert!(plan.allow_progressive_playback);
        assert_eq!(plan.target_path, PathBuf::from("prepared/song.opus"));
    }

    #[test]
    fn track_screen_surface_exposes_play_queue_and_organize_actions() {
        let surface = EntitySurface::track_screen(
            "Track",
            EntityRoute::Track(TrackEntityRef {
                track_id: Some(track_id()),
                provider_identity: None,
                source_url: None,
            }),
        );

        assert_eq!(
            surface.actions,
            [
                EntitySurfaceAction::Play,
                EntitySurfaceAction::AddToQueue,
                EntitySurfaceAction::Organize,
            ]
        );
    }

    #[test]
    fn inbox_items_cover_provider_attention_and_play_preparation_failure() {
        let attention = InboxItem::provider_attention(provider_id(), "login required");
        let failure = InboxItem::play_preparation_failure("yt-dlp failed");

        assert_eq!(attention.severity, InboxSeverity::Attention);
        assert_eq!(failure.severity, InboxSeverity::Error);
        assert!(attention.recoverable);
        assert!(failure.recoverable);
    }

    #[test]
    fn playback_chrome_keeps_management_entry_available_when_idle() {
        let chrome = PlaybackChromeState::idle();

        assert!(chrome.can_control);
        assert!(chrome.management_entry_visible);
        assert!(chrome.current_track.is_none());
    }

    #[test]
    fn provider_follow_up_indicator_requests_detail_only_for_issues() {
        let mut sync = ShadowSyncState {
            provider: provider_id(),
            track_id: track_id(),
            identity: identity(),
            provider_position: None,
            drift: None,
            muted: true,
            session_state: ProviderSessionState::SignedIn,
            issue: None,
        };

        let clean = ProviderFollowUpIndicator::from_shadow_sync(&sync);
        sync.session_state = ProviderSessionState::SessionExpired;
        let attention = ProviderFollowUpIndicator::from_shadow_sync(&sync);

        assert!(clean.visible);
        assert!(!clean.detail_required);
        assert!(attention.detail_required);
    }

    #[test]
    fn youtube_music_watch_url_resolves_to_track_route() {
        let route =
            EntityRoute::resolve_youtube_music_url("https://music.youtube.com/watch?v=abc123");

        match route {
            EntityRoute::Track(track) => {
                let identity = track.provider_identity.expect("provider identity");
                assert_eq!(identity.provider, provider_id());
                assert_eq!(identity.external_id, "abc123");
            }
            other => panic!("expected track route, got {other:?}"),
        }
    }

    #[test]
    fn youtube_music_collection_urls_resolve_to_entity_routes() {
        let playlist =
            EntityRoute::resolve_youtube_music_url("https://music.youtube.com/playlist?list=PL123");
        let album =
            EntityRoute::resolve_youtube_music_url("https://music.youtube.com/browse/MPREabc");
        let channel =
            EntityRoute::resolve_youtube_music_url("https://www.youtube.com/channel/UC123");

        assert!(matches!(playlist, EntityRoute::Playlist(_)));
        assert!(matches!(album, EntityRoute::Album(_)));
        assert!(matches!(channel, EntityRoute::Channel(_)));
    }

    #[test]
    fn unsupported_youtube_music_url_falls_back_to_search() {
        let route = EntityRoute::resolve_youtube_music_url("https://example.test/not-supported");

        assert_eq!(
            route,
            EntityRoute::SearchFallback {
                query: "https://example.test/not-supported".to_owned()
            }
        );
    }
}
