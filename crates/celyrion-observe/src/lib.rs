//! Observability: bounded telemetry that can never grow without limit, and a
//! health signal per model generation that the supervisor acts on.

#![forbid(unsafe_code)]

use std::collections::VecDeque;

use celyrion_types::VersionedId;

/// A single telemetry sample.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// Metric name.
    pub name: &'static str,
    /// Metric value.
    pub value: f64,
}

/// Fixed-capacity ring buffer of samples; the oldest sample is dropped when full.
#[derive(Debug)]
pub struct Telemetry {
    samples: VecDeque<Sample>,
    capacity: usize,
}

impl Telemetry {
    /// Creates a buffer that keeps the most recent `capacity` samples.
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }

    /// Records a sample.
    pub fn record(&mut self, sample: Sample) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        tracing::trace!(metric = sample.name, value = sample.value, "telemetry");
        self.samples.push_back(sample);
    }

    /// Samples from oldest to newest.
    pub fn samples(&self) -> impl Iterator<Item = &Sample> {
        self.samples.iter()
    }
}

/// Health of one model generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    /// Serving normally.
    Healthy,
    /// Serving, but degraded (e.g. missing deadlines).
    Degraded,
    /// Not serving; should be torn down.
    Failed,
}

/// Health signal for a model generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationHealth {
    /// Generation being described.
    pub generation: VersionedId,
    /// Current health.
    pub health: Health,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_drops_oldest() {
        let mut telemetry = Telemetry::new(2);
        for value in [1.0, 2.0, 3.0] {
            telemetry.record(Sample { name: "x", value });
        }
        let values: Vec<_> = telemetry.samples().map(|s| s.value).collect();
        assert_eq!(values, [2.0, 3.0]);
    }
}
