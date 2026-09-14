//! Deterministic simulation. Drives a real [`celyrion_core::StateMachine`]
//! through a scripted [`Trace`] of events, records the effects it requests via
//! [`celyrion_cell::RecordingSink`], and can inject [`Fault`]s at chosen steps.
//! The same trace always produces the same effects, which makes it the basis
//! for replaying production incidents offline.

#![forbid(unsafe_code)]

use celyrion_cell::{Effect, EffectSink, RecordingSink};
use celyrion_core::StateMachine;
use celyrion_types::{Error, Result};

/// A fault injected at a given step of a trace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fault {
    /// The transition at this step fails.
    TransitionError,
    /// Effects requested at this step are dropped instead of dispatched.
    DropEffects,
}

/// A scripted sequence of events with optional faults.
#[derive(Debug, Clone)]
pub struct Trace<E> {
    /// Events in order.
    pub events: Vec<E>,
    /// Faults keyed by the step index at which they fire.
    pub faults: Vec<(usize, Fault)>,
}

impl<E> Trace<E> {
    /// A trace with no faults.
    pub fn new(events: Vec<E>) -> Self {
        Self {
            events,
            faults: Vec::new(),
        }
    }

    fn fault_at(&self, step: usize) -> Option<&Fault> {
        self.faults.iter().find(|(s, _)| *s == step).map(|(_, f)| f)
    }
}

/// Runs a state machine deterministically.
#[derive(Debug)]
pub struct Simulation<M: StateMachine> {
    machine: M,
    sink: RecordingSink,
}

impl<M: StateMachine> Simulation<M>
where
    M::Event: Clone,
{
    /// Creates a simulation over `machine`.
    pub fn new(machine: M) -> Self {
        Self {
            machine,
            sink: RecordingSink::default(),
        }
    }

    /// Replays `trace` from `initial`, returning the final state.
    ///
    /// Effect derivation from state is not yet modelled; every successful
    /// step currently logs one [`Effect::Log`] so faults are observable.
    pub fn replay(&mut self, initial: M::State, trace: &Trace<M::Event>) -> Result<M::State> {
        let mut state = initial;
        for (step, event) in trace.events.iter().enumerate() {
            match trace.fault_at(step) {
                Some(Fault::TransitionError) => {
                    return Err(Error::NotImplemented("injected transition fault"));
                }
                Some(Fault::DropEffects) => {
                    state = self.machine.transition(&state, event.clone())?;
                }
                None => {
                    state = self.machine.transition(&state, event.clone())?;
                    self.sink.dispatch(Effect::Log(format!("step {step}")))?;
                }
            }
        }
        Ok(state)
    }

    /// Effects dispatched so far.
    pub fn effects(&self) -> &[Effect] {
        &self.sink.effects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Adder;

    impl StateMachine for Adder {
        type State = i64;
        type Event = i64;

        fn transition(&self, state: &i64, event: i64) -> Result<i64> {
            Ok(state + event)
        }
    }

    #[test]
    fn replay_is_deterministic() {
        let trace = Trace::new(vec![1, 2, 3]);
        let mut a = Simulation::new(Adder);
        let mut b = Simulation::new(Adder);
        assert_eq!(a.replay(0, &trace).unwrap(), 6);
        assert_eq!(b.replay(0, &trace).unwrap(), 6);
        assert_eq!(a.effects(), b.effects());
        assert_eq!(a.effects().len(), 3);
    }

    #[test]
    fn faults_fire_at_their_step() {
        let mut trace = Trace::new(vec![1, 2]);
        trace.faults.push((1, Fault::TransitionError));
        assert!(Simulation::new(Adder).replay(0, &trace).is_err());
    }
}
