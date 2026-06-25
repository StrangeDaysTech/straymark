//! The coherence engine — turns the `IntentModel` into actionable findings.
//!
//! Reconciles intended architecture (SpecKit) against governance and code and
//! emits the four high-confidence finding classes of spec §5, calibrated on the
//! #304 oracle:
//!
//! - **C1** `intended-not-implemented` — a component designed in `.specify/memory`
//!   with no implementing files (the PolicyEngine case).
//! - **C2** `consumer-field-without-producer` — a consumer wants a field no
//!   producer provides (the phantom `latency/cpu/memory`).
//! - **C3** `contract-shape-mismatch` — producer and consumer disagree on enum
//!   values (`OPERATIONAL/…` vs `GREEN/…`).
//! - **C4** `consumer-vs-changed-decision` — a consumer spec depends on a
//!   contract a later decision defined, without acknowledging it (spec 005 ↛ PM-002).
//!
//! `analyze` is pure over `(IntentModel, Inventory)`; the only I/O is building
//! those inputs (read-only).

use std::collections::BTreeSet;
use std::path::Path;

use serde::Serialize;

use crate::intent::{Confidence, IntentContract, IntentModel, SourceRef};

/// A flat, lowercased listing of on-disk file paths (read-only) — used to tell
/// whether an intended component has any implementing files (C1).
#[derive(Debug, Clone, Default)]
pub struct Inventory {
    pub files: Vec<String>,
}

impl Inventory {
    /// Walk `root` collecting relative, lowercased file paths of *implementation*
    /// (skips vendor/build dirs **and** the design sources `.specify/`+`specs/`,
    /// so a component is only "implemented" when real code references it — not
    /// when its own memory doc names it). Read-only.
    pub fn scan(root: &Path) -> Self {
        const SKIP: &[&str] = &[
            ".git",
            "target",
            "node_modules",
            "dist",
            "build",
            ".docusaurus",
            ".specify",
            "specs",
        ];
        let mut files = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let mut entries: Vec<_> = std::fs::read_dir(&dir)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
                .collect();
            entries.sort();
            for p in entries {
                if p.is_dir() {
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                    if !SKIP.contains(&name) {
                        stack.push(p);
                    }
                } else if let Ok(rel) = p.strip_prefix(root) {
                    files.push(rel.to_string_lossy().to_lowercase());
                }
            }
        }
        files.sort();
        Inventory { files }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingClass {
    IntendedNotImplemented,
    ConsumerFieldWithoutProducer,
    ContractShapeMismatch,
    ConsumerVsChangedDecision,
}

impl FindingClass {
    pub fn code(self) -> &'static str {
        match self {
            FindingClass::IntendedNotImplemented => "C1",
            FindingClass::ConsumerFieldWithoutProducer => "C2",
            FindingClass::ContractShapeMismatch => "C3",
            FindingClass::ConsumerVsChangedDecision => "C4",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Blocking,
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub id: String,
    pub class: FindingClass,
    pub severity: Severity,
    pub confidence: Confidence,
    pub message: String,
    pub locations: Vec<SourceRef>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CoherenceReport {
    pub findings: Vec<Finding>,
}

impl CoherenceReport {
    /// Build the report for the project at `root` (read-only).
    pub fn build(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref();
        let model = IntentModel::build(root);
        let inventory = Inventory::scan(root);
        analyze(&model, &inventory)
    }

    pub fn blocking_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Blocking)
            .count()
    }

    /// Keep only findings at or above `min` confidence.
    pub fn filtered(&self, min: Confidence) -> CoherenceReport {
        let rank = |c: Confidence| match c {
            Confidence::Low => 0,
            Confidence::Medium => 1,
            Confidence::High => 2,
        };
        CoherenceReport {
            findings: self
                .findings
                .iter()
                .filter(|f| rank(f.confidence) >= rank(min))
                .cloned()
                .collect(),
        }
    }
}

/// Pure reconciliation: `(IntentModel, Inventory)` → findings.
pub fn analyze(model: &IntentModel, inv: &Inventory) -> CoherenceReport {
    let mut findings = Vec::new();
    let mut n = 0usize;
    let mut next_id = |class: FindingClass| {
        n += 1;
        format!("{}-{:03}", class.code(), n)
    };

    // --- C1: intended component with no implementing files -----------------
    for comp in &model.intended_components {
        let implemented = inv.files.iter().any(|f| f.contains(&comp.id));
        if !implemented {
            findings.push(Finding {
                id: next_id(FindingClass::IntendedNotImplemented),
                class: FindingClass::IntendedNotImplemented,
                severity: Severity::Warning,
                confidence: Confidence::Medium,
                message: format!(
                    "component '{}' is designed in .specify/memory ({:?}) but no implementing files were found",
                    comp.label, comp.kind
                ),
                locations: comp
                    .sources
                    .iter()
                    .map(|s| SourceRef {
                        file: format!(".specify/memory/{s}"),
                        symbol: Some(comp.label.clone()),
                    })
                    .collect(),
            });
        }
    }

    // --- contract-level checks (C2/C3) -------------------------------------
    for c in &model.contracts {
        let Some(producer) = &c.producer else {
            continue;
        };
        let conf = contract_confidence(model, c);
        let producer_fields: BTreeSet<String> =
            producer.fields.iter().map(|f| f.name.to_lowercase()).collect();
        let producer_variants: BTreeSet<String> = producer
            .enums
            .iter()
            .flat_map(|e| e.variants.iter().cloned())
            .collect();

        for consumer in &c.consumers {
            // C2 — consumer fields with no producing counterpart.
            let orphans: Vec<String> = consumer
                .fields
                .iter()
                .filter(|f| !producer_fields.contains(&f.name.to_lowercase()))
                .map(|f| f.name.clone())
                .collect();
            if !orphans.is_empty() {
                findings.push(Finding {
                    id: next_id(FindingClass::ConsumerFieldWithoutProducer),
                    class: FindingClass::ConsumerFieldWithoutProducer,
                    severity: Severity::Blocking,
                    confidence: conf,
                    message: format!(
                        "contract '{}': consumer {} requires field(s) no producer provides: {}",
                        c.id,
                        consumer.source.file,
                        orphans.join(", ")
                    ),
                    locations: vec![consumer.source.clone(), producer.source.clone()],
                });
            }

            // C3 — disjoint enum value sets.
            let consumer_variants: BTreeSet<String> = consumer
                .enums
                .iter()
                .flat_map(|e| e.variants.iter().cloned())
                .collect();
            if !producer_variants.is_empty()
                && !consumer_variants.is_empty()
                && producer_variants.is_disjoint(&consumer_variants)
            {
                findings.push(Finding {
                    id: next_id(FindingClass::ContractShapeMismatch),
                    class: FindingClass::ContractShapeMismatch,
                    severity: Severity::Blocking,
                    confidence: conf,
                    message: format!(
                        "contract '{}': enum mismatch — producer {{{}}} vs consumer {{{}}}",
                        c.id,
                        join(&producer_variants),
                        join(&consumer_variants)
                    ),
                    locations: vec![consumer.source.clone(), producer.source.clone()],
                });
            }
        }
    }

    // --- C4: consumer spec depends on a contract a decision defined, without
    //         acknowledging that decision -----------------------------------
    for c in &model.contracts {
        if c.defined_by.is_empty() {
            continue;
        }
        // spec consumers come from provenance edges into spec.md files.
        for edge in model.provenance.iter().filter(|e| e.contract == c.id) {
            let Some(spec_id) = edge.consumer.symbol.as_deref() else {
                continue;
            };
            if !edge.consumer.file.ends_with("spec.md") {
                continue;
            }
            let referenced = model
                .specs
                .iter()
                .find(|s| s.id == spec_id)
                .map(|s| s.referenced_decisions.clone())
                .unwrap_or_default();
            for d in &c.defined_by {
                // the spec that *owns* the decision is exempt
                if d.location == spec_id {
                    continue;
                }
                if !referenced.iter().any(|r| r == &d.id) {
                    findings.push(Finding {
                        id: next_id(FindingClass::ConsumerVsChangedDecision),
                        class: FindingClass::ConsumerVsChangedDecision,
                        severity: Severity::Warning,
                        confidence: edge.confidence,
                        message: format!(
                            "spec '{}' consumes contract '{}' but never references its defining decision {} ({})",
                            spec_id, c.id, d.id, d.location
                        ),
                        locations: vec![edge.consumer.clone()],
                    });
                }
            }
        }
    }

    findings.sort_by(|a, b| a.id.cmp(&b.id));
    CoherenceReport { findings }
}

/// Best provenance-edge confidence for a contract (defaults to Low).
fn contract_confidence(model: &IntentModel, c: &IntentContract) -> Confidence {
    let rank = |c: Confidence| match c {
        Confidence::Low => 0,
        Confidence::Medium => 1,
        Confidence::High => 2,
    };
    model
        .provenance
        .iter()
        .filter(|e| e.contract == c.id)
        .map(|e| e.confidence)
        .max_by_key(|c| rank(*c))
        .unwrap_or(Confidence::Low)
}

fn join(set: &BTreeSet<String>) -> String {
    set.iter().cloned().collect::<Vec<_>>().join(", ")
}
