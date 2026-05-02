//! Provider RPC protocol types.
//!
//! The protocol is intentionally small while YTM and SoundCloud providers are
//! being built. Messages are transported as byte buffers and encoded with
//! bincode 2.

pub mod codec;
pub mod message;

pub use codec::{decode, encode, ProtocolCodecError};
pub use message::{
    Capability, ErrorCode, MessageEnvelope, Method, ProtocolVersion, ProviderError,
    ProviderRequest, ProviderResponse, RequestId,
};
