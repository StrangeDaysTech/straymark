//! Sanity tests for the audit-related skills shipped under `dist/`.
//!
//! These tests verify that the skill files exist and have the expected
//! frontmatter shape (Claude has `allowed-tools`, Gemini has `name` but no
//! `allowed-tools`, agent workflow has only `description`). They run
//! against the source tree, not against an `init`-ed project, because the
//! manifest already includes the parent directories recursively — if the
//! files exist in `dist/`, `init` will copy them.

use std::path::PathBuf;

fn dist_root() -> PathBuf {
    // tests/ is at cli/tests/, dist/ is at the repo root.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("..").join("dist")
}

fn read(path: PathBuf) -> String {
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("expected file to exist: {}", path.display()))
}

#[test]
fn straymark_audit_prompt_claude_skill_exists_and_has_allowed_tools() {
    let body = read(
        dist_root()
            .join(".claude")
            .join("skills")
            .join("straymark-audit-prompt")
            .join("SKILL.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        body.contains("name: straymark-audit-prompt"),
        "missing name field"
    );
    assert!(
        body.contains("allowed-tools:"),
        "Claude skill must declare allowed-tools"
    );
    assert!(
        body.contains("straymark charter audit"),
        "skill body must reference the CLI command it wraps"
    );
}

#[test]
fn straymark_audit_prompt_gemini_skill_exists_without_allowed_tools() {
    let body = read(
        dist_root()
            .join(".gemini")
            .join("skills")
            .join("straymark-audit-prompt")
            .join("SKILL.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        body.contains("name: straymark-audit-prompt"),
        "missing name field"
    );
    assert!(
        !body.contains("allowed-tools:"),
        "Gemini skill must not declare allowed-tools (no such field in Gemini skill schema)"
    );
}

#[test]
fn straymark_audit_prompt_agent_workflow_exists_with_description_only() {
    let body = read(
        dist_root()
            .join(".agent")
            .join("workflows")
            .join("straymark-audit-prompt.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        !body.contains("name:"),
        "agent workflow must not declare a name field (description-only frontmatter)"
    );
    assert!(
        !body.contains("allowed-tools:"),
        "agent workflow must not declare allowed-tools"
    );
    assert!(
        body.contains("description:"),
        "agent workflow must declare description"
    );
}

#[test]
fn straymark_audit_prompt_three_platforms_share_core_guidance() {
    let claude = read(
        dist_root()
            .join(".claude")
            .join("skills")
            .join("straymark-audit-prompt")
            .join("SKILL.md"),
    );
    let gemini = read(
        dist_root()
            .join(".gemini")
            .join("skills")
            .join("straymark-audit-prompt")
            .join("SKILL.md"),
    );
    let agent = read(
        dist_root()
            .join(".agent")
            .join("workflows")
            .join("straymark-audit-prompt.md"),
    );

    // v1: skill no longer surfaces prompts inline. It runs --prepare and
    // points the operator at /straymark-audit-execute in N CLIs. The
    // wait-for-all warning is the load-bearing UX guarantee.
    for body in [&claude, &gemini, &agent] {
        assert!(
            body.contains("/straymark-audit-execute"),
            "skill must point operator at the auditor-side execute skill"
        );
        assert!(
            body.contains("/straymark-audit-review"),
            "skill must point operator at the follow-up review skill"
        );
        assert!(
            body.contains(".straymark/audits/")
                && body.contains("audit-prompt.md"),
            "skill must reference the v1 canonical prompt location"
        );
        assert!(
            body.contains("--prepare"),
            "skill must invoke `straymark charter audit ... --prepare`"
        );
        assert!(
            body.contains("ALL audits") && body.contains("complete"),
            "skill must include the wait-for-all warning before review"
        );
        assert!(
            body.contains("DIFFERENT model families")
                || body.contains("different model families"),
            "skill must surface the heterogeneity inter-family recommendation"
        );
    }
}

// ── straymark-audit-review (PR 2) ───────────────────────────────────────────

#[test]
fn straymark_audit_review_claude_skill_exists_and_has_allowed_tools() {
    let body = read(
        dist_root()
            .join(".claude")
            .join("skills")
            .join("straymark-audit-review")
            .join("SKILL.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        body.contains("name: straymark-audit-review"),
        "missing name field"
    );
    assert!(
        body.contains("allowed-tools:"),
        "Claude skill must declare allowed-tools"
    );
    assert!(
        body.contains("--merge-into"),
        "skill body must reference the CLI flag it wraps"
    );
}

#[test]
fn straymark_audit_review_gemini_skill_exists_without_allowed_tools() {
    let body = read(
        dist_root()
            .join(".gemini")
            .join("skills")
            .join("straymark-audit-review")
            .join("SKILL.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        body.contains("name: straymark-audit-review"),
        "missing name field"
    );
    assert!(
        !body.contains("allowed-tools:"),
        "Gemini skill must not declare allowed-tools"
    );
}

#[test]
fn straymark_audit_review_agent_workflow_exists_with_description_only() {
    let body = read(
        dist_root()
            .join(".agent")
            .join("workflows")
            .join("straymark-audit-review.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        !body.contains("name:"),
        "agent workflow must not declare a name field"
    );
    assert!(
        !body.contains("allowed-tools:"),
        "agent workflow must not declare allowed-tools"
    );
    assert!(
        body.contains("description:"),
        "agent workflow must declare description"
    );
}

#[test]
fn straymark_audit_review_three_platforms_share_core_guidance() {
    let claude = read(
        dist_root()
            .join(".claude")
            .join("skills")
            .join("straymark-audit-review")
            .join("SKILL.md"),
    );
    let gemini = read(
        dist_root()
            .join(".gemini")
            .join("skills")
            .join("straymark-audit-review")
            .join("SKILL.md"),
    );
    let agent = read(
        dist_root()
            .join(".agent")
            .join("workflows")
            .join("straymark-audit-review.md"),
    );

    for body in [&claude, &gemini, &agent] {
        // v1: review consolidates N reports + writes review.md +
        // optionally merges YAML. No more --calibrate / --finalize
        // round-trip.
        assert!(
            body.contains("--merge-reports"),
            "skill must invoke the v1 merge-reports CLI subcommand"
        );
        assert!(
            body.contains("review.md"),
            "skill must produce the consolidated review.md document"
        );
        // Six-section structure lifted from Sentinel skill.
        assert!(
            body.contains("Executive summary") || body.contains("executive summary"),
            "review.md must include an Executive summary section"
        );
        assert!(
            body.contains("Remediation plan") || body.contains("remediation plan"),
            "review.md must include the prioritized remediation plan"
        );
        assert!(
            body.contains("Auditor ratings") || body.contains("auditor ratings"),
            "review.md must include the per-auditor ratings"
        );
        // Verdict vocabulary.
        assert!(
            body.contains("VALID")
                && body.contains("PARTIALLY VALID")
                && body.contains("MISATTRIBUTED")
                && body.contains("FALSE POSITIVE")
                && body.contains("DUPLICATE"),
            "skill must use the five-verdict vocabulary lifted from Sentinel"
        );
        // Branch B: pending YAML when telemetry doesn't exist yet.
        assert!(
            body.contains("external-audit-pending.yaml"),
            "skill must handle the Branch B (telemetry not yet present) case"
        );
        // The four-criterion weighted auditor rating.
        assert!(
            body.contains("Scope precision")
                && body.contains("Technical depth")
                && body.contains("Bug detection")
                && body.contains("False positive rate"),
            "skill must include the four-criterion weighted rating"
        );
    }
}

// ── straymark-audit-execute (PR 5 — v1 audit-skills) ────────────────────────

#[test]
fn straymark_audit_execute_claude_skill_exists_and_has_allowed_tools() {
    let body = read(
        dist_root()
            .join(".claude")
            .join("skills")
            .join("straymark-audit-execute")
            .join("SKILL.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        body.contains("name: straymark-audit-execute"),
        "missing name field"
    );
    assert!(
        body.contains("allowed-tools:"),
        "Claude skill must declare allowed-tools"
    );
    // The skill orchestrates audit execution with tool use; common build/test
    // commands across stacks should be allowlisted.
    assert!(
        body.contains("go vet")
            && body.contains("cargo")
            && body.contains("npm")
            && body.contains("pytest"),
        "allowed-tools should permit common build/test commands across stacks"
    );
    assert!(
        body.contains("argument-hint:"),
        "Claude skill should declare argument-hint for the optional CHARTER-NN arg"
    );
}

#[test]
fn straymark_audit_execute_gemini_skill_exists_without_allowed_tools() {
    let body = read(
        dist_root()
            .join(".gemini")
            .join("skills")
            .join("straymark-audit-execute")
            .join("SKILL.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        body.contains("name: straymark-audit-execute"),
        "missing name field"
    );
    assert!(
        !body.contains("allowed-tools:"),
        "Gemini skill must not declare allowed-tools"
    );
}

#[test]
fn straymark_audit_execute_agent_workflow_exists_with_description_only() {
    let body = read(
        dist_root()
            .join(".agent")
            .join("workflows")
            .join("straymark-audit-execute.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        !body.contains("name:"),
        "agent workflow must not declare a name field"
    );
    assert!(
        !body.contains("allowed-tools:"),
        "agent workflow must not declare allowed-tools"
    );
    assert!(
        body.contains("description:"),
        "agent workflow must declare description"
    );
}

#[test]
fn straymark_audit_execute_three_platforms_share_core_guidance() {
    let claude = read(
        dist_root()
            .join(".claude")
            .join("skills")
            .join("straymark-audit-execute")
            .join("SKILL.md"),
    );
    let gemini = read(
        dist_root()
            .join(".gemini")
            .join("skills")
            .join("straymark-audit-execute")
            .join("SKILL.md"),
    );
    let agent = read(
        dist_root()
            .join(".agent")
            .join("workflows")
            .join("straymark-audit-execute.md"),
    );

    for body in [&claude, &gemini, &agent] {
        // Canonical paths the skill uses.
        assert!(
            body.contains(".straymark/audits/"),
            "skill must reference the v1 canonical audit dir"
        );
        assert!(
            body.contains("audit-prompt.md"),
            "skill must read the unified audit prompt"
        );
        assert!(
            body.contains("report-<self-model-slug>.md")
                || body.contains("report-<slug>.md"),
            "skill must write the report at the keyed path"
        );

        // D14: discovery automático when arg omitted.
        assert!(
            body.contains("argument is optional")
                || body.contains("argument provided")
                || body.contains("arg omitted")
                || body.contains("Auto-discover"),
            "skill must handle the optional-argument auto-discovery case"
        );

        // D14: model-slug detection.
        assert!(
            body.contains("model identifier") && body.contains("slug"),
            "skill must explain how to detect and slugify the model identifier"
        );

        // The wait warning — load-bearing for parallel-CLI workflows.
        assert!(
            body.contains("ALL audits") && body.contains("complete"),
            "skill must warn the operator to wait for ALL commissioned audits before invoking review"
        );
        assert!(
            body.contains("/straymark-audit-review"),
            "skill must point at the audit-review skill as the next step"
        );

        // Discipline carried from the prompt template.
        assert!(
            body.contains("path:line"),
            "skill must reference the path:line citation discipline"
        );
        assert!(
            body.contains("Read-only") || body.contains("read-only"),
            "skill must reinforce the read-only constraint"
        );
    }
}
