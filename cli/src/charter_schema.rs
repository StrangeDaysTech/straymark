//! JSON Schema validation for Charter frontmatter.
//!
//! The schema is shipped at `<framework>/.straymark/schemas/charter.schema.v0.json`
//! (the framework distribution drops it into the project at `straymark init` time).
//! This module loads and compiles the schema once and validates frontmatter
//! parsed as `serde_yaml::Value`, mapping JSON-Schema errors to the
//! `ValidationIssue` shape used by the rest of the validate pipeline so the
//! existing output formatter handles them uniformly.

use anyhow::{anyhow, Context, Result};
use jsonschema::JSONSchema;
use serde_json::Value;
use std::path::{Path, PathBuf};

use crate::validation::{Severity, ValidationIssue};

/// Path to the Charter schema relative to a project's `.straymark/` directory.
pub const SCHEMA_RELATIVE_PATH: &str = "schemas/charter.schema.v0.json";

/// A loaded and compiled Charter schema, ready to validate frontmatter.
pub struct CharterSchema {
    compiled: JSONSchema,
}

impl CharterSchema {
    /// Load and compile the Charter schema from a project's `.straymark/`
    /// directory. Returns an error if the schema file is missing, not valid
    /// JSON, or not a valid JSON Schema.
    pub fn load(straymark_dir: &Path) -> Result<Self> {
        let path = straymark_dir.join(SCHEMA_RELATIVE_PATH);
        let raw = std::fs::read_to_string(&path).with_context(|| {
            format!(
                "Failed to read Charter schema at {}. Run `straymark repair` to restore framework files.",
                path.display()
            )
        })?;
        Self::from_json_str(&raw, path)
    }

    /// Compile the schema from a raw JSON string. Split out for testability.
    /// `source_path` is used only in error messages; it is not stored.
    pub fn from_json_str(raw: &str, source_path: PathBuf) -> Result<Self> {
        let schema_json: Value = serde_json::from_str(raw).with_context(|| {
            format!("Charter schema at {} is not valid JSON", source_path.display())
        })?;
        let compiled = JSONSchema::options()
            .compile(&schema_json)
            .map_err(|e| anyhow!("Failed to compile Charter schema: {e}"))?;
        Ok(Self { compiled })
    }

    /// Validate a Charter frontmatter (parsed YAML) against the schema. Returns
    /// a Vec of `ValidationIssue` (empty if the frontmatter is valid).
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
                    rule: "CHARTER-CONVERT".to_string(),
                    message: format!(
                        "Charter frontmatter cannot be converted to JSON for schema validation: {e}"
                    ),
                    severity: Severity::Error,
                    fix_hint: Some(
                        "Frontmatter values must be JSON-compatible (no YAML-only constructs like timestamps or non-string keys).".to_string(),
                    ),
                }];
            }
        };
        // Bind the validate result to a local before consuming so the
        // ErrorIterator's borrow of `json_value` is released before the
        // function returns.
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

/// Build a stable rule code from a validation error's schema path. The schema
/// path is a JSON Pointer like `/properties/charter_id/pattern` — we translate
/// it to a readable code like `CHARTER-SCHEMA/charter_id/pattern`.
fn rule_from_error(err: &jsonschema::ValidationError) -> String {
    let path = err.schema_path.to_string();
    let trimmed = path.trim_start_matches('/').replace("/properties/", "/");
    if trimmed.is_empty() {
        "CHARTER-SCHEMA".to_string()
    } else {
        format!("CHARTER-SCHEMA/{}", trimmed)
    }
}

/// Format a validation error into a single-line message that includes the
/// instance location (so the user knows which field is at fault).
fn format_message(err: &jsonschema::ValidationError) -> String {
    let instance_path = err.instance_path.to_string();
    let location = if instance_path.is_empty() {
        "frontmatter".to_string()
    } else {
        instance_path.trim_start_matches('/').replace('/', ".")
    };
    format!("{} (at {})", err, location)
}

/// Provide friendlier hints for the most common violations. Returns `None`
/// when no specific guidance is warranted; the schema error message itself
/// is generally clear enough as a fallback.
fn hint_for(err: &jsonschema::ValidationError) -> Option<String> {
    let path = err.schema_path.to_string();
    if path.contains("/charter_id/pattern") {
        Some(
            "charter_id must match CHARTER-NN[-slug] (e.g., CHARTER-01-anomaly-thresholds)."
                .to_string(),
        )
    } else if path.contains("/status/enum") {
        Some("status must be one of: declared, in-progress, closed.".to_string())
    } else if path.contains("/effort_estimate/enum") {
        Some("effort_estimate must be one of: XS, S, M, L.".to_string())
    } else if path.contains("/required") {
        Some("Add the missing required field to the frontmatter (charter_id, status, effort_estimate, trigger).".to_string())
    } else if path.contains("/not") {
        Some(
            "originating_ailogs and originating_spec are mutually exclusive — set exactly one or neither."
                .to_string(),
        )
    } else if path.contains("/originating_ailogs/items/pattern") {
        Some("Each originating_ailogs entry must match AILOG-YYYY-MM-DD-NNN[-slug].".to_string())
    } else {
        None
    }
}

/// Convert a `serde_yaml::Value` to a `serde_json::Value`. YAML is a superset
/// of JSON so the conversion is direct for the constructs the schema expects.
/// YAML-only constructs (non-string mapping keys, tagged values not handled
/// here) cause a typed error.
///
/// Public so that `telemetry_schema` and other future schema validators can
/// reuse the conversion without duplicating it.
pub fn yaml_to_json_value(v: &serde_yaml::Value) -> Result<Value> {
    Ok(match v {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(b) => Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Number(i.into())
            } else if let Some(u) = n.as_u64() {
                Value::Number(u.into())
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        serde_yaml::Value::String(s) => Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            let mut arr = Vec::with_capacity(seq.len());
            for item in seq {
                arr.push(yaml_to_json_value(item)?);
            }
            Value::Array(arr)
        }
        serde_yaml::Value::Mapping(map) => {
            let mut obj = serde_json::Map::with_capacity(map.len());
            for (k, val) in map {
                let key = match k {
                    serde_yaml::Value::String(s) => s.clone(),
                    other => {
                        return Err(anyhow!(
                            "YAML mapping key is not a string: {:?} (JSON Schema requires string keys)",
                            other
                        ));
                    }
                };
                obj.insert(key, yaml_to_json_value(val)?);
            }
            Value::Object(obj)
        }
        serde_yaml::Value::Tagged(t) => yaml_to_json_value(&t.value)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Charter schema text. In tests we keep an in-tree copy of the
    /// schema's structure (a minimal subset) so this module compiles without
    /// depending on the framework distribution being installed at test time.
    /// The full schema lives at `dist/.straymark/schemas/charter.schema.v0.json`.
    const TEST_SCHEMA: &str = r##"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["charter_id", "status", "effort_estimate", "trigger"],
        "properties": {
            "charter_id": {
                "type": "string",
                "pattern": "^CHARTER-[0-9]{2,}(-[a-z0-9-]+)?$"
            },
            "status": {
                "type": "string",
                "enum": ["declared", "in-progress", "closed"]
            },
            "effort_estimate": {
                "type": "string",
                "enum": ["XS", "S", "M", "L"]
            },
            "trigger": { "type": "string", "minLength": 1 },
            "originating_ailogs": {
                "type": "array",
                "items": {
                    "type": "string",
                    "pattern": "^AILOG-[0-9]{4}-[0-9]{2}-[0-9]{2}-[0-9]{3}[a-z]?(-[a-z0-9-]+)?$"
                },
                "minItems": 1
            },
            "originating_spec": { "type": "string", "minLength": 1 }
        },
        "not": { "required": ["originating_ailogs", "originating_spec"] }
    }"##;

    fn schema() -> CharterSchema {
        CharterSchema::from_json_str(TEST_SCHEMA, PathBuf::from("test://schema")).unwrap()
    }

    fn yaml(s: &str) -> serde_yaml::Value {
        serde_yaml::from_str(s).unwrap()
    }

    #[test]
    fn validates_minimal_valid_frontmatter() {
        let s = schema();
        let fm = yaml(
            r#"
charter_id: CHARTER-01-test
status: declared
effort_estimate: M
trigger: "x"
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(issues.is_empty(), "unexpected issues: {:?}", issues);
    }

    #[test]
    fn validates_with_ailogs_origin() {
        let s = schema();
        let fm = yaml(
            r#"
charter_id: CHARTER-02-with-ailog
status: in-progress
effort_estimate: S
trigger: "x"
originating_ailogs:
  - AILOG-2026-04-28-021
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(issues.is_empty(), "unexpected issues: {:?}", issues);
    }

    #[test]
    fn accepts_ailog_id_with_letter_suffix() {
        // Optional single-letter suffix on the sequence number resolves
        // same-day same-sequence filename collisions without renumbering
        // downstream entries. Additive: existing IDs without suffix still pass.
        let s = schema();
        let fm = yaml(
            r#"
charter_id: CHARTER-02-collision
status: declared
effort_estimate: S
trigger: "x"
originating_ailogs:
  - AILOG-2026-05-02-028b
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(issues.is_empty(), "unexpected issues: {:?}", issues);
    }

    #[test]
    fn rejects_ailog_id_with_multiletter_suffix() {
        // Single letter only — discourages ad-hoc multi-letter labels that
        // would erode the convention.
        let s = schema();
        let fm = yaml(
            r#"
charter_id: CHARTER-02-multiletter
status: declared
effort_estimate: S
trigger: "x"
originating_ailogs:
  - AILOG-2026-05-02-028bc
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(!issues.is_empty(), "expected pattern rejection for -028bc");
    }

    #[test]
    fn rejects_ailog_id_with_uppercase_suffix() {
        // Lowercase only — matches the existing [a-z0-9-]+ style elsewhere
        // in the filename grammar.
        let s = schema();
        let fm = yaml(
            r#"
charter_id: CHARTER-02-upper
status: declared
effort_estimate: S
trigger: "x"
originating_ailogs:
  - AILOG-2026-05-02-028B
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(!issues.is_empty(), "expected pattern rejection for -028B");
    }

    #[test]
    fn validates_with_spec_origin() {
        let s = schema();
        let fm = yaml(
            r#"
charter_id: CHARTER-03-from-spec
status: declared
effort_estimate: L
trigger: "x"
originating_spec: specs/001-feature/spec.md
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(issues.is_empty(), "unexpected issues: {:?}", issues);
    }

    #[test]
    fn rejects_missing_required_field() {
        let s = schema();
        // Missing `trigger`.
        let fm = yaml(
            r#"
charter_id: CHARTER-04-missing
status: declared
effort_estimate: M
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(!issues.is_empty(), "expected at least one issue");
        assert!(
            issues.iter().any(|i| i.rule.contains("required") || i.message.contains("required") || i.message.contains("trigger")),
            "expected required-field error, got: {:?}",
            issues
        );
    }

    #[test]
    fn rejects_invalid_status_enum() {
        let s = schema();
        let fm = yaml(
            r#"
charter_id: CHARTER-05-bad-status
status: unknown-state
effort_estimate: M
trigger: "x"
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(!issues.is_empty());
        assert!(
            issues.iter().any(|i| i.rule.contains("status") || i.message.contains("status") || i.fix_hint.as_deref().map(|h| h.contains("status")).unwrap_or(false)),
            "expected status enum error, got: {:?}",
            issues
        );
    }

    #[test]
    fn rejects_invalid_effort_estimate() {
        let s = schema();
        let fm = yaml(
            r#"
charter_id: CHARTER-06-bad-effort
status: declared
effort_estimate: XXL
trigger: "x"
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(!issues.is_empty());
    }

    #[test]
    fn rejects_charter_id_pattern_mismatch() {
        let s = schema();
        // Plan-NN format (Sentinel historical) is rejected — StrayMark vocabulary
        // requires CHARTER-NN.
        let fm = yaml(
            r#"
charter_id: PLAN-01
status: declared
effort_estimate: M
trigger: "x"
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(!issues.is_empty());
        assert!(
            issues.iter().any(|i| i.fix_hint.as_deref().map(|h| h.contains("CHARTER-NN")).unwrap_or(false)),
            "expected charter_id hint, got: {:?}",
            issues
        );
    }

    #[test]
    fn rejects_both_origins_set() {
        let s = schema();
        let fm = yaml(
            r#"
charter_id: CHARTER-07-both
status: declared
effort_estimate: M
trigger: "x"
originating_ailogs: [AILOG-2026-04-28-021]
originating_spec: specs/001-feature/spec.md
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(!issues.is_empty());
        assert!(
            issues.iter().any(|i| i.fix_hint.as_deref().map(|h| h.contains("mutually exclusive")).unwrap_or(false)),
            "expected mutual-exclusion hint, got: {:?}",
            issues
        );
    }

    #[test]
    fn accepts_neither_origin_set() {
        // Both absent is valid (Charter scaffolded without explicit origin).
        let s = schema();
        let fm = yaml(
            r#"
charter_id: CHARTER-08-no-origin
status: declared
effort_estimate: M
trigger: "x"
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(issues.is_empty(), "unexpected issues: {:?}", issues);
    }

    #[test]
    fn additional_properties_are_permitted() {
        // The schema has additionalProperties: true (default), so unknown
        // fields like `note` and `closed_at` must not trigger errors.
        let s = schema();
        let fm = yaml(
            r#"
charter_id: CHARTER-09-extras
status: closed
effort_estimate: XS
trigger: "x"
note: "anonymized example"
closed_at: "2026-04-30"
"#,
        );
        let issues = s.validate(&fm, Path::new("test.md"));
        assert!(issues.is_empty(), "unexpected issues: {:?}", issues);
    }

    #[test]
    fn yaml_to_json_handles_basic_types() {
        let v = yaml(
            r#"
str: hello
int: 42
float: 3.14
bool: true
null_val: null
list: [1, 2, 3]
nested: { a: 1, b: 2 }
"#,
        );
        let j = yaml_to_json_value(&v).unwrap();
        let obj = j.as_object().unwrap();
        assert_eq!(obj.get("str").unwrap().as_str(), Some("hello"));
        assert_eq!(obj.get("int").unwrap().as_i64(), Some(42));
        assert_eq!(obj.get("bool").unwrap().as_bool(), Some(true));
        assert!(obj.get("null_val").unwrap().is_null());
    }

    #[test]
    fn yaml_to_json_rejects_non_string_keys() {
        let mut map = serde_yaml::Mapping::new();
        map.insert(serde_yaml::Value::Number(1.into()), serde_yaml::Value::String("v".into()));
        let v = serde_yaml::Value::Mapping(map);
        let err = yaml_to_json_value(&v).unwrap_err();
        assert!(err.to_string().contains("not a string"));
    }
}
