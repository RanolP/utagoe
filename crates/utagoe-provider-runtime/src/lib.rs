//! Provider runtime abstraction.
//!
//! The first concrete implementation is expected to be Extism-backed, but the
//! Utagoe provider protocol remains independent from any particular WASM host.

use utagoe_provider_protocol::{ProviderRequest, ProviderResponse};

pub trait ProviderRuntime {
    fn invoke(&mut self, request: ProviderRequest) -> anyhow::Result<ProviderResponse>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    NotInstalled,
    Enabled,
    Incompatible,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderAction {
    pub id: String,
    pub label: String,
}

impl ProviderAction {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderRegistration {
    pub provider_id: String,
    pub state: PluginState,
    pub actions: Vec<ProviderAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderSessionState {
    SignedOut,
    SignedIn,
    SessionExpired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttentionRequest {
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSessionReport {
    pub provider_id: String,
    pub state: ProviderSessionState,
    pub attention: Option<AttentionRequest>,
}

impl ProviderSessionReport {
    pub fn signed_in(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            state: ProviderSessionState::SignedIn,
            attention: None,
        }
    }

    pub fn signed_out(provider_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            state: ProviderSessionState::SignedOut,
            attention: Some(AttentionRequest {
                reason: reason.into(),
            }),
        }
    }

    pub fn expired(provider_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            state: ProviderSessionState::SessionExpired,
            attention: Some(AttentionRequest {
                reason: reason.into(),
            }),
        }
    }

    pub fn webview_visibility(&self) -> WebviewVisibility {
        if self.attention.is_some() {
            WebviewVisibility::UserAttention
        } else {
            WebviewVisibility::Hidden
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebviewVisibility {
    Hidden,
    UserAttention,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSearchQuery {
    pub text: String,
    pub page_token: Option<String>,
}

impl ProviderSearchQuery {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            page_token: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSearchPage {
    pub items: Vec<ProviderSearchItem>,
    pub next_page_token: Option<String>,
    pub warning: Option<String>,
}

impl ProviderSearchPage {
    pub fn single(items: Vec<ProviderSearchItem>) -> Self {
        Self {
            items,
            next_page_token: None,
            warning: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSearchItem {
    pub title: String,
    pub kind: ProviderEntityKind,
    pub external_id: String,
    pub source_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderEntityKind {
    Track,
    Artist,
    Channel,
    Album,
    Playlist,
}

pub trait ProviderSearch {
    fn search(&mut self, query: ProviderSearchQuery) -> anyhow::Result<ProviderSearchPage>;
}

impl ProviderRegistration {
    pub fn enabled(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            state: PluginState::Enabled,
            actions: Vec::new(),
        }
    }

    pub fn with_action(mut self, action: ProviderAction) -> Self {
        self.actions.push(action);
        self
    }
}

#[derive(Debug, Default)]
pub struct RuntimeRegistry {
    providers: Vec<ProviderRegistration>,
}

impl RuntimeRegistry {
    pub fn register(&mut self, provider_id: impl Into<String>) {
        self.register_provider(ProviderRegistration::enabled(provider_id));
    }

    pub fn register_provider(&mut self, provider: ProviderRegistration) {
        self.providers.push(provider);
    }

    pub fn providers(&self) -> &[ProviderRegistration] {
        &self.providers
    }

    pub fn provider(&self, provider_id: &str) -> Option<&ProviderRegistration> {
        self.providers
            .iter()
            .find(|provider| provider.provider_id == provider_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_preserves_provider_registration_order() {
        let mut registry = RuntimeRegistry::default();

        registry.register("ytm");
        registry.register("soundcloud");

        assert_eq!(
            registry
                .providers()
                .iter()
                .map(|provider| provider.provider_id.as_str())
                .collect::<Vec<_>>(),
            ["ytm", "soundcloud"]
        );
    }

    #[test]
    fn empty_registry_has_no_providers() {
        let registry = RuntimeRegistry::default();

        assert!(registry.providers().is_empty());
    }

    #[test]
    fn provider_registration_tracks_mvp_plugin_state_and_actions() {
        let mut registry = RuntimeRegistry::default();

        registry.register_provider(
            ProviderRegistration {
                provider_id: "ytm".to_owned(),
                state: PluginState::NotInstalled,
                actions: vec![ProviderAction::new("login", "Sign in")],
            }
            .with_action(ProviderAction::new("diagnostics", "Diagnostics")),
        );

        let provider = registry.provider("ytm").expect("provider registration");
        assert_eq!(provider.state, PluginState::NotInstalled);
        assert_eq!(
            provider
                .actions
                .iter()
                .map(|action| action.id.as_str())
                .collect::<Vec<_>>(),
            ["login", "diagnostics"]
        );
    }

    #[test]
    fn plugin_state_has_only_mvp_variants() {
        let states = [
            PluginState::NotInstalled,
            PluginState::Enabled,
            PluginState::Incompatible,
            PluginState::Error,
        ];

        assert_eq!(states.len(), 4);
    }

    #[test]
    fn signed_in_provider_session_keeps_webview_hidden() {
        let session = ProviderSessionReport::signed_in("ytm");

        assert_eq!(session.state, ProviderSessionState::SignedIn);
        assert_eq!(session.webview_visibility(), WebviewVisibility::Hidden);
    }

    #[test]
    fn signed_out_and_expired_sessions_request_user_attention() {
        let signed_out = ProviderSessionReport::signed_out("ytm", "login required");
        let expired = ProviderSessionReport::expired("ytm", "session expired");

        assert_eq!(signed_out.state, ProviderSessionState::SignedOut);
        assert_eq!(expired.state, ProviderSessionState::SessionExpired);
        assert_eq!(
            signed_out.webview_visibility(),
            WebviewVisibility::UserAttention
        );
        assert_eq!(
            expired.webview_visibility(),
            WebviewVisibility::UserAttention
        );
    }

    #[derive(Default)]
    struct MemorySearchProvider {
        pages: Vec<ProviderSearchPage>,
    }

    impl ProviderSearch for MemorySearchProvider {
        fn search(&mut self, _query: ProviderSearchQuery) -> anyhow::Result<ProviderSearchPage> {
            Ok(self.pages.remove(0))
        }
    }

    #[test]
    fn provider_search_returns_paged_entity_results() {
        let mut provider = MemorySearchProvider {
            pages: vec![ProviderSearchPage {
                items: vec![ProviderSearchItem {
                    title: "Song".to_owned(),
                    kind: ProviderEntityKind::Track,
                    external_id: "video-1".to_owned(),
                    source_url: Some("https://music.youtube.com/watch?v=video-1".to_owned()),
                }],
                next_page_token: Some("page-2".to_owned()),
                warning: None,
            }],
        };

        let page = provider
            .search(ProviderSearchQuery::new("song"))
            .expect("search page");

        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].kind, ProviderEntityKind::Track);
        assert_eq!(page.next_page_token, Some("page-2".to_owned()));
    }

    #[test]
    fn provider_search_page_can_report_non_blocking_warning() {
        let page = ProviderSearchPage {
            items: Vec::new(),
            next_page_token: None,
            warning: Some("provider degraded".to_owned()),
        };

        assert_eq!(page.warning, Some("provider degraded".to_owned()));
    }
}
