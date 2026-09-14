//! Execution groups: sets of endpoints that execute a model together (tensor,
//! pipeline or data parallel) and the collective operations that keep them in
//! lockstep.

#![forbid(unsafe_code)]

use celyrion_transport::Endpoint;
use celyrion_types::{Error, Result, VersionedId};

/// A group of endpoints that execute together.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionGroup {
    /// Identity of the group.
    pub id: VersionedId,
    /// Members in rank order.
    pub members: Vec<Endpoint>,
}

impl ExecutionGroup {
    /// Number of members.
    pub fn size(&self) -> usize {
        self.members.len()
    }

    /// Rank of `endpoint` within the group, if it is a member.
    pub fn rank_of(&self, endpoint: &Endpoint) -> Option<usize> {
        self.members.iter().position(|m| m == endpoint)
    }
}

/// How a group is laid out before it can execute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupPlan {
    /// Group being prepared.
    pub group: ExecutionGroup,
    /// Degree of tensor parallelism.
    pub tensor_parallel: usize,
    /// Degree of pipeline parallelism.
    pub pipeline_parallel: usize,
}

impl GroupPlan {
    /// Checks that the parallelism degrees cover the group exactly.
    pub fn validate(&self) -> Result<()> {
        if self.tensor_parallel * self.pipeline_parallel == self.group.size() {
            Ok(())
        } else {
            Err(Error::CapacityExceeded(format!(
                "tp {} x pp {} != group size {}",
                self.tensor_parallel,
                self.pipeline_parallel,
                self.group.size()
            )))
        }
    }
}

/// Collective operations over an execution group.
pub trait Collective {
    /// Blocks until every member has reached this point.
    fn barrier(&mut self, group: &ExecutionGroup) -> impl Future<Output = Result<()>> + Send;

    /// Reduces `values` across the group with element-wise addition.
    fn all_reduce_sum(
        &mut self,
        group: &ExecutionGroup,
        values: Vec<f32>,
    ) -> impl Future<Output = Result<Vec<f32>>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_must_cover_group() {
        let group = ExecutionGroup {
            id: VersionedId::new("g", 1),
            members: vec![Endpoint("a".into()), Endpoint("b".into())],
        };
        assert_eq!(group.rank_of(&Endpoint("b".into())), Some(1));
        let plan = GroupPlan {
            group: group.clone(),
            tensor_parallel: 2,
            pipeline_parallel: 1,
        };
        assert!(plan.validate().is_ok());
        let bad = GroupPlan {
            group,
            tensor_parallel: 2,
            pipeline_parallel: 2,
        };
        assert!(bad.validate().is_err());
    }
}
