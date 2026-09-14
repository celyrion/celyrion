//! Registry: immutable packages made of artifact references, addressed by
//! revision. Once a revision is published it never changes.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use celyrion_types::{Digest, Error, Result, VersionedId};
use serde::{Deserialize, Serialize};

/// Reference to a stored artifact (weights, tokenizer, compiled kernel, ...).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRef {
    /// Logical name within the package.
    pub name: String,
    /// Content digest of the artifact.
    pub digest: Digest,
    /// Size in bytes.
    pub size_bytes: u64,
}

/// A published, immutable revision of a package.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Revision(pub u64);

/// An immutable set of artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    /// Identity of the package; `version` doubles as its revision.
    pub id: VersionedId,
    /// Artifacts included in this revision.
    pub artifacts: Vec<ArtifactRef>,
}

impl Package {
    /// Revision of this package.
    pub fn revision(&self) -> Revision {
        Revision(self.id.version)
    }
}

/// Stores and resolves packages.
pub trait Registry {
    /// Publishes a package; fails if that revision already exists.
    fn publish(&mut self, package: Package) -> Result<Revision>;

    /// Resolves a package by name and revision.
    fn resolve(&self, name: &str, revision: Revision) -> Option<&Package>;
}

/// In-memory registry.
#[derive(Debug, Default)]
pub struct MemoryRegistry {
    packages: BTreeMap<(String, Revision), Package>,
}

impl Registry for MemoryRegistry {
    fn publish(&mut self, package: Package) -> Result<Revision> {
        let key = (package.id.name.clone(), package.revision());
        if self.packages.contains_key(&key) {
            return Err(Error::VersionMismatch {
                expected: package.id.clone(),
                found: package.id,
            });
        }
        let revision = key.1;
        self.packages.insert(key, package);
        Ok(revision)
    }

    fn resolve(&self, name: &str, revision: Revision) -> Option<&Package> {
        self.packages.get(&(name.to_owned(), revision))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revisions_are_immutable() {
        let mut registry = MemoryRegistry::default();
        let package = Package {
            id: VersionedId::new("llm", 1),
            artifacts: Vec::new(),
        };
        assert_eq!(registry.publish(package.clone()).unwrap(), Revision(1));
        assert!(registry.publish(package).is_err());
        assert!(registry.resolve("llm", Revision(1)).is_some());
    }
}
