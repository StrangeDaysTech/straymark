//! The typed intent model and its building blocks.
//!
//! `IntentModel` is the read-side product of Phase 1: SpecKit's intent
//! (specs + memory) joined with contract shapes mined from code and the
//! provenance edges that link a contract's consumers, producer, and the
//! decision that defined it. The coherence engine (B3) reconciles it; here we
//! only *assemble* it.

use std::path::Path;

use serde::Serialize;

use crate::speckit::{self, IntendedComponent, ParsedSpec};

/// Stable, cross-language join key for a contract (a normalized endpoint),
/// e.g. `services.health`.
pub type ContractId = String;

/// A pointer into a source artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceRef {
    /// Path relative to the project root.
    pub file: String,
    /// The declaration (struct/interface/spec id), when known.
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Lang {
    Go,
    Ts,
    /// Declared in a SpecKit markdown spec rather than code.
    Spec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ShapeRole {
    Producer,
    Consumer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Field {
    pub name: String,
    pub type_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct EnumDef {
    pub name: Option<String>,
    pub variants: Vec<String>,
}

/// The shape of a contract as seen from one side (a producer struct, a
/// consumer interface, …), keyed to a `ContractId`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContractShape {
    pub contract: ContractId,
    pub role: ShapeRole,
    pub language: Lang,
    pub source: SourceRef,
    pub fields: Vec<Field>,
    pub enums: Vec<EnumDef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionKind {
    Pm,
    Ailog,
    Aidec,
    Adr,
    Charter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecisionRef {
    pub kind: DecisionKind,
    pub id: String,
    /// Where the decision lives (spec id / file).
    pub location: String,
}

/// One contract, joined across its producer, consumers, and defining decisions.
#[derive(Debug, Clone, Serialize)]
pub struct IntentContract {
    pub id: ContractId,
    pub endpoint: Option<String>,
    pub producer: Option<ContractShape>,
    pub consumers: Vec<ContractShape>,
    pub defined_by: Vec<DecisionRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// A consumer → contract provenance edge — the core of issue #304: it makes a
/// contract's source-of-truth singular by linking every consumer to the
/// producer and the decision that shaped it.
#[derive(Debug, Clone, Serialize)]
pub struct ProvenanceEdge {
    pub consumer: SourceRef,
    pub contract: ContractId,
    pub producer: Option<SourceRef>,
    pub defined_by: Vec<DecisionRef>,
    pub confidence: Confidence,
}

/// The assembled intent model.
#[derive(Debug, Clone, Serialize, Default)]
pub struct IntentModel {
    pub speckit_version: Option<String>,
    pub version_supported: bool,
    pub specs: Vec<ParsedSpec>,
    pub intended_components: Vec<IntendedComponent>,
    pub contracts: Vec<IntentContract>,
    pub provenance: Vec<ProvenanceEdge>,
}

impl IntentModel {
    /// Build the intent model for the project at `root` (read-only): parse
    /// SpecKit, scan code for contract shapes, infer contracts + provenance.
    pub fn build(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref();
        let artifacts = speckit::load(root);
        let shapes = crate::codescan::scan(root);
        let (contracts, provenance) = crate::provenance::infer(&artifacts, &shapes);
        IntentModel {
            speckit_version: artifacts.speckit_version,
            version_supported: artifacts.version_supported,
            specs: artifacts.specs,
            intended_components: artifacts.intended_components,
            contracts,
            provenance,
        }
    }
}
