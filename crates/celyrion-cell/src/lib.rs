//! Cells: a [`celyrion_core::StateMachine`] paired with a bounded [`Mailbox`]
//! and an [`EffectSink`] that receives the side effects each transition asks for.
//! Backpressure is a property of the mailbox capacity, never of the machine.

#![forbid(unsafe_code)]

use celyrion_types::{Error, Result};
use tokio::sync::mpsc;

/// A bounded, multi-producer single-consumer mailbox.
#[derive(Debug)]
pub struct Mailbox<T> {
    tx: mpsc::Sender<T>,
    rx: mpsc::Receiver<T>,
    capacity: usize,
}

impl<T> Mailbox<T> {
    /// Creates a mailbox that holds at most `capacity` pending items.
    pub fn bounded(capacity: usize) -> Self {
        let (tx, rx) = mpsc::channel(capacity.max(1));
        Self { tx, rx, capacity }
    }

    /// Maximum number of pending items.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns a handle other tasks can use to post into this mailbox.
    pub fn sender(&self) -> mpsc::Sender<T> {
        self.tx.clone()
    }

    /// Posts without waiting; fails when the mailbox is full.
    pub fn try_post(&self, item: T) -> Result<()> {
        self.tx
            .try_send(item)
            .map_err(|_| Error::CapacityExceeded(format!("mailbox of {}", self.capacity)))
    }

    /// Waits for the next item, or `None` once every sender is gone.
    pub async fn next(&mut self) -> Option<T> {
        self.rx.recv().await
    }
}

/// Side effects a cell may request as the outcome of a transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Emit a message to another cell or component.
    Emit(Vec<u8>),
    /// Schedule a wake-up after the given number of microseconds.
    Timer(u64),
    /// Log a diagnostic line.
    Log(String),
}

/// Receives effects for execution. Implemented by the runtime and by `celyrion-sim`.
pub trait EffectSink {
    /// Dispatches one effect.
    fn dispatch(&mut self, effect: Effect) -> Result<()>;
}

/// An [`EffectSink`] that records effects for inspection.
#[derive(Debug, Default)]
pub struct RecordingSink {
    /// Effects seen so far, in order.
    pub effects: Vec<Effect>,
}

impl EffectSink for RecordingSink {
    fn dispatch(&mut self, effect: Effect) -> Result<()> {
        self.effects.push(effect);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mailbox_is_bounded() {
        let mut mailbox = Mailbox::bounded(1);
        mailbox.try_post(1u8).unwrap();
        assert!(mailbox.try_post(2u8).is_err());
        assert_eq!(mailbox.next().await, Some(1));
    }

    #[test]
    fn recording_sink_collects_effects() {
        let mut sink = RecordingSink::default();
        sink.dispatch(Effect::Timer(10)).unwrap();
        assert_eq!(sink.effects, vec![Effect::Timer(10)]);
    }
}
