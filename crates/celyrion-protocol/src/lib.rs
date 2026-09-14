//! Versioned protocol messages built on top of [`celyrion_types`]. This crate
//! defines *what* crosses component boundaries; how it moves is the job of
//! `celyrion-transport`.

#![forbid(unsafe_code)]

use celyrion_types::{Digest, Error, Result, VersionedId};
use serde::{Deserialize, Serialize};

/// Protocol version negotiated between peers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProtocolVersion(pub u32);

impl ProtocolVersion {
    /// The version this crate speaks.
    pub const CURRENT: ProtocolVersion = ProtocolVersion(1);
}

/// Categories of messages carried by the protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageKind {
    /// A request expecting a response.
    Request,
    /// A response to an earlier request.
    Response,
    /// A fire-and-forget event.
    Event,
}

/// A single protocol message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Protocol version the message was encoded with.
    pub version: ProtocolVersion,
    /// Identity of the message schema.
    pub id: VersionedId,
    /// Category of the message.
    pub kind: MessageKind,
    /// Digest of `payload`.
    pub digest: Digest,
    /// Opaque, schema-specific payload.
    pub payload: Vec<u8>,
}

impl Message {
    /// Validates that the message was produced by a compatible protocol version.
    pub fn check_version(&self) -> Result<()> {
        if self.version == ProtocolVersion::CURRENT {
            Ok(())
        } else {
            Err(Error::VersionMismatch {
                expected: VersionedId::new("protocol", u64::from(ProtocolVersion::CURRENT.0)),
                found: VersionedId::new("protocol", u64::from(self.version.0)),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_version_passes_check() {
        let msg = Message {
            version: ProtocolVersion::CURRENT,
            id: VersionedId::new("ping", 1),
            kind: MessageKind::Event,
            digest: Digest::ZERO,
            payload: Vec::new(),
        };
        assert!(msg.check_version().is_ok());
    }
}
