//! JSON Schema validation for the follow-ups registry frontmatter.
//!
//! The schema ships at `.straymark/schemas/follow-ups-backlog.schema.v1.json`
//! (experimental v1 — fw-4.21.0). Validation is advisory: `straymark
//! followups status` surfaces issues as warnings, never as failures, in line
//! with the registry's lenient-parsing contract (ADR-2026-06-03-001). Unlike
//! `charter_schema`, results are plain strings — the registry is not part of
//! the `straymark validate` document pipeline.

use anyhow::{anyhow, Context, Result};
use jsonschema::JSONSchema;
use serde_json::Value;
use std::path::Path;

/// Path to the registry schema relative to a project's `.straymark/` directory.
pub const SCHEMA_RELATIVE_PATH: &str = "schemas/follow-ups-backlog.schema.v1.json";

/// Validate a registry's raw frontmatter against the shipped schema.
/// Returns a list of human-readable issues (empty when valid), or an error
/// when the schema file itself is missing or broken — callers typically
/// downgrade that to a dimmed hint (older framework installs predate the
/// schema).
pub fn validate_frontmatter(straymark_dir: &Path, frontmatter_raw: &str) -> Result<Vec<String>> {
    let schema_path = straymark_dir.join(SCHEMA_RELATIVE_PATH);
    let raw = std::fs::read_to_string(&schema_path).with_context(|| {
        format!(
            "Registry schema not found at {} (ships with fw-4.21.0+; run `straymark update-framework`)",
            schema_path.display()
        )
    })?;
    let schema_json: Value = serde_json::from_str(&raw)
        .with_context(|| format!("Registry schema at {} is not valid JSON", schema_path.display()))?;
    let compiled = JSONSchema::options()
        .compile(&schema_json)
        .map_err(|e| anyhow!("Failed to compile registry schema: {e}"))?;

    let yaml_value: serde_yaml::Value = serde_yaml::from_str(frontmatter_raw)
        .context("Registry frontmatter is not parseable YAML")?;
    let json_value = yaml_to_json(&yaml_value)?;

    let issues = match compiled.validate(&json_value) {
        Ok(()) => Vec::new(),
        Err(errors) => errors
            .map(|e| {
                let path = e.instance_path.to_string();
                if path.is_empty() {
                    e.to_string()
                } else {
                    format!("{}: {}", path, e)
                }
            })
            .collect(),
    };
    Ok(issues)
}

/// Convert YAML to JSON for schema validation. Mirrors the conversion in
/// `charter_schema` (kept local: that module's helper is private and the
/// two modules migrate to straymark-core on different timelines).
fn yaml_to_json(yaml: &serde_yaml::Value) -> Result<Value> {
    serde_json::to_value(yaml).context("Failed to convert registry frontmatter YAML to JSON")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Minimal but real copy of the shipped schema's required-fields core.
    const MINI_SCHEMA: &str = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["schema_version", "last_scan", "buckets", "fully_extracted_ailogs"],
        "additionalProperties": true,
        "properties": {
            "schema_version": {"type": "string", "enum": ["v0", "v1"]}
        }
    }"#;

    fn setup(schema: &str) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("schemas");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("follow-ups-backlog.schema.v1.json"), schema).unwrap();
        tmp
    }

    #[test]
    fn valid_frontmatter_yields_no_issues() {
        let tmp = setup(MINI_SCHEMA);
        let fm = "schema_version: v1\nlast_scan: 2026-06-04\nbuckets: [ready]\nfully_extracted_ailogs: []";
        let issues = validate_frontmatter(tmp.path(), fm).unwrap();
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn missing_required_field_is_reported() {
        let tmp = setup(MINI_SCHEMA);
        let fm = "schema_version: v1\nlast_scan: 2026-06-04\nbuckets: [ready]";
        let issues = validate_frontmatter(tmp.path(), fm).unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("fully_extracted_ailogs")));
    }

    #[test]
    fn missing_schema_file_is_an_error_with_update_hint() {
        let tmp = TempDir::new().unwrap();
        let err = validate_frontmatter(tmp.path(), "schema_version: v1").unwrap_err();
        assert!(err.to_string().contains("update-framework"));
    }
}
