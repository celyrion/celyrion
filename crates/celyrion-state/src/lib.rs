//! Typed state bundles. A [`Bundle`] is the unit of state that the memory
//! manager caches, the transport moves, and the scheduler reasons about.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use celyrion_types::{Digest, Quantity, Unit, VersionedId};
use serde::{Deserialize, Serialize};

/// Key identifying one entry inside a bundle.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BundleKey(pub String);

/// A typed, digest-addressed collection of state entries.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bundle {
    /// Schema of the bundle.
    pub schema: Option<VersionedId>,
    /// Digest over all entries.
    pub digest: Digest,
    entries: BTreeMap<BundleKey, Vec<u8>>,
}

impl Bundle {
    /// Creates an empty bundle for the given schema.
    pub fn new(schema: VersionedId) -> Self {
        Self {
            schema: Some(schema),
            digest: Digest::ZERO,
            entries: BTreeMap::new(),
        }
    }

    /// Inserts or replaces an entry.
    pub fn insert(&mut self, key: BundleKey, bytes: Vec<u8>) {
        self.entries.insert(key, bytes);
    }

    /// Looks up an entry.
    pub fn get(&self, key: &BundleKey) -> Option<&[u8]> {
        self.entries.get(key).map(Vec::as_slice)
    }

    /// Total payload size of the bundle.
    pub fn size(&self) -> Quantity {
        let value = self.entries.values().map(|v| v.len() as u64).sum();
        Quantity {
            value,
            unit: Unit::Bytes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_tracks_size() {
        let mut bundle = Bundle::new(VersionedId::new("kv", 1));
        bundle.insert(BundleKey("a".into()), vec![0; 4]);
        assert_eq!(
            bundle.size(),
            Quantity {
                value: 4,
                unit: Unit::Bytes
            }
        );
        assert_eq!(bundle.get(&BundleKey("a".into())), Some(&[0u8; 4][..]));
    }
}
