//! GH #415 — duplicate FU ids in the registry.
//!
//! Two entries answering to one id is recoverable state; what was not
//! recoverable was finding out. `note` wrote into whichever entry came first
//! and `set-status` answered `already closed — nothing to change`, which reads
//! as success while the intended follow-up stayed open and unannotated.
//!
//! These tests pin the three surfaces that close that: mutating commands refuse
//! an ambiguous id, `validate` reports the duplicate, and a pruned entry's
//! number is never handed out again.

use assert_cmd::cargo_bin_cmd;
use predicates::prelude::*;
use tempfile::TempDir;

const REGISTRY_WITH_DUPLICATES: &str = r#"---
schema_version: v1
last_scan: 2026-08-06
buckets: [ready]
fully_extracted_ailogs: []
total_open: 2
---

# Follow-ups Backlog

## Bucket: ready

### FU-007 — primera entrada, la que quedó cerrada
- **Origin**: AILOG-2026-08-06-003 §Follow-ups
- **Source-hash**: aaaaaaaaaaaa
- **Status**: closed

### FU-007 — segunda entrada, distinta, que el operador quería anotar
- **Origin**: AILOG-2026-08-06-004 §Follow-ups
- **Source-hash**: bbbbbbbbbbbb
- **Status**: open
"#;

fn project_with(registry: &str) -> TempDir {
    let tmp = TempDir::new().unwrap();
    let sm = tmp.path().join(".straymark");
    std::fs::create_dir_all(&sm).unwrap();
    std::fs::write(sm.join("config.yml"), "language: en\n").unwrap();
    std::fs::write(tmp.path().join("STRAYMARK.md"), "# rules\n").unwrap();
    std::fs::write(sm.join("follow-ups-backlog.md"), registry).unwrap();
    tmp
}

/// The exact sequence from the report: the note landed on the wrong entry and
/// `set-status` reported a no-op that read as success.
#[test]
fn mutating_commands_refuse_an_ambiguous_id() {
    let tmp = project_with(REGISTRY_WITH_DUPLICATES);
    let before = std::fs::read_to_string(tmp.path().join(".straymark/follow-ups-backlog.md")).unwrap();

    for args in [
        vec!["followups", "note", "FU-007", "la medición"],
        vec!["followups", "set-status", "FU-007", "closed"],
    ] {
        cargo_bin_cmd!("straymark")
            .args(&args)
            .args(["--path", tmp.path().to_str().unwrap()])
            .assert()
            .failure()
            .stderr(predicate::str::contains("ambiguous"))
            // Both entries named, so the operator can tell which is which.
            .stderr(predicate::str::contains("AILOG-2026-08-06-003"))
            .stderr(predicate::str::contains("AILOG-2026-08-06-004"));
    }

    let after = std::fs::read_to_string(tmp.path().join(".straymark/follow-ups-backlog.md")).unwrap();
    assert_eq!(
        before, after,
        "a refused command must not have written anything"
    );
}

/// Reading is refused too: showing one of the two is how an operator concludes
/// the wrong entry is the one they are about to act on.
#[test]
fn status_detail_refuses_an_ambiguous_id() {
    let tmp = project_with(REGISTRY_WITH_DUPLICATES);
    cargo_bin_cmd!("straymark")
        .args(["followups", "status", "FU-007", "--path"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("ambiguous"));
}

#[test]
fn validate_reports_duplicate_ids_as_an_error() {
    let tmp = project_with(REGISTRY_WITH_DUPLICATES);
    cargo_bin_cmd!("straymark")
        .args(["validate"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("FOLLOWUP-DUPLICATE-ID"))
        .stdout(predicate::str::contains("FU-007"));
}

/// A registry with unique ids must stay silent — the rule keys on headings, so
/// a `FU-NNN` cited in prose or in `Notes` is not a second entry.
#[test]
fn a_clean_registry_reports_nothing() {
    let clean = REGISTRY_WITH_DUPLICATES
        .replacen("### FU-007 — segunda", "### FU-008 — segunda", 1)
        + "\n### FU-009 — cita a FU-007 y FU-008 en su cuerpo\n- **Status**: open\n";
    let tmp = project_with(&clean);
    cargo_bin_cmd!("straymark")
        .args(["validate"])
        .arg(tmp.path())
        .assert()
        .stdout(predicate::str::contains("FOLLOWUP-DUPLICATE-ID").not());
}

/// The id of an entry triage pruned down to a provenance bullet must stay
/// retired. Reported from Sentinel on 2026-06-04, before #415 restated it.
#[test]
fn a_pruned_id_is_never_handed_out_again() {
    let pruned = r#"---
schema_version: v1
last_scan: 2026-08-06
buckets: [ready]
fully_extracted_ailogs: []
total_open: 1
---

# Follow-ups Backlog

## Bucket: ready

### FU-001 — la que sigue viva
- **Origin**: AILOG-2026-08-06-001 §Follow-ups
- **Status**: open

## Closed at triage

- FU-002 (closed 2026-08-06) — pruned to a provenance bullet, heading gone
"#;
    let tmp = project_with(pruned);
    std::fs::create_dir_all(tmp.path().join(".straymark/07-ai-audit/agent-logs")).unwrap();
    std::fs::write(
        tmp.path().join(".straymark/07-ai-audit/agent-logs/AILOG-2026-08-06-009-x.md"),
        "---\nid: AILOG-2026-08-06-009\ntitle: x\nstatus: accepted\ncreated: 2026-08-06\n\
         agent: claude-opus-5-v1.0\nconfidence: high\nreview_required: false\nrisk_level: low\n---\n\n\
         # AILOG: x\n\n## Summary\ns\n\n## Follow-ups\n\n- (new) una entrada nueva que no debe reusar FU-002\n",
    )
    .unwrap();

    cargo_bin_cmd!("straymark")
        .args(["followups", "drift", "--apply", "--scan-all", "--path"])
        .arg(tmp.path())
        .assert()
        .success();

    let registry =
        std::fs::read_to_string(tmp.path().join(".straymark/follow-ups-backlog.md")).unwrap();
    assert!(
        !registry.contains("### FU-002 "),
        "FU-002 was pruned, not freed — reassigning it collides with the pruned record:\n{registry}"
    );
    assert!(
        registry.contains("### FU-003 "),
        "the new entry should take the next number past the high-water mark:\n{registry}"
    );
}
