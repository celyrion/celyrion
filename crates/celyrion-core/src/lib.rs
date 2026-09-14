//! Core execution model: pure state transitions and serialized ownership.
//!
//! A [`StateMachine`] describes how a piece of state evolves in response to
//! events without performing any I/O. [`Owned`] wraps a value so that exactly
//! one task holds it at a time; mailboxes and effect dispatch live in
//! `celyrion-cell`.

#![forbid(unsafe_code)]

use std::sync::Arc;

use celyrion_types::Result;
use tokio::sync::Mutex;

/// A deterministic state transition function.
pub trait StateMachine {
    /// State held by the machine.
    type State;
    /// Event applied to the state.
    type Event;

    /// Computes the next state from the current one and an event.
    fn transition(&self, state: &Self::State, event: Self::Event) -> Result<Self::State>;
}

/// A value whose access is serialized: only one holder may operate on it at a time.
#[derive(Debug, Clone)]
pub struct Owned<T> {
    inner: Arc<Mutex<T>>,
}

impl<T> Owned<T> {
    /// Wraps a value for serialized access.
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(value)),
        }
    }

    /// Runs `f` with exclusive access to the value.
    pub async fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.inner.lock().await;
        f(&mut guard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Counter;

    impl StateMachine for Counter {
        type State = u64;
        type Event = u64;

        fn transition(&self, state: &u64, event: u64) -> Result<u64> {
            Ok(state + event)
        }
    }

    #[test]
    fn transition_is_pure() {
        assert_eq!(Counter.transition(&1, 2).unwrap(), 3);
    }

    #[tokio::test]
    async fn owned_serializes_access() {
        let owned = Owned::new(0u32);
        owned.with(|v| *v += 1).await;
        assert_eq!(owned.with(|v| *v).await, 1);
    }
}
