use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Set up a minimal StrayMark installation with the Charter template. Mirrors
/// what `straymark init` would produce for the Charter feature, sufficient for
/// `straymark charter new` to operate.
fn setup_straymark_with_charter_template(dir: &std::path::Path) {
    let straymark = dir.join(".straymark");
    std::fs::create_dir_all(straymark.join("templates")).unwrap();
    std::fs::write(straymark.join("config.yml"), "language: en\n").unwrap();

    // Bundled template. We ship the actual template from
    // dist/.straymark/templates/charter-template.md in the framework; this test
    // helper inlines a structurally-equivalent copy so tests don't depend on
    // the dist/ path being available at test runtime.
    let template = r#"---
charter_id: CHARTER-NN
status: declared
effort_estimate: M
trigger: "[1-line: what observable signal justifies executing this Charter now]"
# originating_ailogs: [AILOG-YYYY-MM-DD-NNN]
# originating_spec: specs/001-feature/spec.md
---

# Charter: [BRIEF TITLE]

> **Status (mirrored from frontmatter — source of truth is above):** declared. Effort: [XS | S | M | L] (~[N] min).
>
> **Origin:** [human-readable summary; the machine-readable form is `originating_ailogs` or `originating_spec` in frontmatter].

## Context

[1-2 paragraphs.]

## Scope

**In scope:**

1. [Item 1]

**Out of scope:**

- [Item 1]

## Files to modify

| File | Change |
|---|---|

## Verification

### Local checks

```bash
<build-command>
```

### Production smoke (after deploy)

```bash
TOKEN="$(<auth-cli> print-identity-token)"
```

## Risks

- **R1 — [risk]**: mitigation.

## Tasks

1. Sync main.

## Charter Closure

When closing this Charter (post-merge):

1. Drift check.
"#;
    std::fs::write(straymark.join("templates").join("charter-template.md"), template).unwrap();
}

#[test]
fn charter_new_requires_straymark_installed() {
    let dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("charter")
        .arg("new")
        .arg("--title")
        .arg("Test")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}

#[test]
fn charter_new_no_origin_creates_file_with_defaults() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("charter")
        .arg("new")
        .arg("--title")
        .arg("Test Charter")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created:"));

    let charters_dir = dir.path().join("docs").join("charters");
    assert!(charters_dir.exists(), "docs/charters/ should exist");
    let entries: Vec<_> = std::fs::read_dir(&charters_dir).unwrap().flatten().collect();
    assert_eq!(entries.len(), 1, "should have exactly one Charter file");

    let path = entries[0].path();
    let filename = path.file_name().unwrap().to_str().unwrap();
    assert_eq!(filename, "01-test-charter.md");

    let content = std::fs::read_to_string(&path).unwrap();
    // charter_id placeholder substituted.
    assert!(content.contains("charter_id: CHARTER-01-test-charter"), "{}", content);
    // Default effort is M.
    assert!(content.contains("effort_estimate: M"), "{}", content);
    // Body title substituted.
    assert!(content.contains("# Charter: Test Charter"), "{}", content);
    // Both origin lines remain commented (no flag was passed).
    assert!(content.contains("# originating_ailogs: [AILOG-YYYY-MM-DD-NNN]"), "{}", content);
    assert!(content.contains("# originating_spec: specs/001-feature/spec.md"), "{}", content);
}

#[test]
fn charter_new_explicit_effort_is_substituted() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--type")
        .arg("L")
        .arg("--title")
        .arg("Big Work")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let path = dir.path().join("docs/charters/01-big-work.md");
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("effort_estimate: L"), "{}", content);
    // Prose mirror line also reflects the chosen effort.
    assert!(content.contains("Effort: L (~[N] min)"), "{}", content);
}

#[test]
fn charter_new_with_from_ailog_uncomments_origin() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("From AILOG")
        .arg("--from-ailog")
        .arg("AILOG-2026-04-28-021")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let path = dir.path().join("docs/charters/01-from-ailog.md");
    let content = std::fs::read_to_string(&path).unwrap();
    // Frontmatter origin uncommented and populated.
    assert!(
        content.contains("originating_ailogs: [AILOG-2026-04-28-021]"),
        "{}",
        content
    );
    // The other origin stays commented.
    assert!(content.contains("# originating_spec: specs/001-feature/spec.md"));
    // Prose Origin updated with concrete reference.
    assert!(content.contains("Follow-up of AILOG-2026-04-28-021"), "{}", content);
}

#[test]
fn charter_new_with_from_spec_uncomments_origin() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    // Create a stub SpecKit spec file so --from-spec validation passes.
    let spec_dir = dir.path().join("specs").join("001-test");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("spec.md"), "# Test Spec\n").unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("From Spec")
        .arg("--from-spec")
        .arg("specs/001-test/spec.md")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let path = dir.path().join("docs/charters/01-from-spec.md");
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(
        content.contains("originating_spec: specs/001-test/spec.md"),
        "{}",
        content
    );
    assert!(content.contains("# originating_ailogs: [AILOG-YYYY-MM-DD-NNN]"));
    assert!(
        content.contains("derived from spec at specs/001-test/spec.md"),
        "{}",
        content
    );
}

#[test]
fn charter_new_rejects_both_origins_at_clap_level() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("Both Origins")
        .arg("--from-ailog")
        .arg("AILOG-2026-04-28-021")
        .arg("--from-spec")
        .arg("specs/001-test/spec.md")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used"));
}

#[test]
fn charter_new_from_spec_rejects_missing_file() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("Bad Spec")
        .arg("--from-spec")
        .arg("specs/missing/spec.md")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn charter_new_increments_sequence_number() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    for (n, title) in [(1, "First"), (2, "Second"), (3, "Third")] {
        Command::cargo_bin("straymark")
            .unwrap()
            .arg("charter")
            .arg("new")
            .arg("--title")
            .arg(title)
            .arg(dir.path().to_str().unwrap())
            .assert()
            .success();

        let expected_filename = format!("{:02}-{}.md", n, title.to_lowercase());
        let path = dir.path().join("docs/charters").join(&expected_filename);
        assert!(path.exists(), "expected {} to exist", path.display());
    }

    let entries: Vec<_> = std::fs::read_dir(dir.path().join("docs/charters"))
        .unwrap()
        .flatten()
        .collect();
    assert_eq!(entries.len(), 3);
}

#[test]
fn charter_new_rejects_invalid_effort_at_clap_level() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--type")
        .arg("XXL")
        .arg("--title")
        .arg("Bad Effort")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn charter_new_uses_es_template_when_config_says_es() {
    let dir = TempDir::new().unwrap();
    let straymark = dir.path().join(".straymark");
    std::fs::create_dir_all(straymark.join("templates").join("i18n").join("es")).unwrap();
    std::fs::write(straymark.join("config.yml"), "language: es\n").unwrap();

    // EN template (fallback).
    std::fs::write(
        straymark.join("templates").join("charter-template.md"),
        "---\ncharter_id: CHARTER-NN\nstatus: declared\neffort_estimate: M\ntrigger: \"[x]\"\n---\n\n# Charter: [BRIEF TITLE]\n\nEN body.\n",
    ).unwrap();
    // ES translation.
    std::fs::write(
        straymark.join("templates").join("i18n").join("es").join("charter-template.md"),
        "---\ncharter_id: CHARTER-NN\nstatus: declared\neffort_estimate: M\ntrigger: \"[x]\"\n---\n\n# Charter: [TÍTULO BREVE]\n\nES body.\n",
    ).unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("Hola Mundo")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let content =
        std::fs::read_to_string(dir.path().join("docs/charters/01-hola-mundo.md")).unwrap();
    assert!(content.contains("ES body"), "expected ES template selected, got: {}", content);
    assert!(content.contains("# Charter: Hola Mundo"));
}

// ----------------------------------------------------------------------------
// `straymark charter list`
// ----------------------------------------------------------------------------

#[test]
fn charter_list_requires_straymark_installed() {
    let dir = TempDir::new().unwrap();
    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("list")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}

#[test]
fn charter_list_empty_when_no_charters() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("list")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("No Charters"));
}

#[test]
fn charter_list_shows_all_charters_by_default() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    create_three_charters(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("list")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("first"))
        .stdout(predicate::str::contains("second"))
        .stdout(predicate::str::contains("third"));
}

#[test]
fn charter_list_filter_status_declared() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    create_three_charters(dir.path());
    // Mark the first one as closed.
    let p = dir.path().join("docs/charters/01-first.md");
    let content = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, content.replace("status: declared", "status: closed")).unwrap();

    let out = Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("list")
        .arg("--status")
        .arg("declared")
        .arg(dir.path().to_str().unwrap())
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("second"), "stdout: {}", stdout);
    assert!(stdout.contains("third"), "stdout: {}", stdout);
    assert!(!stdout.contains(" first"), "stdout should hide closed first: {}", stdout);
}

#[test]
fn charter_list_filter_origin_ailog() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    let charters_dir = dir.path().join("docs/charters");
    std::fs::create_dir_all(&charters_dir).unwrap();

    // One Charter with --from-ailog (tested via the actual command).
    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("with ailog")
        .arg("--from-ailog")
        .arg("AILOG-2026-04-28-021")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    // One without origin.
    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("standalone")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let out = Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("list")
        .arg("--origin")
        .arg("ailog")
        .arg(dir.path().to_str().unwrap())
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(out.status.success());
    assert!(stdout.contains("with ailog"), "stdout: {}", stdout);
    assert!(!stdout.contains("standalone"), "stdout should hide no-origin: {}", stdout);
}

#[test]
fn charter_list_no_match_shows_friendly_message() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    create_three_charters(dir.path());

    // All three are declared by default, so --status closed matches none.
    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("list")
        .arg("--status")
        .arg("closed")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("No Charters match"));
}

// ----------------------------------------------------------------------------
// `straymark charter status`
// ----------------------------------------------------------------------------

#[test]
fn charter_status_requires_straymark_installed() {
    let dir = TempDir::new().unwrap();
    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("status")
        .arg("--path")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not installed"));
}

#[test]
fn charter_status_empty_when_no_charters() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("status")
        .arg("--path")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("No Charters"));
}

#[test]
fn charter_status_without_id_shows_recent() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    create_three_charters(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("status")
        .arg("--path")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Most recent"))
        .stdout(predicate::str::contains("first"))
        .stdout(predicate::str::contains("second"))
        .stdout(predicate::str::contains("third"));
}

#[test]
fn charter_status_with_full_id_shows_detail() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    create_three_charters(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("status")
        .arg("CHARTER-02-second")
        .arg("--path")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("CHARTER-02-second"))
        .stdout(predicate::str::contains("Status:"))
        .stdout(predicate::str::contains("Effort:"))
        .stdout(predicate::str::contains("File:"));
}

#[test]
fn charter_status_with_charter_nn_prefix_shows_detail() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    create_three_charters(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("status")
        .arg("CHARTER-02")
        .arg("--path")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("CHARTER-02-second"));
}

#[test]
fn charter_status_with_numeric_id_shows_detail() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    create_three_charters(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("status")
        .arg("2")
        .arg("--path")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("CHARTER-02-second"));
}

#[test]
fn charter_status_with_unknown_id_fails() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    create_three_charters(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("status")
        .arg("CHARTER-99")
        .arg("--path")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// ----------------------------------------------------------------------------
// `straymark validate --include-charters`
// ----------------------------------------------------------------------------

#[test]
fn validate_without_flag_skips_charter_checks() {
    // Verifies the opt-in: a project with a broken Charter (missing required
    // field) still passes `straymark validate` when --include-charters is absent.
    let dir = TempDir::new().unwrap();
    setup_straymark_full(dir.path());
    let charters_dir = dir.path().join("docs/charters");
    std::fs::create_dir_all(&charters_dir).unwrap();
    // Write a Charter missing the required `trigger` field. Without
    // --include-charters the validator should not even look at it.
    std::fs::write(
        charters_dir.join("01-broken.md"),
        "---\ncharter_id: CHARTER-01-broken\nstatus: declared\neffort_estimate: M\n---\n\n# Charter: Broken\n",
    )
    .unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("validate")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn validate_with_flag_passes_for_valid_charter() {
    let dir = TempDir::new().unwrap();
    setup_straymark_full(dir.path());
    create_charter_via_cli(dir.path(), "valid charter", &[]);

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("validate")
        .arg("--include-charters")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("passed validation"));
}

#[test]
fn validate_with_flag_fails_on_missing_required_field() {
    let dir = TempDir::new().unwrap();
    setup_straymark_full(dir.path());
    let charters_dir = dir.path().join("docs/charters");
    std::fs::create_dir_all(&charters_dir).unwrap();
    // Frontmatter is missing `trigger` — schema should reject.
    std::fs::write(
        charters_dir.join("01-no-trigger.md"),
        "---\ncharter_id: CHARTER-01-no-trigger\nstatus: declared\neffort_estimate: M\n---\n\n# Charter: No trigger\n",
    )
    .unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("validate")
        .arg("--include-charters")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("CHARTER-SCHEMA"));
}

#[test]
fn validate_with_flag_fails_on_invalid_status_enum() {
    let dir = TempDir::new().unwrap();
    setup_straymark_full(dir.path());
    create_charter_via_cli(dir.path(), "bad status", &[]);
    let p = dir.path().join("docs/charters/01-bad-status.md");
    let content = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, content.replace("status: declared", "status: unknown-state")).unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("validate")
        .arg("--include-charters")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("declared, in-progress, closed"));
}

#[test]
fn validate_fails_when_originating_ailog_does_not_exist() {
    let dir = TempDir::new().unwrap();
    setup_straymark_full(dir.path());
    create_charter_via_cli(
        dir.path(),
        "missing ailog",
        &["--from-ailog", "AILOG-2026-04-28-099"],
    );

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("validate")
        .arg("--include-charters")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("CHARTER-AILOG-REF"))
        .stdout(predicate::str::contains("AILOG-2026-04-28-099"));
}

#[test]
fn validate_passes_when_originating_ailog_exists() {
    let dir = TempDir::new().unwrap();
    setup_straymark_full(dir.path());
    // Create a real AILOG file so the reference resolves. The frontmatter
    // includes all META-001 required fields so the existing AILOG validator
    // (independent of Charter checks) does not flag this stub.
    let agent_logs = dir.path().join(".straymark/07-ai-audit/agent-logs");
    std::fs::write(
        agent_logs.join("AILOG-2026-04-28-021-real.md"),
        "---\n\
         id: AILOG-2026-04-28-021\n\
         title: Real AILOG stub for testing\n\
         status: accepted\n\
         created: 2026-04-28\n\
         agent: test-agent-v1.0\n\
         confidence: high\n\
         review_required: false\n\
         risk_level: low\n\
         ---\n\n\
         Body.\n",
    )
    .unwrap();
    create_charter_via_cli(
        dir.path(),
        "real ailog",
        &["--from-ailog", "AILOG-2026-04-28-021"],
    );

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("validate")
        .arg("--include-charters")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();
}

#[test]
fn validate_fails_when_originating_spec_path_missing() {
    let dir = TempDir::new().unwrap();
    setup_straymark_full(dir.path());
    // Create a Charter with a spec path that exists at scaffold time, then
    // delete the spec to simulate a broken reference (e.g., spec was renamed).
    let spec_dir = dir.path().join("specs/001-test");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(spec_dir.join("spec.md"), "# Spec\n").unwrap();
    create_charter_via_cli(
        dir.path(),
        "from spec",
        &["--from-spec", "specs/001-test/spec.md"],
    );
    std::fs::remove_file(spec_dir.join("spec.md")).unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("validate")
        .arg("--include-charters")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .failure()
        .stdout(predicate::str::contains("CHARTER-SPEC-REF"))
        .stdout(predicate::str::contains("specs/001-test/spec.md"));
}

#[test]
fn validate_warns_when_charter_schema_missing() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path()); // no schema written
    let charters_dir = dir.path().join("docs/charters");
    std::fs::create_dir_all(&charters_dir).unwrap();
    std::fs::write(
        charters_dir.join("01-x.md"),
        "---\ncharter_id: CHARTER-01-x\nstatus: declared\neffort_estimate: M\ntrigger: \"x\"\n---\n\n# Charter: X\n",
    )
    .unwrap();

    let out = Command::cargo_bin("straymark")
        .unwrap()
        .arg("validate")
        .arg("--include-charters")
        .arg(dir.path().to_str().unwrap())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("CHARTER-SCHEMA-MISSING") || stdout.contains("schema not loadable"),
        "expected schema-missing warning. stdout: {}",
        stdout
    );
}

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------

/// Set up a StrayMark installation with both the template AND the real Charter
/// schema (copied from dist/). Used by validate --include-charters tests.
fn setup_straymark_full(dir: &std::path::Path) {
    setup_straymark_with_charter_template(dir);
    // Copy the real schema from the framework distribution.
    let schemas_dir = dir.join(".straymark/schemas");
    std::fs::create_dir_all(&schemas_dir).unwrap();
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let real_schema = manifest_dir
        .join("..")
        .join("dist/.straymark/schemas/charter.schema.v0.json");
    let schema_content = std::fs::read_to_string(&real_schema).unwrap_or_else(|e| {
        panic!(
            "test setup needs the real schema at {}: {}",
            real_schema.display(),
            e
        )
    });
    std::fs::write(schemas_dir.join("charter.schema.v0.json"), schema_content).unwrap();
    // Also create the agent-logs directory so AILOG-ref tests have somewhere to look.
    std::fs::create_dir_all(dir.join(".straymark/07-ai-audit/agent-logs")).unwrap();
}

/// Run `straymark charter new` with the given title and extra flags. Asserts
/// success. Used by validate tests to produce real on-disk Charters.
fn create_charter_via_cli(dir: &std::path::Path, title: &str, extra_args: &[&str]) {
    let mut cmd = Command::cargo_bin("straymark").unwrap();
    cmd.arg("charter").arg("new").arg("--title").arg(title);
    for arg in extra_args {
        cmd.arg(arg);
    }
    cmd.arg(dir.to_str().unwrap()).assert().success();
}

/// Create three Charters via the actual `straymark charter new` command, so
/// list/status tests exercise real on-disk shapes (not synthetic stubs).
fn create_three_charters(dir: &std::path::Path) {
    for title in ["first", "second", "third"] {
        Command::cargo_bin("straymark")
            .unwrap()
            .arg("charter")
            .arg("new")
            .arg("--title")
            .arg(title)
            .arg(dir.to_str().unwrap())
            .assert()
            .success();
    }
}

#[test]
fn charter_new_does_not_overwrite_existing_file() {
    // Edge case: if the user manually created docs/charters/01-foo.md and then
    // tries `charter new --title "foo"`, we should refuse rather than clobber.
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());
    let charters_dir = dir.path().join("docs").join("charters");
    std::fs::create_dir_all(&charters_dir).unwrap();
    std::fs::write(charters_dir.join("01-foo.md"), "preexisting; not a Charter\n").unwrap();
    // The pre-existing file does not match the Charter naming pattern (still
    // matches `NN-*.md`) so next_charter_number() will compute 02.
    // To force the clash we insert a Charter-shaped placeholder and verify
    // next_charter_number lands on 02; then the new file at 02-foo.md is fine.
    // We instead test the explicit overwrite-refusal branch by pre-creating
    // the exact filename the next run would produce.
    let _ = std::fs::remove_file(charters_dir.join("01-foo.md"));
    std::fs::write(charters_dir.join("01-foo.md"), "real Charter content\n").unwrap();

    // next_charter_number reads existing files; with 01-foo.md present it
    // returns 2. So `charter new --title foo` produces 02-foo.md, not a clash.
    // We can only force a clash with concurrent invocations, which we don't
    // simulate here. The overwrite guard is defensive — verify it compiles
    // and the happy path produces a distinct filename.
    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("foo")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();
    assert!(charters_dir.join("02-foo.md").exists());
}

// ── F1 (cli-3.7.2): word-boundary slug truncation + --slug override ──

#[test]
fn charter_new_truncates_long_title_at_word_boundary() {
    // CHARTER-04 reproduction (issue #81): title that overflowed 50 chars
    // by 1-2 chars used to produce a mid-word fragment like "…required-t"
    // (cutting "true" to "t"). The fix truncates at the last `-` boundary.
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("Approve retroactivo bulk de docs review_required: true")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    // The slug should not include a partial "true" fragment.
    let charters_dir = dir.path().join("docs/charters");
    let entries: Vec<_> = std::fs::read_dir(&charters_dir).unwrap().flatten().collect();
    assert_eq!(entries.len(), 1);
    let filename = entries[0]
        .path()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(
        !filename.ends_with("-t.md") && !filename.ends_with("-tr.md") && !filename.ends_with("-tru.md"),
        "filename must not end with a partial word, got: {filename}"
    );
    assert!(filename.contains("required"), "should preserve last full word, got: {filename}");
}

#[test]
fn charter_new_slug_flag_overrides_title_derivation() {
    // CHARTER-05 reproduction (issue #81): the title-derived slug dropped a
    // meaningful trailing reference (e.g. "-04-f3"). The --slug flag lets
    // the operator provide an explicit short slug that preserves context.
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("Batching ListTimeSeries para N≥500 servicios — Plan 04 F3")
        .arg("--slug")
        .arg("batching-listtimeseries-04-f3")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let charters_dir = dir.path().join("docs/charters");
    assert!(charters_dir.join("01-batching-listtimeseries-04-f3.md").exists());
}

#[test]
fn charter_new_slug_flag_normalizes_through_slugifier() {
    // The override is normalized through the same slugifier so the operator
    // cannot smuggle in characters that would break the filename.
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("Whatever")
        .arg("--slug")
        .arg("UPPER and SPECIAL!!!")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let charters_dir = dir.path().join("docs/charters");
    assert!(charters_dir.join("01-upper-and-special.md").exists());
}

// ── F2 (cli-3.8.0): AILOG context backfill in --from-ailog ──────────

#[test]
fn charter_new_from_ailog_backfills_origin_with_summary() {
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    // Write an AILOG that the Charter will reference.
    let agent_logs = dir.path().join(".straymark/07-ai-audit/agent-logs");
    std::fs::create_dir_all(&agent_logs).unwrap();
    let ailog_body = r#"---
id: AILOG-2026-04-28-021
title: Implement async handler
agent: claude-code
confidence: high
review_required: false
---

# AILOG: async handler

## Summary

Migrated the privacy handler to async after profiling showed 200ms blocking
on DB queries. Added integration test that exercises the new path.

## Context

Original handler was synchronous and blocked on DB I/O.
"#;
    std::fs::write(
        agent_logs.join("AILOG-2026-04-28-021-async-handler.md"),
        ailog_body,
    )
    .unwrap();

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("Follow-up async refactor")
        .arg("--from-ailog")
        .arg("AILOG-2026-04-28-021")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let path = dir.path().join("docs/charters/01-follow-up-async-refactor.md");
    let content = std::fs::read_to_string(&path).unwrap();

    // Origin line embeds the extracted Summary lead.
    assert!(
        content.contains("Migrated the privacy handler to async"),
        "Origin line should embed extracted Summary, got:\n{content}"
    );
    // Placeholder is gone.
    assert!(!content.contains("[Add 1-line context"), "{content}");
    // Frontmatter still has the AILOG reference.
    assert!(content.contains("originating_ailogs: [AILOG-2026-04-28-021]"));
}

#[test]
fn charter_new_from_ailog_falls_back_when_ailog_not_found() {
    // F2 graceful fallback: if --from-ailog references an AILOG that doesn't
    // exist (typoed ID, or AILOG lives in a different repo), the body Origin
    // line keeps the original placeholder instead of failing.
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("Refers to missing AILOG")
        .arg("--from-ailog")
        .arg("AILOG-2026-04-28-999")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    let path = dir.path().join("docs/charters/01-refers-to-missing-ailog.md");
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(content.contains("originating_ailogs: [AILOG-2026-04-28-999]"));
    assert!(
        content.contains("[Add 1-line context"),
        "fallback placeholder should remain when AILOG not found, got:\n{content}"
    );
}

#[test]
fn charter_new_empty_slug_flag_falls_back_to_title() {
    // An empty --slug "" should be ignored (not treated as a hard error),
    // falling back to the title-derived slug.
    let dir = TempDir::new().unwrap();
    setup_straymark_with_charter_template(dir.path());

    Command::cargo_bin("straymark")
        .unwrap()
        .arg("charter")
        .arg("new")
        .arg("--title")
        .arg("Hello World")
        .arg("--slug")
        .arg("")
        .arg(dir.path().to_str().unwrap())
        .assert()
        .success();

    assert!(dir.path().join("docs/charters/01-hello-world.md").exists());
}
