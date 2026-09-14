//! Memory management for state bundles: cache reuse, capacity reservations,
//! pins that keep bundles resident, and reclamation when capacity runs low.

#![forbid(unsafe_code)]

use celyrion_state::Bundle;
use celyrion_types::{Digest, Error, Quantity, Result, Unit};

/// A promise of capacity handed out before a bundle is materialized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reservation {
    /// Identifier of the reservation.
    pub id: u64,
    /// Amount reserved.
    pub amount: Quantity,
}

/// Keeps a bundle resident until dropped or released.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    /// Digest of the pinned bundle.
    pub digest: Digest,
}

/// Cache accounting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CacheStats {
    /// Bytes currently resident.
    pub resident_bytes: u64,
    /// Bytes currently reserved but not yet resident.
    pub reserved_bytes: u64,
    /// Number of cache hits served.
    pub hits: u64,
    /// Number of cache misses.
    pub misses: u64,
}

/// Chooses bundles to evict when capacity is exhausted.
pub trait Reclaimer {
    /// Returns digests to evict, in order, to free at least `needed`.
    fn reclaim(&mut self, needed: Quantity) -> Vec<Digest>;
}

/// Memory manager stub: tracks capacity and reservations, does not yet store bundles.
#[derive(Debug)]
pub struct MemoryManager {
    capacity_bytes: u64,
    stats: CacheStats,
    next_reservation: u64,
}

impl MemoryManager {
    /// Creates a manager with a fixed byte budget.
    pub fn new(capacity_bytes: u64) -> Self {
        Self {
            capacity_bytes,
            stats: CacheStats::default(),
            next_reservation: 0,
        }
    }

    /// Reserves capacity for a future bundle.
    pub fn reserve(&mut self, bytes: u64) -> Result<Reservation> {
        let in_use = self.stats.resident_bytes + self.stats.reserved_bytes;
        if in_use + bytes > self.capacity_bytes {
            return Err(Error::CapacityExceeded(format!(
                "{bytes} bytes requested, {} available",
                self.capacity_bytes - in_use
            )));
        }
        self.stats.reserved_bytes += bytes;
        self.next_reservation += 1;
        Ok(Reservation {
            id: self.next_reservation,
            amount: Quantity {
                value: bytes,
                unit: Unit::Bytes,
            },
        })
    }

    /// Materializes a bundle against a reservation and pins it.
    pub fn commit(&mut self, _reservation: Reservation, _bundle: Bundle) -> Result<Pin> {
        Err(Error::NotImplemented("MemoryManager::commit"))
    }

    /// Current accounting.
    pub fn stats(&self) -> CacheStats {
        self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reservations_are_bounded_by_capacity() {
        let mut mm = MemoryManager::new(10);
        assert!(mm.reserve(8).is_ok());
        assert!(mm.reserve(4).is_err());
        assert_eq!(mm.stats().reserved_bytes, 8);
    }
}
