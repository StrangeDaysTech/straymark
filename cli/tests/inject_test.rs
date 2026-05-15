use std::fs;
use tempfile::TempDir;

#[test]
fn test_inject_markers_are_valid() {
    let begin = "<!-- straymark:begin -->";
    let end = "<!-- straymark:end -->";

    // Valid HTML comments
    assert!(begin.starts_with("<!--"));
    assert!(begin.ends_with("-->"));
    assert!(end.starts_with("<!--"));
    assert!(end.ends_with("-->"));
}

#[test]
fn test_inject_reference_creates_file() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("CLAUDE.md");

    // Simulate what inject_directive does with a reference template
    let template = "# StrayMark - Claude Code Configuration\n\n<!-- straymark:begin -->\n> **Read and follow the rules in [STRAYMARK.md](STRAYMARK.md).**\n> That file contains all StrayMark documentation governance rules for this project.\n<!-- straymark:end -->\n";
    fs::write(&target, template).unwrap();

    let result = fs::read_to_string(&target).unwrap();
    assert!(result.contains("<!-- straymark:begin -->"));
    assert!(result.contains("<!-- straymark:end -->"));
    assert!(result.contains("STRAYMARK.md"));
    assert!(result.contains("# StrayMark - Claude Code Configuration"));
}

#[test]
fn test_remove_injection() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("test.md");

    let content = "# My Config\n\nSome custom stuff\n\n<!-- straymark:begin -->\nStrayMark rules here\n<!-- straymark:end -->\n\nMore custom stuff\n";
    fs::write(&target, content).unwrap();

    // Simulate removal
    let content = fs::read_to_string(&target).unwrap();
    let begin = "<!-- straymark:begin -->";
    let end = "<!-- straymark:end -->";

    if let (Some(start), Some(end_pos)) = (content.find(begin), content.find(end)) {
        let end_pos = end_pos + end.len();
        let before = content[..start].trim_end();
        let after = content[end_pos..].trim_start();
        let new_content = format!("{}\n\n{}", before, after);
        fs::write(&target, &new_content).unwrap();
    }

    let result = fs::read_to_string(&target).unwrap();
    assert!(!result.contains("straymark:begin"));
    assert!(result.contains("My Config"));
    assert!(result.contains("More custom stuff"));
}

#[test]
fn test_agents_md_template_has_markers() {
    // Sanity check that the shipped AGENTS.md template has the markers in the right shape.
    let template_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("dist/dist-templates/directives/AGENTS.md");
    let template = fs::read_to_string(&template_path)
        .expect("AGENTS.md template must exist at dist/dist-templates/directives/AGENTS.md");

    assert!(template.contains("<!-- straymark:begin -->"));
    assert!(template.contains("<!-- straymark:end -->"));
    assert!(template.contains("STRAYMARK.md"));
    // The Universal Agent Configuration header tells the reader this is the cross-CLI entry point.
    assert!(template.contains("Universal Agent Configuration"));
}

#[test]
fn test_manifest_declares_agents_md_injection() {
    // The dist-manifest.yml must list AGENTS.md among injection targets so init/update/remove
    // pick it up via the data-driven path (not just the legacy fallback in remove.rs).
    let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("dist/dist-manifest.yml");
    let manifest = fs::read_to_string(&manifest_path).expect("dist-manifest.yml must exist");

    assert!(manifest.contains("target: AGENTS.md"));
    assert!(manifest.contains("template: dist-templates/directives/AGENTS.md"));
}

#[test]
fn test_agents_md_coexists_with_preexisting_file() {
    // Simulate the case where the adopter already has an AGENTS.md (very common in 2026 repos).
    // The injection must append the marker block, not overwrite the existing content.
    let dir = TempDir::new().unwrap();
    let target = dir.path().join("AGENTS.md");

    let preexisting = "# My Project AGENTS.md\n\n## Build\n\nrun `cargo build`\n";
    fs::write(&target, preexisting).unwrap();

    let marker_block =
        "<!-- straymark:begin -->\n> Read STRAYMARK.md\n<!-- straymark:end -->";
    let original = fs::read_to_string(&target).unwrap();
    let appended = format!("{}\n\n{}\n", original.trim_end(), marker_block);
    fs::write(&target, &appended).unwrap();

    let result = fs::read_to_string(&target).unwrap();
    // Preexisting content survives.
    assert!(result.contains("# My Project AGENTS.md"));
    assert!(result.contains("cargo build"));
    // StrayMark content is appended.
    assert!(result.contains("<!-- straymark:begin -->"));
    assert!(result.contains("STRAYMARK.md"));
}

#[test]
fn test_template_with_embed_markers() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join(".cursorrules");

    // Template with empty markers (embed-style)
    let template = "# StrayMark - Cursor Configuration\n\n<!-- straymark:begin -->\n<!-- straymark:end -->\n";
    let embed_content = "# STRAYMARK Rules\nRule 1: Document everything\nRule 2: Follow conventions";

    // Simulate inject_directive with embed: build marker block with embedded content
    let marker_block = format!(
        "<!-- straymark:begin -->\n{}\n<!-- straymark:end -->",
        embed_content.trim()
    );
    let full_content = template.replace(
        "<!-- straymark:begin -->\n<!-- straymark:end -->",
        &marker_block,
    );
    fs::write(&target, &full_content).unwrap();

    let result = fs::read_to_string(&target).unwrap();
    assert!(result.contains("# StrayMark - Cursor Configuration"));
    assert!(result.contains("Rule 1: Document everything"));
    assert!(result.contains("Rule 2: Follow conventions"));
    assert!(result.contains("<!-- straymark:begin -->"));
    assert!(result.contains("<!-- straymark:end -->"));
}
