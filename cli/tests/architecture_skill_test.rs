//! Sanity tests for the architecture / Loom skills shipped under `dist/`.
//!
//! These verify that each skill file exists with the expected frontmatter
//! shape (Claude has `allowed-tools`, Gemini has `name` but no `allowed-tools`,
//! agent workflow has only `description`) and that all three platforms carry
//! the load-bearing guidance. They run against the source tree, not against an
//! `init`-ed project, because the manifest already includes the parent
//! directories recursively — if the files exist in `dist/`, `init` copies them.

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

fn claude(skill: &str) -> String {
    read(dist_root().join(".claude").join("skills").join(skill).join("SKILL.md"))
}

fn gemini(skill: &str) -> String {
    read(dist_root().join(".gemini").join("skills").join(skill).join("SKILL.md"))
}

fn codex(skill: &str) -> String {
    read(dist_root().join(".codex").join("skills").join(skill).join("SKILL.md"))
}

fn agent(skill: &str) -> String {
    read(dist_root().join(".agent").join("workflows").join(format!("{skill}.md")))
}

/// The four-variant frontmatter contract every shipped skill must honor.
fn assert_four_variant_shape(skill: &str) {
    let c = claude(skill);
    assert!(c.starts_with("---\n"), "{skill}: claude missing YAML frontmatter");
    assert!(c.contains(&format!("name: {skill}")), "{skill}: claude missing name field");
    assert!(c.contains("allowed-tools:"), "{skill}: claude must declare allowed-tools");

    let g = gemini(skill);
    assert!(g.starts_with("---\n"), "{skill}: gemini missing YAML frontmatter");
    assert!(g.contains(&format!("name: {skill}")), "{skill}: gemini missing name field");
    assert!(
        !g.contains("allowed-tools:"),
        "{skill}: gemini must not declare allowed-tools"
    );

    let x = codex(skill);
    assert!(x.starts_with("---\n"), "{skill}: codex missing YAML frontmatter");
    assert!(x.contains(&format!("name: {skill}")), "{skill}: codex missing name field");
    assert!(
        !x.contains("allowed-tools:"),
        "{skill}: codex must not declare allowed-tools"
    );

    let a = agent(skill);
    assert!(a.starts_with("---\n"), "{skill}: agent workflow missing YAML frontmatter");
    assert!(
        !a.contains("name:"),
        "{skill}: agent workflow must not declare a name field (description-only)"
    );
    assert!(
        !a.contains("allowed-tools:"),
        "{skill}: agent workflow must not declare allowed-tools"
    );
    assert!(a.contains("description:"), "{skill}: agent workflow must declare description");
}

// ── straymark-architecture ──────────────────────────────────────────────────

#[test]
fn architecture_skill_has_four_variant_shape() {
    assert_four_variant_shape("straymark-architecture");
}

#[test]
fn architecture_skill_shares_core_guidance() {
    for body in [
        &claude("straymark-architecture"),
        &gemini("straymark-architecture"),
        &codex("straymark-architecture"),
        &agent("straymark-architecture"),
    ] {
        // Wraps the CLI commands it drives.
        assert!(
            body.contains("straymark architecture generate"),
            "skill must wrap `architecture generate`"
        );
        assert!(
            body.contains("straymark architecture validate"),
            "skill must wrap `architecture validate`"
        );
        // The Sentinel gotchas — each must be pre-empted by the skill.
        assert!(
            body.contains("component.id") && body.contains("layer.id"),
            "skill must encode the component.id != layer.id gotcha"
        );
        assert!(
            body.contains("list of target component ids"),
            "skill must encode that links is a list of string ids"
        );
        assert!(
            body.contains("3D renders edges") && body.contains("2D renders them"),
            "skill must encode the dual-edge (model.yml links + plan.drawio edges) gotcha"
        );
        // The refinement is the headline: reassign out of unassigned.
        assert!(
            body.contains("unassigned"),
            "skill must reference the placeholder unassigned layer"
        );
        // Validate signal vocabulary.
        assert!(
            body.contains("undrawn") && body.contains("unmodeled") && body.contains("empty"),
            "skill must explain the three validate signals"
        );
    }
}

// ── straymark-loom ──────────────────────────────────────────────────────────

#[test]
fn loom_skill_has_four_variant_shape() {
    assert_four_variant_shape("straymark-loom");
}

#[test]
fn loom_skill_shares_core_guidance() {
    for body in [
        &claude("straymark-loom"),
        &gemini("straymark-loom"),
        &codex("straymark-loom"),
        &agent("straymark-loom"),
    ] {
        assert!(
            body.contains("straymark loom serve"),
            "skill must wrap `loom serve`"
        );
        // The lifecycle verbs.
        assert!(
            body.contains("up") && body.contains("down") && body.contains("status"),
            "skill must cover the up/down/status lifecycle"
        );
        // Correct default port (7700, not 7779).
        assert!(
            body.contains("7700"),
            "skill must report the correct default port 7700"
        );
        assert!(
            !body.contains("7779"),
            "skill must not reference the wrong port 7779"
        );
        // Terminal-free contract: --no-open, agent owns the process.
        assert!(
            body.contains("--no-open"),
            "skill must launch with --no-open (agent owns the process, not a browser)"
        );
    }
}

// ── straymark-architecture-sync ─────────────────────────────────────────────

#[test]
fn architecture_sync_skill_has_four_variant_shape() {
    assert_four_variant_shape("straymark-architecture-sync");
}

#[test]
fn architecture_sync_skill_shares_core_guidance() {
    for body in [
        &claude("straymark-architecture-sync"),
        &gemini("straymark-architecture-sync"),
        &codex("straymark-architecture-sync"),
        &agent("straymark-architecture-sync"),
    ] {
        assert!(
            body.contains("straymark architecture sync"),
            "skill must wrap `architecture sync`"
        );
        // Append-only is the load-bearing guarantee.
        assert!(
            body.contains("append-only"),
            "skill must state the append-only guarantee"
        );
        assert!(
            body.contains("--apply"),
            "skill must explain the dry-run → --apply flow"
        );
        // Confirm-before-write discipline.
        assert!(
            body.contains("Confirm with the operator") || body.contains("confirm"),
            "skill must require operator confirmation before writing"
        );
    }
}
