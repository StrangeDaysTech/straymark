//! JSON Schema validation for Charter telemetry YAML.
//!
//! The schema is shipped at `<framework>/.devtrail/schemas/charter-telemetry.schema.v0.json`
//! (the framework distribution drops it into the project at `devtrail init` time).
//! This module mirrors `charter_schema.rs` — load and compile the schema once,
//! then validate parsed YAML.
//!
//! Telemetry YAML files have a single top-level key `charter_telemetry:`, so
//! the schema validates the whole file (including that wrapper). The CLI's
//! validator reports issues using the same `ValidationIssue` shape used for
//! Charter frontmatter validation.

use anyhow::{anyhow, Context, Result};
use jsonschema::JSONSchema;
use serde_json::Value;
use std::path::{Path, PathBuf};

use crate::charter_schema::yaml_to_json_value;
use crate::validation::{Severity, ValidationIssue};

/// Path to the telemetry schema relative to a project's `.devtrail/` directory.
pub const SCHEMA_RELATIVE_PATH: &str = "schemas/charter-telemetry.schema.v0.json";

/// A loaded and compiled telemetry schema, ready to validate YAML.
pub struct TelemetrySchema {
    compiled: JSONSchema,
}

impl TelemetrySchema {
    /// Load and compile the telemetry schema from a project's `.devtrail/`
    /// directory. Returns an error if the schema file is missing, not valid
    /// JSON, or not a valid JSON Schema.
    pub fn load(devtrail_dir: &Path) -> Result<Self> {
        let path = devtrail_dir.join(SCHEMA_RELATIVE_PATH);
        let raw = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "Failed to read telemetry schema at {}. Run `devtrail repair` to restore framework files.",
                path.display()
            )
        })?;
        Self::from_json_str(&raw, path)
    }

    /// Compile the schema from a raw JSON string. Split out for testability.
    pub fn from_json_str(raw: &str, source_path: PathBuf) -> Result<Self> {
        let schema_json: Value = serde_json::from_str(raw).with_context(|| {
            format!("Telemetry schema at {} is not valid JSON", source_path.display())
        })?;
        let compiled = JSONSchema::options()
            .compile(&schema_json)
            .map_err(|e| anyhow!("Failed to compile telemetry schema: {e}"))?;
        Ok(Self { compiled })
    }

    /// Validate parsed telemetry YAML against the schema. Returns an empty Vec
    /// if the YAML is valid; otherwise one `ValidationIssue` per violation.
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
                    rule: "TELEMETRY-CONVERT".to_string(),
                    message: format!(
                        "Telemetry YAML cannot be converted to JSON for schema validation: {e}"
                    ),
                    severity: Severity::Error,
                    fix_hint: Some(
                        "Telemetry must be JSON-compatible YAML (no tagged values or non-string keys).".to_string(),
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
        "TELEMETRY-SCHEMA".to_string()
    } else {
        format!("TELEMETRY-SCHEMA/{}", trimmed)
    }
}

fn format_message(err: &jsonschema::ValidationError) -> String {
    let instance_path = err.instance_path.to_string();
    let location = if instance_path.is_empty() {
        "telemetry".to_string()
    } else {
        instance_path.trim_start_matches('/').replace('/', ".")
    };
    format!("{} (at {})", err, location)
}

fn hint_for(err: &jsonschema::ValidationError) -> Option<String> {
    let path = err.schema_path.to_string();
    if path.contains("/charter_id/pattern") {
        Some(
            "charter_id must match CHARTER-NN[-slug] (e.g., CHARTER-01-anomaly-thresholds)."
                .to_string(),
        )
    } else if path.contains("/closed_at/format") || path.contains("/closed_at/type") {
        Some("closed_at must be a date string in YYYY-MM-DD format.".to_string())
    } else if path.contains("/scope_changes/enum") {
        Some("scope_changes must be one of: ninguno, menor, mayor.".to_string())
    } else if path.contains("/effort/required") || path.contains("/charter_telemetry/required") {
        Some(
            "Required telemetry fields are missing. At minimum: charter_id, charter_title, closed_at, effort, outcome."
                .to_string(),
        )
    } else if path.contains("/estimated_effort/pattern") || path.contains("/actual_effort/pattern")
    {
        Some(
            "Effort fields must match XS|S|M|L optionally followed by a parenthesized hint, e.g., 'M (~1.5h)'."
                .to_string(),
        )
    } else if path.contains("/overall_satisfaction/maximum")
        || path.contains("/overall_satisfaction/minimum")
    {
        Some("overall_satisfaction must be an integer in 1..=5.".to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal-shape telemetry schema for unit tests. The real schema lives
    /// at `dist/.devtrail/schemas/charter-telemetry.schema.v0.json` and is
    /// not bundled into the binary — these tests are for the wrapper logic.
    const TEST_SCHEMA: &str = r##"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["charter_telemetry"],
        "properties": {
            "charter_telemetry": {
                "type": "object",
                "required": ["charter_id", "charter_title", "closed_at", "effort", "outcome"],
                "properties": {
                    "charter_id": {
                        "type": "string",
                        "pattern": "^CHARTER-[0-9]{2,}(-[a-z0-9-]+)?$"
                    },
                    "charter_title": { "type": "string" },
                    "closed_at": { "type": "string", "format": "date" },
                    "effort": {
                        "type": "object",
                        "required": ["estimated_effort", "actual_effort"],
                        "properties": {
                            "estimated_effort": { "type": "string" },
                            "actual_effort": { "type": "string" }
                        }
                    },
                    "outcome": {
                        "type": "object",
                        "required": ["completed_as_planned", "scope_changes"],
                        "properties": {
                            "completed_as_planned": { "type": "boolean" },
                            "scope_changes": {
                                "type": "string",
                                "enum": ["ninguno", "menor", "mayor"]
                            }
                        }
                    }
                }
            }
        }
    }"##;

    fn schema() -> TelemetrySchema {
        TelemetrySchema::from_json_str(TEST_SCHEMA, PathBuf::from("test://schema")).unwrap()
    }

    fn yaml(s: &str) -> serde_yaml::Value {
        serde_yaml::from_str(s).unwrap()
    }

    #[test]
    fn valid_minimal_telemetry_passes() {
        let v = yaml(
            r#"
charter_telemetry:
  charter_id: CHARTER-01
  charter_title: Test
  closed_at: "2026-05-02"
  effort:
    estimated_effort: M
    actual_effort: M
  outcome:
    completed_as_planned: true
    scope_changes: ninguno
"#,
        );
        assert!(schema().validate(&v, Path::new("test.yaml")).is_empty());
    }

    #[test]
    fn missing_required_field_is_caught() {
        let v = yaml(
            r#"
charter_telemetry:
  charter_id: CHARTER-01
  charter_title: Test
  closed_at: "2026-05-02"
"#,
        );
        let issues = schema().validate(&v, Path::new("test.yaml"));
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.message.contains("required")));
    }

    #[test]
    fn bad_charter_id_pattern_is_caught() {
        let v = yaml(
            r#"
charter_telemetry:
  charter_id: not-a-charter-id
  charter_title: Test
  closed_at: "2026-05-02"
  effort:
    estimated_effort: M
    actual_effort: M
  outcome:
    completed_as_planned: true
    scope_changes: ninguno
"#,
        );
        let issues = schema().validate(&v, Path::new("test.yaml"));
        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|i| i.rule.contains("charter_id") || i.message.contains("pattern")));
    }
}
