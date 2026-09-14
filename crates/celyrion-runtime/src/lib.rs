//! The core Celyrion crate. It owns the [`Runtime`], which loads a
//! [`ModelConfig`] and assembles the individual modules into a serving
//! process. Every module is re-exported here so downstream code (the CLI,
//! integration tests) only needs one dependency.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use celyrion_authority as authority;
pub use celyrion_cell as cell;
pub use celyrion_core as core;
pub use celyrion_gateway as gateway;
pub use celyrion_groups as groups;
pub use celyrion_memory as memory;
pub use celyrion_observe as observe;
pub use celyrion_pipeline as pipeline;
pub use celyrion_protocol as protocol;
pub use celyrion_registry as registry;
pub use celyrion_scheduler as scheduler;
pub use celyrion_sim as sim;
pub use celyrion_state as state;
pub use celyrion_supervisor as supervisor;
pub use celyrion_transport as transport;
pub use celyrion_types as types;
pub use celyrion_workers as workers;

/// Errors produced while loading or running the runtime.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    /// The config file could not be read.
    #[error("failed to read config {path}: {source}")]
    Read {
        /// Path that was read.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The config file could not be parsed.
    #[error("failed to parse config {path}: {source}")]
    Parse {
        /// Path that was parsed.
        path: PathBuf,
        /// Underlying parse error.
        #[source]
        source: serde_yaml_ng::Error,
    },
    /// An error from one of the modules.
    #[error(transparent)]
    Module(#[from] celyrion_types::Error),
}

/// Contents of a `model.yaml` file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Name of the model to serve.
    pub name: String,
    /// Where the model package lives.
    pub path: PathBuf,
    /// Workers allowed to execute this model.
    #[serde(default)]
    pub workers: Vec<workers::WorkerKind>,
    /// Memory budget for state bundles, in bytes.
    #[serde(default = "ModelConfig::default_memory_bytes")]
    pub memory_bytes: u64,
    /// Maximum batch size the scheduler may build.
    #[serde(default = "ModelConfig::default_max_batch")]
    pub max_batch: usize,
}

impl ModelConfig {
    fn default_memory_bytes() -> u64 {
        1 << 30
    }

    fn default_max_batch() -> usize {
        32
    }

    /// Parses a config from YAML text.
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml_ng::Error> {
        serde_yaml_ng::from_str(text)
    }
}

/// A loaded, not-yet-serving runtime.
#[derive(Debug)]
pub struct Runtime {
    config: ModelConfig,
}

impl Runtime {
    /// Loads the runtime from a `model.yaml` path.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, RuntimeError> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path).map_err(|source| RuntimeError::Read {
            path: path.to_owned(),
            source,
        })?;
        let config = ModelConfig::from_yaml(&text).map_err(|source| RuntimeError::Parse {
            path: path.to_owned(),
            source,
        })?;
        Ok(Self::from_config(config))
    }

    /// Builds a runtime from an already-parsed config.
    pub fn from_config(config: ModelConfig) -> Self {
        Self { config }
    }

    /// The config this runtime was built from.
    pub fn config(&self) -> &ModelConfig {
        &self.config
    }

    /// Serves the model. Currently a stub that logs and returns.
    pub async fn serve(self) -> Result<(), RuntimeError> {
        tracing::info!(
            model = %self.config.name,
            path = %self.config.path.display(),
            workers = ?self.config.workers,
            "serve is not implemented yet"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_parses_with_defaults() {
        let config =
            ModelConfig::from_yaml("name: tiny\npath: ./tiny\nworkers: [Python]\n").unwrap();
        assert_eq!(config.name, "tiny");
        assert_eq!(config.workers, vec![workers::WorkerKind::Python]);
        assert_eq!(config.max_batch, 32);
    }

    #[tokio::test]
    async fn serve_stub_returns_ok() {
        let config = ModelConfig::from_yaml("name: tiny\npath: ./tiny\n").unwrap();
        Runtime::from_config(config).serve().await.unwrap();
    }
}
