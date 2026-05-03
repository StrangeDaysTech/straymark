//! JSON Schema validation for audit output frontmatter.
//!
//! The schema is shipped at
//! `<framework>/.devtrail/schemas/audit-output.schema.v0.json` and validates
//! the YAML frontmatter of the markdown files produced by auditors and the
//! calibrator-reconciler during a `devtrail charter audit` cycle. The schema
//! uses `oneOf` to discriminate auditor outputs (primary/secondary) from
//! calibrator outputs via the `audit_role` field.
//!
//! Mirrors `charter_schema.rs` and `telemetry_schema.rs` — same shape, same
//! `ValidationIssue` integration so the existing output formatter handles
//! audit-output errors uniformly.

use anyhow::{anyhow, Context, Result};
use jsonschema::JSONSchema;
use serde_json::Value;
use std::path::{Path, PathBuf};

use crate::charter_schema::yaml_to_json_value;
use crate::validation::{Severity, ValidationIssue};

/// Path to the audit-output schema relative to a project's `.devtrail/`.
pub const SCHEMA_RELATIVE_PATH: &str = "schemas/audit-output.schema.v0.json";

/// A loaded and compiled audit-output schema, ready to validate YAML.
pub struct AuditOutputSchema {
    compiled: JSONSchema,
}

impl AuditOutputSchema {
    /// Load and compile the audit-output schema from a project's `.devtrail/`
    /// directory. Returns an error if the schema file is missing, not valid
    /// JSON, or not a valid JSON Schema.
    pub fn load(devtrail_dir: &Path) -> Result<Self> {
        let path = devtrail_dir.join(SCHEMA_RELATIVE_PATH);
        let raw = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "Failed to read audit-output schema at {}. Run `devtrail repair` to restore framework files.",
                path.display()
            )
        })?;
        Self::from_json_str(&raw, path)
    }

    /// Compile the schema from a raw JSON string. Split out for testability.
    pub fn from_json_str(raw: &str, source_path: PathBuf) -> Result<Self> {
        let schema_json: Value = serde_json::from_str(raw).with_context(|| {
            format!(
                "Audit-output schema at {} is not valid JSON",
                source_path.display()
            )
        })?;
        let compiled = JSONSchema::options()
            .compile(&schema_json)
            .map_err(|e| anyhow!("Failed to compile audit-output schema: {e}"))?;
        Ok(Self { compiled })
    }

    /// Validate parsed audit-output frontmatter (YAML) against the schema.
    /// Returns an empty Vec if valid; otherwise one `ValidationIssue` per
    /// violation.
    pub fn validate(
        &self,
        yaml_value: &serde_yaml::Value,
        file_path: &Path,
    ) -> Vec<ValidationIssue> {
        let json_value = match yaml_to_json_value(yaml_value) {
            Ok(v) => v,
            Err(e) => {
                return vec![ValidationIssue {
                    file: file_path.to_path_buf(),
                    rule: "AUDIT-CONVERT".to_string(),
                    message: format!(
                        "Audit-output YAML cannot be converted to JSON for schema validation: {e}"
                    ),
                    severity: Severity::Error,
                    fix_hint: Some(
                        "Frontmatter must be JSON-compatible YAML (no tagged values or non-string keys)."
                            .to_string(),
                    ),
                }];
            }
        };
        let issues: Vec<ValidationIssue> = match self.compiled.validate(&json_value) {
            Ok(()) => Vec::new(),
            Err(errors) => errors
                .map(|err| ValidationIssue {
                    file: file_path.to_path_buf(),
                    rule: rule_from_error(&err),
                    message: format_message(&err),
                    severity: Severity::Error,
                    fix_hint: hint_for(&err),
                })
                .collect(),
        };
        issues
    }
}

fn rule_from_error(err: &jsonschema::ValidationError) -> String {
    let path = err.schema_path.to_string();
    let trimmed = path.trim_start_matches('/').replace("/properties/", "/");
    if trimmed.is_empty() {
        "AUDIT-SCHEMA".to_string()
    } else {
        format!("AUDIT-SCHEMA/{}", trimmed)
    }
}

fn format_message(err: &jsonschema::ValidationError) -> String {
    let instance_path = err.instance_path.to_string();
    let location = if instance_path.is_empty() {
        "frontmatter".to_string()
    } else {
        instance_path.trim_start_matches('/').replace('/', ".")
    };
    format!("{} (at {})", err, location)
}

fn hint_for(err: &jsonschema::ValidationError) -> Option<String> {
    let path = err.schema_path.to_string();
    if path.contains("/audit_role/enum") || path.contains("/audit_role/const") {
        Some(
            "audit_role must be one of: auditor-primary, auditor-secondary, calibrator-reconciler."
                .to_string(),
        )
    } else if path.contains("/charter_id/pattern") {
        Some(
            "charter_id must match CHARTER-NN[-slug] (e.g., CHARTER-05-baseline-recompute)."
                .to_string(),
        )
    } else if path.contains("/audited_at/format") || path.contains("/calibrated_at/format") {
        Some("Date fields must be in YYYY-MM-DD format.".to_string())
    } else if path.contains("/auditors_reconciled/minItems")
        || path.contains("/auditors_reconciled/maxItems")
    {
        Some(
            "auditors_reconciled must have exactly 2 entries — the dual-audit pattern is fixed in v0."
                .to_string(),
        )
    } else if path.contains("/oneOf") {
        Some(
            "Output must match exactly one of the auditor or calibrator shapes — check audit_role and required fields."
                .to_string(),
        )
    } else if path.contains("/required") {
        Some(
            "Required field missing. See audit-output.schema.v0.json for the full list."
                .to_string(),
        )
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal-shape schema for unit tests. The real schema lives at
    /// dist/.devtrail/schemas/audit-output.schema.v0.json and is not bundled
    /// into the binary — these tests are for the wrapper logic.
    const TEST_SCHEMA: &str = r##"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "oneOf": [
          {
            "type": "object",
            "required": ["audit_role", "auditor", "charter_id", "audited_at", "findings_total"],
            "properties": {
              "audit_role": { "type": "string", "enum": ["auditor-primary", "auditor-secondary"] },
              "auditor": { "type": "string", "minLength": 1 },
              "charter_id": { "type": "string", "pattern": "^CHARTER-[0-9]{2,}(-[a-z0-9-]+)?$" },
              "audited_at": { "type": "string", "format": "date" },
              "findings_total": { "type": "integer", "minimum": 0 }
            }
          },
          {
            "type": "object",
            "required": ["audit_role", "calibrator", "charter_id", "calibrated_at", "auditors_reconciled"],
            "properties": {
              "audit_role": { "type": "string", "const": "calibrator-reconciler" },
              "calibrator": { "type": "string", "minLength": 1 },
              "charter_id": { "type": "string", "pattern": "^CHARTER-[0-9]{2,}(-[a-z0-9-]+)?$" },
              "calibrated_at": { "type": "string", "format": "date" },
              "auditors_reconciled": {
                "type": "array",
                "minItems": 2,
                "maxItems": 2,
                "items": { "type": "string" }
              }
            }
          }
        ]
    }"##;

    fn schema() -> AuditOutputSchema {
        AuditOutputSchema::from_json_str(TEST_SCHEMA, PathBuf::from("test://schema")).unwrap()
    }

    fn yaml(s: &str) -> serde_yaml::Value {
        serde_yaml::from_str(s).unwrap()
    }

    #[test]
    fn valid_auditor_primary_passes() {
        let v = yaml(
            r#"
audit_role: auditor-primary
auditor: copilot-v1.0.37
charter_id: CHARTER-05
audited_at: "2026-05-03"
findings_total: 3
"#,
        );
        assert!(schema().validate(&v, Path::new("test.md")).is_empty());
    }

    #[test]
    fn valid_calibrator_passes() {
        let v = yaml(
            r#"
audit_role: calibrator-reconciler
calibrator: claude-opus-4
charter_id: CHARTER-05
calibrated_at: "2026-05-03"
auditors_reconciled:
  - auditor-primary.md
  - auditor-secondary.md
"#,
        );
        assert!(schema().validate(&v, Path::new("test.md")).is_empty());
    }

    #[test]
    fn auditor_with_calibrator_role_fails() {
        // audit_role: calibrator-reconciler but with auditor fields → fails oneOf.
        let v = yaml(
            r#"
audit_role: calibrator-reconciler
auditor: copilot-v1.0.37
charter_id: CHARTER-05
audited_at: "2026-05-03"
findings_total: 3
"#,
        );
        let issues = schema().validate(&v, Path::new("test.md"));
        assert!(!issues.is_empty(), "expected oneOf violation, got no issues");
    }

    #[test]
    fn calibrator_with_one_auditor_fails() {
        let v = yaml(
            r#"
audit_role: calibrator-reconciler
calibrator: claude-opus-4
charter_id: CHARTER-05
calibrated_at: "2026-05-03"
auditors_reconciled:
  - auditor-primary.md
"#,
        );
        let issues = schema().validate(&v, Path::new("test.md"));
        assert!(!issues.is_empty(), "minItems:2 should be enforced");
    }

    #[test]
    fn invalid_charter_id_pattern_caught() {
        let v = yaml(
            r#"
audit_role: auditor-primary
auditor: copilot
charter_id: not-a-charter-id
audited_at: "2026-05-03"
findings_total: 0
"#,
        );
        let issues = schema().validate(&v, Path::new("test.md"));
        assert!(!issues.is_empty());
    }
}
