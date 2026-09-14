//! Authority: the single place where a change to shared state is *declared*
//! before it is *applied*. A [`Transaction`] carries a [`Declaration`] of the
//! state it intends to touch; an [`Authority`] decides whether to admit it.

#![forbid(unsafe_code)]

use celyrion_types::{Digest, Error, Result, VersionedId};

/// Up-front statement of what a transaction will read and write.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Declaration {
    /// Bundles the transaction reads.
    pub reads: Vec<Digest>,
    /// Bundles the transaction writes.
    pub writes: Vec<Digest>,
}

/// A unit of authoritative change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// Identity of the transaction.
    pub id: VersionedId,
    /// What the transaction declares it will touch.
    pub declaration: Declaration,
}

/// Outcome of asking an authority to admit a transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    /// The transaction may proceed.
    Admitted,
    /// The transaction conflicts with one already admitted.
    Conflict,
}

/// Decides which transactions are authoritative.
pub trait Authority {
    /// Attempts to admit `tx`.
    fn admit(&mut self, tx: &Transaction) -> Result<Admission>;

    /// Marks `tx` as finished, releasing anything it declared.
    fn release(&mut self, tx: &Transaction) -> Result<()>;
}

/// Authority that rejects overlapping writes while a transaction is in flight.
#[derive(Debug, Default)]
pub struct WriteExclusive {
    in_flight: Vec<Transaction>,
}

impl Authority for WriteExclusive {
    fn admit(&mut self, tx: &Transaction) -> Result<Admission> {
        let conflicts = self.in_flight.iter().any(|other| {
            other
                .declaration
                .writes
                .iter()
                .any(|w| tx.declaration.writes.contains(w) || tx.declaration.reads.contains(w))
        });
        if conflicts {
            return Ok(Admission::Conflict);
        }
        self.in_flight.push(tx.clone());
        Ok(Admission::Admitted)
    }

    fn release(&mut self, tx: &Transaction) -> Result<()> {
        let before = self.in_flight.len();
        self.in_flight.retain(|t| t.id != tx.id);
        if self.in_flight.len() == before {
            return Err(Error::NotImplemented("release of unknown transaction"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlapping_writes_conflict() {
        let mut authority = WriteExclusive::default();
        let a = Transaction {
            id: VersionedId::new("a", 1),
            declaration: Declaration {
                reads: vec![],
                writes: vec![Digest::ZERO],
            },
        };
        let b = Transaction {
            id: VersionedId::new("b", 1),
            ..a.clone()
        };
        assert_eq!(authority.admit(&a).unwrap(), Admission::Admitted);
        assert_eq!(authority.admit(&b).unwrap(), Admission::Conflict);
        authority.release(&a).unwrap();
        assert_eq!(authority.admit(&b).unwrap(), Admission::Admitted);
    }
}
