//! Supervisor: watches [`celyrion_observe::GenerationHealth`] and tears down
//! model generations safely — draining in-flight work before releasing state.

#![forbid(unsafe_code)]

use std::time::Duration;

use celyrion_core::Owned;
use celyrion_observe::{GenerationHealth, Health};
use celyrion_types::{Result, VersionedId};

/// How aggressively a generation is torn down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeardownPolicy {
    /// How long to wait for in-flight work before forcing teardown.
    pub drain_timeout: Duration,
}

impl Default for TeardownPolicy {
    fn default() -> Self {
        Self {
            drain_timeout: Duration::from_secs(30),
        }
    }
}

/// Supervises a set of generations.
#[derive(Debug)]
pub struct Supervisor {
    policy: TeardownPolicy,
    generations: Owned<Vec<GenerationHealth>>,
}

impl Supervisor {
    /// Creates a supervisor with the given policy.
    pub fn new(policy: TeardownPolicy) -> Self {
        Self {
            policy,
            generations: Owned::new(Vec::new()),
        }
    }

    /// Records the latest health signal for a generation.
    pub async fn observe(&self, signal: GenerationHealth) {
        self.generations
            .with(|gens| {
                gens.retain(|g| g.generation != signal.generation);
                gens.push(signal);
            })
            .await;
    }

    /// Generations that have failed and should be torn down.
    pub async fn failed(&self) -> Vec<VersionedId> {
        self.generations
            .with(|gens| {
                gens.iter()
                    .filter(|g| g.health == Health::Failed)
                    .map(|g| g.generation.clone())
                    .collect()
            })
            .await
    }

    /// Tears down a generation. Draining is not yet implemented; this only forgets it.
    pub async fn teardown(&self, generation: &VersionedId) -> Result<()> {
        tracing::info!(%generation, timeout = ?self.policy.drain_timeout, "teardown requested");
        self.generations
            .with(|gens| gens.retain(|g| &g.generation != generation))
            .await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn failed_generations_are_reported_until_torn_down() {
        let supervisor = Supervisor::new(TeardownPolicy::default());
        let generation = VersionedId::new("gen", 1);
        supervisor
            .observe(GenerationHealth {
                generation: generation.clone(),
                health: Health::Failed,
            })
            .await;
        assert_eq!(supervisor.failed().await, vec![generation.clone()]);
        supervisor.teardown(&generation).await.unwrap();
        assert!(supervisor.failed().await.is_empty());
    }
}
