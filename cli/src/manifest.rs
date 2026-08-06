use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Parsed dist-manifest.yml
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DistManifest {
    pub version: String,
    pub description: String,
    pub files: Vec<String>,
    #[serde(default)]
    pub injections: Vec<Injection>,
    /// Paths this release no longer distributes. `update-framework` and
    /// `repair` delete them from an existing installation; `init` ignores the
    /// key entirely (nothing to retire in a fresh tree).
    ///
    /// `#[serde(default)]` is load-bearing: every `.straymark/dist-manifest.yml`
    /// written by a release before fw-4.42.0 lacks the key, and those copies are
    /// re-read by `repair` and `remove`.
    #[serde(default)]
    pub retired: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Injection {
    pub target: String,
    pub template: String,
    #[serde(default)]
    pub embed: Option<String>,
}

impl DistManifest {
    /// Load manifest from within a ZIP archive's extracted directory
    pub fn from_str(content: &str) -> Result<Self> {
        serde_yaml::from_str(content).context("Failed to parse dist-manifest.yml")
    }

    /// Load manifest from a file path
    pub fn load(path: &Path) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).context("Failed to read dist-manifest.yml")?;
        Self::from_str(&content)
    }

    /// Serialize manifest to YAML string
    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Failed to serialize dist-manifest.yml")
    }
}
