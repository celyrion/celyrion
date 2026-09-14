//! Foundational vocabulary for Celyrion: versioned identities, units, records,
//! digests and the shared error type. Every other crate depends on this one and
//! it depends on nothing internal.

#![forbid(unsafe_code)]

use std::fmt;

use serde::{Deserialize, Serialize};

/// A stable, human-readable name paired with a monotonically increasing version.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VersionedId {
    /// Stable name of the entity.
    pub name: String,
    /// Version of the entity; bumps whenever its definition changes.
    pub version: u64,
}

impl VersionedId {
    /// Creates a new identity at the given version.
    pub fn new(name: impl Into<String>, version: u64) -> Self {
        Self {
            name: name.into(),
            version,
        }
    }
}

impl fmt::Display for VersionedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

/// Units in which quantities are measured throughout the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Unit {
    /// Model tokens.
    Tokens,
    /// Raw bytes.
    Bytes,
    /// Microseconds of wall-clock time.
    Micros,
}

/// A quantity with an attached unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Quantity {
    /// Magnitude of the quantity.
    pub value: u64,
    /// Unit the magnitude is expressed in.
    pub unit: Unit,
}

/// Content digest (32 bytes, algorithm to be fixed by the protocol crate).
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Digest(pub [u8; 32]);

impl Digest {
    /// The all-zero digest, used as a placeholder before hashing is wired in.
    pub const ZERO: Digest = Digest([0; 32]);
}

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Digest(")?;
        for b in &self.0 {
            write!(f, "{b:02x}")?;
        }
        write!(f, ")")
    }
}

/// An immutable, digest-addressed record of some payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record<T> {
    /// Identity of the record's schema.
    pub id: VersionedId,
    /// Digest of the serialized payload.
    pub digest: Digest,
    /// The payload itself.
    pub payload: T,
}

/// Shared error type for Celyrion crates.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A version mismatch between two identities that were expected to agree.
    #[error("version mismatch: expected {expected}, found {found}")]
    VersionMismatch {
        /// Version that was expected.
        expected: VersionedId,
        /// Version that was found.
        found: VersionedId,
    },
    /// A bounded resource had no capacity left.
    #[error("capacity exceeded: {0}")]
    CapacityExceeded(String),
    /// Functionality that is declared but not yet implemented.
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
    /// Any other error.
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// Result alias using the shared [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versioned_id_displays_name_and_version() {
        assert_eq!(VersionedId::new("model", 3).to_string(), "model@3");
    }

    #[test]
    fn zero_digest_debug_is_hex() {
        assert!(format!("{:?}", Digest::ZERO).starts_with("Digest(0000"));
    }
}
