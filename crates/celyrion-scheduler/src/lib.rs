//! Scheduling: decides which requests are eligible, ranks them fairly against
//! their deadlines, builds batches, and estimates what each batch will cost.

#![forbid(unsafe_code)]

use celyrion_types::{Quantity, Result, Unit, VersionedId};

/// Whether a request may be scheduled right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eligibility {
    /// May be placed in a batch.
    Ready,
    /// Waiting on state (e.g. a cache fill) before it can run.
    Blocked,
    /// Past its deadline; should be rejected or degraded.
    Expired,
}

/// Absolute deadline in microseconds since the scheduler epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Deadline(pub u64);

/// A schedulable unit of work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// Identity of the request.
    pub id: VersionedId,
    /// When the request must complete.
    pub deadline: Deadline,
    /// Expected work size.
    pub size: Quantity,
}

/// A set of requests chosen to execute together.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Batch {
    /// Requests in the batch, in ranked order.
    pub requests: Vec<Request>,
}

/// Predicted cost of executing a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostEstimate {
    /// Predicted wall-clock time.
    pub latency: Quantity,
    /// Predicted memory footprint.
    pub memory: Quantity,
}

/// Orders eligible requests according to a fairness policy.
pub trait Ranker {
    /// Sorts `requests` in place, highest priority first.
    fn rank(&self, requests: &mut [Request]);
}

/// Earliest-deadline-first ranking.
#[derive(Debug, Default)]
pub struct EarliestDeadline;

impl Ranker for EarliestDeadline {
    fn rank(&self, requests: &mut [Request]) {
        requests.sort_by_key(|r| r.deadline);
    }
}

/// Scheduler stub.
#[derive(Debug)]
pub struct Scheduler<R: Ranker> {
    ranker: R,
    max_batch: usize,
}

impl<R: Ranker> Scheduler<R> {
    /// Creates a scheduler that never builds batches larger than `max_batch`.
    pub fn new(ranker: R, max_batch: usize) -> Self {
        Self { ranker, max_batch }
    }

    /// Classifies a request relative to `now`.
    pub fn eligibility(&self, request: &Request, now: Deadline) -> Eligibility {
        if request.deadline < now {
            Eligibility::Expired
        } else {
            Eligibility::Ready
        }
    }

    /// Ranks the given requests and takes up to `max_batch` of them.
    pub fn build_batch(&self, mut requests: Vec<Request>) -> Batch {
        self.ranker.rank(&mut requests);
        requests.truncate(self.max_batch);
        Batch { requests }
    }

    /// Estimates the cost of a batch. Not yet backed by a real model.
    pub fn estimate(&self, batch: &Batch) -> Result<CostEstimate> {
        let tokens: u64 = batch.requests.iter().map(|r| r.size.value).sum();
        Ok(CostEstimate {
            latency: Quantity {
                value: tokens,
                unit: Unit::Micros,
            },
            memory: Quantity {
                value: tokens,
                unit: Unit::Bytes,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(name: &str, deadline: u64) -> Request {
        Request {
            id: VersionedId::new(name, 1),
            deadline: Deadline(deadline),
            size: Quantity {
                value: 1,
                unit: Unit::Tokens,
            },
        }
    }

    #[test]
    fn batches_are_ranked_and_bounded() {
        let scheduler = Scheduler::new(EarliestDeadline, 2);
        let batch = scheduler.build_batch(vec![req("c", 30), req("a", 10), req("b", 20)]);
        let names: Vec<_> = batch.requests.iter().map(|r| r.id.name.as_str()).collect();
        assert_eq!(names, ["a", "b"]);
        assert_eq!(
            scheduler.eligibility(&req("x", 5), Deadline(6)),
            Eligibility::Expired
        );
    }
}
