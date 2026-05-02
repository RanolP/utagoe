use bincode::{Decode, Encode};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

impl ProtocolVersion {
    pub const CURRENT: Self = Self { major: 0, minor: 1 };

    pub fn is_compatible_with(self, other: Self) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct MessageEnvelope {
    pub version: ProtocolVersion,
    pub request_id: RequestId,
    pub method: Method,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum Method {
    Initialize,
    Capabilities,
    Search,
    Resolve,
    Acquire,
    ShadowSync,
    Relations,
    Diagnostics,
    Experimental(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct ProviderRequest {
    pub envelope: MessageEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct ProviderResponse {
    pub request_id: RequestId,
    pub result: Result<Vec<u8>, ProviderError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct ProviderError {
    pub code: ErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum ErrorCode {
    UnsupportedMethod,
    InvalidRequest,
    ProviderUnavailable,
    AttentionRequired,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum Capability {
    Search,
    Acquisition,
    ShadowSync,
    Metadata,
    RelationHints,
    BrowserHost,
    Storage,
    Experimental(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct ProviderIdentity {
    pub id: String,
    pub provider: String,
    pub external_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{decode, encode};

    #[test]
    fn current_protocol_is_compatible_with_same_major_older_minor() {
        assert!(ProtocolVersion::CURRENT.is_compatible_with(ProtocolVersion { major: 0, minor: 0 }));
        assert!(ProtocolVersion::CURRENT.is_compatible_with(ProtocolVersion::CURRENT));
        assert!(
            !ProtocolVersion::CURRENT.is_compatible_with(ProtocolVersion { major: 1, minor: 0 })
        );
        assert!(
            !ProtocolVersion { major: 0, minor: 0 }.is_compatible_with(ProtocolVersion::CURRENT)
        );
    }

    #[test]
    fn provider_request_round_trips_through_bincode_payload() {
        let request = ProviderRequest {
            envelope: MessageEnvelope {
                version: ProtocolVersion::CURRENT,
                request_id: RequestId(7),
                method: Method::Search,
                payload: vec![1, 2, 3],
            },
        };

        let encoded = encode(&request).expect("request must encode");
        let decoded: ProviderRequest = decode(&encoded).expect("request must decode");

        assert_eq!(decoded, request);
    }
}
