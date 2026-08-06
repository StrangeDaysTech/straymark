//! `retired:` — the declarative retirement mechanism (fw-4.42.0).
//!
//! Behavioral coverage of the sweep itself lives next to the code, in
//! `commands::update_framework::tests` (this crate has no lib target, so
//! integration tests can only reach the filesystem). What is pinned here is the
//! *shipped manifest*: the mechanism is inert unless the release actually
//! declares the channels it retires.

use std::fs;

fn manifest() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("dist/dist-manifest.yml");
    fs::read_to_string(&path).expect("dist-manifest.yml must exist")
}

#[test]
fn shipped_manifest_retires_the_dead_channels() {
    let m = manifest();
    let retired_block = m
        .split("retired:")
        .nth(1)
        .expect("dist-manifest.yml must declare a `retired:` key");

    for path in [".gemini/skills/", ".agent/workflows/"] {
        assert!(
            retired_block.contains(&format!("- {path}")),
            "dist-manifest.yml must retire {path}"
        );
    }
}

#[test]
fn retired_channels_are_not_also_distributed() {
    let m = manifest();
    let files_block = m
        .split("files:")
        .nth(1)
        .and_then(|s| s.split("injections:").next())
        .expect("dist-manifest.yml must declare `files:`");

    for path in [".gemini/skills/", ".agent/workflows/"] {
        assert!(
            !files_block.contains(&format!("- {path}")),
            "{path} cannot be distributed and retired at the same time"
        );
    }
}
