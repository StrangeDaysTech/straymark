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
fn devtrail_audit_prompt_claude_skill_exists_and_has_allowed_tools() {
    let body = read(
        dist_root()
            .join(".claude")
            .join("skills")
            .join("devtrail-audit-prompt")
            .join("SKILL.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        body.contains("name: devtrail-audit-prompt"),
        "missing name field"
    );
    assert!(
        body.contains("allowed-tools:"),
        "Claude skill must declare allowed-tools"
    );
    assert!(
        body.contains("devtrail charter audit"),
        "skill body must reference the CLI command it wraps"
    );
}

#[test]
fn devtrail_audit_prompt_gemini_skill_exists_without_allowed_tools() {
    let body = read(
        dist_root()
            .join(".gemini")
            .join("skills")
            .join("devtrail-audit-prompt")
            .join("SKILL.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        body.contains("name: devtrail-audit-prompt"),
        "missing name field"
    );
    assert!(
        !body.contains("allowed-tools:"),
        "Gemini skill must not declare allowed-tools (no such field in Gemini skill schema)"
    );
}

#[test]
fn devtrail_audit_prompt_agent_workflow_exists_with_description_only() {
    let body = read(
        dist_root()
            .join(".agent")
            .join("workflows")
            .join("devtrail-audit-prompt.md"),
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
fn devtrail_audit_prompt_three_platforms_share_core_guidance() {
    let claude = read(
        dist_root()
            .join(".claude")
            .join("skills")
            .join("devtrail-audit-prompt")
            .join("SKILL.md"),
    );
    let gemini = read(
        dist_root()
            .join(".gemini")
            .join("skills")
            .join("devtrail-audit-prompt")
            .join("SKILL.md"),
    );
    let agent = read(
        dist_root()
            .join(".agent")
            .join("workflows")
            .join("devtrail-audit-prompt.md"),
    );

    // The next-steps guidance text is load-bearing for the workflow —
    // the operator gets the same instructions regardless of platform.
    for body in [&claude, &gemini, &agent] {
        assert!(
            body.contains("Run AUDITOR PRIMARY PROMPT"),
            "next-steps guidance missing"
        );
        assert!(
            body.contains("DO NOT use the same family for both"),
            "heterogeneity recommendation missing"
        );
        assert!(
            body.contains("/devtrail-audit-review"),
            "skill must point operator at the follow-up review skill"
        );
    }
}

// ── devtrail-audit-review (PR 2) ───────────────────────────────────────────

#[test]
fn devtrail_audit_review_claude_skill_exists_and_has_allowed_tools() {
    let body = read(
        dist_root()
            .join(".claude")
            .join("skills")
            .join("devtrail-audit-review")
            .join("SKILL.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        body.contains("name: devtrail-audit-review"),
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
fn devtrail_audit_review_gemini_skill_exists_without_allowed_tools() {
    let body = read(
        dist_root()
            .join(".gemini")
            .join("skills")
            .join("devtrail-audit-review")
            .join("SKILL.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        body.contains("name: devtrail-audit-review"),
        "missing name field"
    );
    assert!(
        !body.contains("allowed-tools:"),
        "Gemini skill must not declare allowed-tools"
    );
}

#[test]
fn devtrail_audit_review_agent_workflow_exists_with_description_only() {
    let body = read(
        dist_root()
            .join(".agent")
            .join("workflows")
            .join("devtrail-audit-review.md"),
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
fn devtrail_audit_review_three_platforms_share_core_guidance() {
    let claude = read(
        dist_root()
            .join(".claude")
            .join("skills")
            .join("devtrail-audit-review")
            .join("SKILL.md"),
    );
    let gemini = read(
        dist_root()
            .join(".gemini")
            .join("skills")
            .join("devtrail-audit-review")
            .join("SKILL.md"),
    );
    let agent = read(
        dist_root()
            .join(".agent")
            .join("workflows")
            .join("devtrail-audit-review.md"),
    );

    for body in [&claude, &gemini, &agent] {
        assert!(
            body.contains("--calibrate"),
            "skill must reference the CALIBRATE step"
        );
        assert!(
            body.contains("--finalize"),
            "skill must reference the FINALIZE step"
        );
        assert!(
            body.contains("external-audit-pending.yaml"),
            "skill must handle the Branch B (telemetry not yet present) case"
        );
        assert!(
            body.contains("re-audit") || body.contains("re-merge"),
            "skill must surface the v0 re-audit limitation"
        );
    }
}

// ── devtrail-audit-execute (PR 5 — v1 audit-skills) ────────────────────────

#[test]
fn devtrail_audit_execute_claude_skill_exists_and_has_allowed_tools() {
    let body = read(
        dist_root()
            .join(".claude")
            .join("skills")
            .join("devtrail-audit-execute")
            .join("SKILL.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        body.contains("name: devtrail-audit-execute"),
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
fn devtrail_audit_execute_gemini_skill_exists_without_allowed_tools() {
    let body = read(
        dist_root()
            .join(".gemini")
            .join("skills")
            .join("devtrail-audit-execute")
            .join("SKILL.md"),
    );
    assert!(body.starts_with("---\n"), "missing YAML frontmatter");
    assert!(
        body.contains("name: devtrail-audit-execute"),
        "missing name field"
    );
    assert!(
        !body.contains("allowed-tools:"),
        "Gemini skill must not declare allowed-tools"
    );
}

#[test]
fn devtrail_audit_execute_agent_workflow_exists_with_description_only() {
    let body = read(
        dist_root()
            .join(".agent")
            .join("workflows")
            .join("devtrail-audit-execute.md"),
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
fn devtrail_audit_execute_three_platforms_share_core_guidance() {
    let claude = read(
        dist_root()
            .join(".claude")
            .join("skills")
            .join("devtrail-audit-execute")
            .join("SKILL.md"),
    );
    let gemini = read(
        dist_root()
            .join(".gemini")
            .join("skills")
            .join("devtrail-audit-execute")
            .join("SKILL.md"),
    );
    let agent = read(
        dist_root()
            .join(".agent")
            .join("workflows")
            .join("devtrail-audit-execute.md"),
    );

    for body in [&claude, &gemini, &agent] {
        // Canonical paths the skill uses.
        assert!(
            body.contains(".devtrail/audits/"),
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
            body.contains("/devtrail-audit-review"),
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
