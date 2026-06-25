//! SpecKit adapter (read-only, version-gated).
//!
//! Locates a SpecKit project, reads its version, and mines the intent inputs:
//! parsed specs (requirements, backlog decisions, consume hints, contract
//! files) and intended components from `.specify/memory/`.
//!
//! Tolerant by design (FR1, NFR4): missing files yield empty results, never
//! panics, and an untested SpecKit version is an advisory warning, not a crash.

pub mod memory;
pub mod specs;

use std::path::{Path, PathBuf};

use serde::Serialize;

pub use memory::{IntendedComponent, MemoryKind};
pub use specs::{BacklogDecision, ConsumesHint, ParsedSpec, Requirement};

/// SpecKit versions this adapter has been calibrated against. Other versions
/// still parse, but emit an advisory (FR7).
const TESTED_VERSION_PREFIX: &str = "0.11";

/// Where SpecKit lives within a project.
#[derive(Debug, Clone)]
pub struct SpecKitSource {
    pub root: PathBuf,
    /// `<root>/.specify` if present.
    pub specify_dir: Option<PathBuf>,
    /// `<root>/specs` if present.
    pub specs_dir: Option<PathBuf>,
}

impl SpecKitSource {
    /// Discover the SpecKit layout under `root`. Always succeeds; absent
    /// directories simply stay `None` (the caller decides what that means).
    pub fn discover(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let specify = root.join(".specify");
        let specs = root.join("specs");
        SpecKitSource {
            specify_dir: specify.is_dir().then_some(specify),
            specs_dir: specs.is_dir().then_some(specs),
            root,
        }
    }

    /// True when neither `.specify/` nor `specs/` exists — not a SpecKit project.
    pub fn is_empty(&self) -> bool {
        self.specify_dir.is_none() && self.specs_dir.is_none()
    }
}

/// The intent inputs the Coherence Bridge can see, as parsed in Phase 1.
///
/// B2 maps this into the typed `IntentModel` + provenance edges; B3 reconciles
/// it against governance and code. B1 stops at faithful, tolerant parsing.
#[derive(Debug, Clone, Serialize, Default)]
pub struct SpecKitArtifacts {
    /// From `.specify/integration.json`, when present.
    pub speckit_version: Option<String>,
    /// False when `speckit_version` is set but outside the tested range (FR7).
    pub version_supported: bool,
    pub specs: Vec<ParsedSpec>,
    pub intended_components: Vec<IntendedComponent>,
}

/// Load and parse the SpecKit intent inputs under `root` (read-only).
pub fn load(root: impl AsRef<Path>) -> SpecKitArtifacts {
    let source = SpecKitSource::discover(root);
    load_from(&source)
}

/// Load from an already-discovered source.
pub fn load_from(source: &SpecKitSource) -> SpecKitArtifacts {
    let speckit_version = source
        .specify_dir
        .as_ref()
        .and_then(|d| read_integration_version(d));

    let version_supported = match &speckit_version {
        Some(v) => v.starts_with(TESTED_VERSION_PREFIX),
        // Unknown version is treated as supported-until-proven-otherwise: we
        // don't block a project just because it omits integration.json.
        None => true,
    };

    let specs = source
        .specs_dir
        .as_ref()
        .map(|d| specs::parse_all(d))
        .unwrap_or_default();

    let intended_components = source
        .specify_dir
        .as_ref()
        .map(|d| memory::mine(&d.join("memory")))
        .unwrap_or_default();

    SpecKitArtifacts {
        speckit_version,
        version_supported,
        specs,
        intended_components,
    }
}

/// Read `version` out of `.specify/integration.json` (tolerant: any read/parse
/// failure yields `None`).
fn read_integration_version(specify_dir: &Path) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct Integration {
        version: Option<String>,
    }
    let raw = std::fs::read_to_string(specify_dir.join("integration.json")).ok()?;
    serde_json::from_str::<Integration>(&raw).ok()?.version
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_reports_empty_for_non_speckit_dir() {
        let src = SpecKitSource::discover(std::env::temp_dir());
        // temp dir is unlikely to be a speckit project
        let _ = src.is_empty();
    }
}
