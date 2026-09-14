//! Rust-side contract for model workers, whatever language they are written in.
//!
//! A worker (Python, C++, CUDA, Triton) participates in three steps:
//! 1. it **proposes** how a model adapter would execute a batch,
//! 2. the runtime **prepares** one of the proposed execution variants,
//! 3. the worker reports a **completion event** that the runtime verifies
//!    before any state becomes visible.
//!
//! The implementations themselves live under `workers/` at the repository root;
//! this crate will host the FFI / IPC glue, hence `unsafe_code` is only warned,
//! not forbidden.

use celyrion_types::{Digest, Error, Result, VersionedId};
use serde::{Deserialize, Serialize};

/// The technology a worker is implemented in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkerKind {
    /// Pure Python adapter (see `workers/python`).
    Python,
    /// Native C++ worker (see `workers/native`).
    Native,
    /// CUDA kernels driven from a native worker (see `workers/native`).
    Cuda,
    /// Triton kernels driven from a Python worker (see `workers/python/.../kernels`).
    Triton,
}

/// A worker's proposal for executing a batch with a given model adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    /// Model adapter the proposal is for.
    pub adapter: VersionedId,
    /// Execution variants the worker can run, in order of preference.
    pub variants: Vec<PreparedVariant>,
}

/// One concrete way of executing a batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedVariant {
    /// Identity of the variant (kernel set, precision, layout, ...).
    pub id: VersionedId,
    /// Which worker technology executes it.
    pub kind: WorkerKind,
    /// Digest of the compiled/prepared artifact, once prepared.
    pub artifact: Option<Digest>,
}

/// Reported by a worker when execution finishes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionEvent {
    /// Variant that was executed.
    pub variant: VersionedId,
    /// Digest of the produced outputs.
    pub output: Digest,
    /// Whether the runtime has verified the event against its expectations.
    pub verified: bool,
}

/// Contract every worker host implements.
pub trait Worker: Send + Sync {
    /// Technology of this worker.
    fn kind(&self) -> WorkerKind;

    /// Asks the worker how it would execute work for `adapter`.
    fn propose(&self, adapter: &VersionedId) -> impl Future<Output = Result<Proposal>> + Send;

    /// Prepares one variant for execution, filling in its artifact digest.
    fn prepare(
        &self,
        variant: PreparedVariant,
    ) -> impl Future<Output = Result<PreparedVariant>> + Send;

    /// Executes a prepared variant and returns an unverified completion event.
    fn complete(
        &self,
        variant: &PreparedVariant,
    ) -> impl Future<Output = Result<CompletionEvent>> + Send;
}

/// Marks a completion event as verified once its output matches expectations.
pub fn verify(event: CompletionEvent, expected: Option<Digest>) -> Result<CompletionEvent> {
    match expected {
        Some(digest) if digest != event.output => Err(Error::VersionMismatch {
            expected: event.variant.clone(),
            found: event.variant,
        }),
        _ => Ok(CompletionEvent {
            verified: true,
            ..event
        }),
    }
}

/// A worker that accepts everything and does nothing; useful for wiring tests.
#[derive(Debug, Default)]
pub struct NullWorker;

impl Worker for NullWorker {
    fn kind(&self) -> WorkerKind {
        WorkerKind::Native
    }

    async fn propose(&self, adapter: &VersionedId) -> Result<Proposal> {
        Ok(Proposal {
            adapter: adapter.clone(),
            variants: vec![PreparedVariant {
                id: VersionedId::new("null", 1),
                kind: WorkerKind::Native,
                artifact: None,
            }],
        })
    }

    async fn prepare(&self, variant: PreparedVariant) -> Result<PreparedVariant> {
        Ok(PreparedVariant {
            artifact: Some(Digest::ZERO),
            ..variant
        })
    }

    async fn complete(&self, variant: &PreparedVariant) -> Result<CompletionEvent> {
        Ok(CompletionEvent {
            variant: variant.id.clone(),
            output: Digest::ZERO,
            verified: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn null_worker_round_trip() {
        let worker = NullWorker;
        let proposal = worker
            .propose(&VersionedId::new("adapter", 1))
            .await
            .unwrap();
        let variant = worker.prepare(proposal.variants[0].clone()).await.unwrap();
        let event = worker.complete(&variant).await.unwrap();
        assert!(!event.verified);
        assert!(verify(event, Some(Digest::ZERO)).unwrap().verified);
    }
}
