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
