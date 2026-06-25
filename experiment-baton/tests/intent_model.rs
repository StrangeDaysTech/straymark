//! B2 integration tests: the IntentModel joins SpecKit intent, code shapes, and
//! decisions into contracts + provenance edges — calibrated on the #304 health
//! contract (Go producer ↔ TS consumer ↔ PM-002, one ContractId, High conf).

use std::path::PathBuf;

use straymark_baton::intent::{Confidence, IntentModel, ShapeRole};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample-project")
}

#[test]
fn health_contract_keys_producer_and_consumer_to_one_id() {
    let m = IntentModel::build(fixture_root());
    let c = m
        .contracts
        .iter()
        .find(|c| c.id == "services.health")
        .expect("the health endpoint must resolve to one ContractId");

    let producer = c.producer.as_ref().expect("Go producer shape");
    assert_eq!(producer.role, ShapeRole::Producer);
    assert!(producer.source.file.ends_with("handler.go"));
    let pfields: Vec<&str> = producer.fields.iter().map(|f| f.name.as_str()).collect();
    for expected in ["service_id", "state", "health_score", "name", "detail"] {
        assert!(pfields.contains(&expected), "producer missing {expected}");
    }
    let penums: Vec<String> = producer.enums.iter().flat_map(|e| e.variants.clone()).collect();
    assert!(penums.contains(&"OPERATIONAL".to_string()));

    let consumer = c
        .consumers
        .iter()
        .find(|s| s.source.file.ends_with("types.ts"))
        .expect("TS consumer shape");
    let cfields: Vec<&str> = consumer.fields.iter().map(|f| f.name.as_str()).collect();
    for expected in ["name", "status", "latency_p95_ms", "cpu", "memory"] {
        assert!(cfields.contains(&expected), "consumer missing {expected}");
    }
    let cenums: Vec<String> = consumer.enums.iter().flat_map(|e| e.variants.clone()).collect();
    assert!(cenums.contains(&"GREEN".to_string()));
}

#[test]
fn health_contract_is_defined_by_pm002() {
    let m = IntentModel::build(fixture_root());
    let c = m.contracts.iter().find(|c| c.id == "services.health").unwrap();
    assert!(
        c.defined_by.iter().any(|d| d.id == "PM-002"),
        "PM-002 should be linked as the defining decision"
    );
}

#[test]
fn high_confidence_edge_links_ts_consumer_to_go_producer_via_pm002() {
    let m = IntentModel::build(fixture_root());
    let edge = m
        .provenance
        .iter()
        .find(|e| e.contract == "services.health" && e.consumer.file.ends_with("types.ts"))
        .expect("a code-consumer provenance edge for the TS types");

    assert_eq!(
        edge.confidence,
        Confidence::High,
        "producer + code consumer + defining decision ⇒ High confidence"
    );
    assert!(edge.producer.as_ref().unwrap().file.ends_with("handler.go"));
    assert!(edge.defined_by.iter().any(|d| d.id == "PM-002"));
}

#[test]
fn spec_consumer_edge_exists_for_frontend() {
    // The frontend spec consumes the health endpoint but never references PM-002;
    // the edge still resolves the truth to the producer + decision (the #304 fix).
    let m = IntentModel::build(fixture_root());
    assert!(
        m.provenance.iter().any(|e| e.contract == "services.health"
            && e.consumer.file.contains("005-frontend")),
        "the frontend spec should appear as a consumer of the health contract"
    );
}
