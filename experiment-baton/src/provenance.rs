//! Provenance inference — joins contract shapes (code), spec consume hints, and
//! backlog decisions into `IntentContract`s and `ProvenanceEdge`s.
//!
//! The edge is the missing link of issue #304: it connects each *consumer* of a
//! contract to its *producer* and the *decision* that shaped it, so the
//! contract's source-of-truth is singular. Conservative confidence (R3/R6):
//! `High` only when a code producer and a code consumer share an endpoint and a
//! defining decision exists.

use std::collections::{BTreeMap, BTreeSet};

use crate::intent::{
    Confidence, ContractShape, DecisionKind, DecisionRef, IntentContract, ProvenanceEdge, ShapeRole,
    SourceRef,
};
use crate::scan::normalize_endpoint;
use crate::speckit::SpecKitArtifacts;

/// Infer the contracts and provenance edges for a project.
pub fn infer(
    artifacts: &SpecKitArtifacts,
    shapes: &[ContractShape],
) -> (Vec<IntentContract>, Vec<ProvenanceEdge>) {
    // --- spec side: endpoints, spec consumers, defining decisions -----------
    let mut endpoint_of: BTreeMap<String, String> = BTreeMap::new();
    let mut spec_consumers: BTreeMap<String, Vec<SourceRef>> = BTreeMap::new();
    let mut defined_by: BTreeMap<String, Vec<DecisionRef>> = BTreeMap::new();

    for spec in &artifacts.specs {
        let cids: BTreeSet<String> = spec
            .consumes
            .iter()
            .map(|c| normalize_endpoint(&c.endpoint))
            .filter(|c| !c.is_empty())
            .collect();

        for c in &spec.consumes {
            let cid = normalize_endpoint(&c.endpoint);
            if !cid.is_empty() {
                endpoint_of.entry(cid).or_insert_with(|| c.endpoint.clone());
            }
        }

        let spec_src = SourceRef {
            file: format!("specs/{}/spec.md", spec.id),
            symbol: Some(spec.id.clone()),
        };
        for cid in &cids {
            spec_consumers
                .entry(cid.clone())
                .or_default()
                .push(spec_src.clone());
        }

        // A decision defines only the contracts its *own section* names (precise
        // link). Falls back to nothing when the decision references no endpoint —
        // we never claim a decision defines a contract it doesn't mention.
        for d in &spec.decisions {
            for cid in &d.endpoints {
                if cid.is_empty() {
                    continue;
                }
                let bucket = defined_by.entry(cid.clone()).or_default();
                push_decision(bucket, DecisionKind::Pm, &d.id, &spec.id);
                for r in &d.references {
                    push_decision(bucket, DecisionKind::Ailog, r, &spec.id);
                }
            }
        }
    }

    // --- code side: producers and consumers keyed by contract ---------------
    let mut producers: BTreeMap<String, ContractShape> = BTreeMap::new();
    let mut code_consumers: BTreeMap<String, Vec<ContractShape>> = BTreeMap::new();
    for sh in shapes {
        match sh.role {
            ShapeRole::Producer => {
                producers.entry(sh.contract.clone()).or_insert_with(|| sh.clone());
            }
            ShapeRole::Consumer => {
                code_consumers.entry(sh.contract.clone()).or_default().push(sh.clone());
            }
        }
    }

    // --- assemble -----------------------------------------------------------
    let cids: BTreeSet<String> = producers
        .keys()
        .chain(code_consumers.keys())
        .chain(spec_consumers.keys())
        .chain(defined_by.keys())
        .cloned()
        .collect();

    let mut contracts = Vec::new();
    let mut edges = Vec::new();

    for cid in cids {
        let producer = producers.get(&cid).cloned();
        let consumers = code_consumers.get(&cid).cloned().unwrap_or_default();
        let dby = defined_by.get(&cid).cloned().unwrap_or_default();
        let has_producer = producer.is_some();
        let has_decision = !dby.is_empty();
        let producer_ref = producer.as_ref().map(|p| p.source.clone());

        contracts.push(IntentContract {
            id: cid.clone(),
            endpoint: endpoint_of.get(&cid).cloned(),
            producer: producer.clone(),
            consumers: consumers.clone(),
            defined_by: dby.clone(),
        });

        for c in &consumers {
            edges.push(ProvenanceEdge {
                consumer: c.source.clone(),
                contract: cid.clone(),
                producer: producer_ref.clone(),
                defined_by: dby.clone(),
                confidence: confidence(has_producer, true, has_decision),
            });
        }
        for sc in spec_consumers.get(&cid).cloned().unwrap_or_default() {
            edges.push(ProvenanceEdge {
                consumer: sc,
                contract: cid.clone(),
                producer: producer_ref.clone(),
                defined_by: dby.clone(),
                confidence: confidence(has_producer, false, has_decision),
            });
        }
    }

    edges.sort_by(|a, b| {
        (&a.contract, &a.consumer.file).cmp(&(&b.contract, &b.consumer.file))
    });
    (contracts, edges)
}

fn confidence(has_producer: bool, consumer_is_code: bool, has_decision: bool) -> Confidence {
    if has_producer && consumer_is_code && has_decision {
        Confidence::High
    } else if has_producer && (consumer_is_code || has_decision) {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

fn push_decision(bucket: &mut Vec<DecisionRef>, kind: DecisionKind, id: &str, location: &str) {
    if !bucket.iter().any(|d| d.kind == kind && d.id == id) {
        bucket.push(DecisionRef {
            kind,
            id: id.to_string(),
            location: location.to_string(),
        });
    }
}
