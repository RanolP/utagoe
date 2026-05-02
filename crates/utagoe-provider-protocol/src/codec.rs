use bincode::{config, Decode, Encode};

const CONFIG: config::Configuration = config::standard();

pub fn encode<T: Encode>(value: &T) -> Result<Vec<u8>, ProtocolCodecError> {
    bincode::encode_to_vec(value, CONFIG).map_err(ProtocolCodecError::Encode)
}

pub fn decode<T: Decode<()>>(bytes: &[u8]) -> Result<T, ProtocolCodecError> {
    let (value, consumed) =
        bincode::decode_from_slice(bytes, CONFIG).map_err(ProtocolCodecError::Decode)?;

    if consumed != bytes.len() {
        return Err(ProtocolCodecError::TrailingBytes {
            consumed,
            total: bytes.len(),
        });
    }

    Ok(value)
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolCodecError {
    #[error("failed to encode provider message: {0}")]
    Encode(bincode::error::EncodeError),
    #[error("failed to decode provider message: {0}")]
    Decode(bincode::error::DecodeError),
    #[error("provider message had trailing bytes: consumed {consumed} of {total}")]
    TrailingBytes { consumed: usize, total: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MessageEnvelope, Method, ProtocolVersion, ProviderRequest, RequestId};

    #[test]
    fn decode_rejects_trailing_bytes() {
        let request = ProviderRequest {
            envelope: MessageEnvelope {
                version: ProtocolVersion::CURRENT,
                request_id: RequestId(1),
                method: Method::Initialize,
                payload: Vec::new(),
            },
        };

        let mut encoded = encode(&request).expect("request must encode");
        encoded.extend_from_slice(&[0, 1, 2]);

        let err = decode::<ProviderRequest>(&encoded).expect_err("trailing bytes must fail");

        assert!(matches!(err, ProtocolCodecError::TrailingBytes { .. }));
    }
}
