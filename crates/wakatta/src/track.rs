use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::identity::ExternalIdentity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub title: String,
    pub artists: Vec<String>,
    pub identities: Vec<ExternalIdentity>,
}

impl Track {
    pub fn new(title: impl Into<String>, artists: Vec<String>) -> Self {
        Self {
            id: TrackId(Uuid::now_v7()),
            title: title.into(),
            artists,
            identities: Vec::new(),
        }
    }

    pub fn attach_identity(&mut self, identity: ExternalIdentity) -> bool {
        if self.identities.contains(&identity) {
            return false;
        }

        self.identities.push(identity);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::ProviderId;

    #[test]
    fn track_new_starts_without_external_identities() {
        let track = Track::new("Blue Bird", vec!["Ikimono-gakari".to_owned()]);

        assert_eq!(track.title, "Blue Bird");
        assert_eq!(track.artists, ["Ikimono-gakari"]);
        assert!(track.identities.is_empty());
    }

    #[test]
    fn attach_identity_is_idempotent() {
        let mut track = Track::new("Song", vec!["Artist".to_owned()]);
        let identity = ExternalIdentity {
            provider: ProviderId("ytm".to_owned()),
            external_id: "abc".to_owned(),
            url: Some("https://music.youtube.com/watch?v=abc".to_owned()),
        };

        assert!(track.attach_identity(identity.clone()));
        assert!(!track.attach_identity(identity));
        assert_eq!(track.identities.len(), 1);
    }
}
