//! Pipelines: an ordered, bounded sequence of [`Stage`]s. Each stage consumes
//! artifacts from the registry and produces new ones (download, verify,
//! convert, compile, load).

#![forbid(unsafe_code)]

use celyrion_registry::ArtifactRef;
use celyrion_types::{Error, Result, VersionedId};

/// One processing step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage {
    /// Identity of the stage implementation.
    pub id: VersionedId,
    /// Artifacts the stage requires as input.
    pub inputs: Vec<ArtifactRef>,
    /// Maximum number of items the stage may hold in flight.
    pub bound: usize,
}

/// A bounded sequence of stages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipeline {
    stages: Vec<Stage>,
    max_stages: usize,
}

impl Pipeline {
    /// Creates an empty pipeline that accepts at most `max_stages` stages.
    pub fn new(max_stages: usize) -> Self {
        Self {
            stages: Vec::new(),
            max_stages,
        }
    }

    /// Appends a stage, failing when the pipeline is full.
    pub fn push(&mut self, stage: Stage) -> Result<()> {
        if self.stages.len() >= self.max_stages {
            return Err(Error::CapacityExceeded(format!(
                "pipeline of {} stages",
                self.max_stages
            )));
        }
        self.stages.push(stage);
        Ok(())
    }

    /// Stages in execution order.
    pub fn stages(&self) -> &[Stage] {
        &self.stages
    }

    /// Runs the pipeline. Not yet implemented.
    pub fn run(&self) -> Result<()> {
        Err(Error::NotImplemented("Pipeline::run"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_is_bounded() {
        let mut pipeline = Pipeline::new(1);
        let stage = Stage {
            id: VersionedId::new("download", 1),
            inputs: Vec::new(),
            bound: 4,
        };
        pipeline.push(stage.clone()).unwrap();
        assert!(pipeline.push(stage).is_err());
        assert_eq!(pipeline.stages().len(), 1);
    }
}
