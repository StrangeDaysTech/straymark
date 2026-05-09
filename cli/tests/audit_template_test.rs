//! Sanity tests for the v1 unified audit prompt template
//! (`dist/.straymark/audit-prompts/audit-prompt.md`) and the schema
//! evolution that accompanies it. These verify that:
//!
//! - The unified template ships and is well-formed (7 universal sections,
//!   expected placeholders, didactic example preserved, credit to Sentinel).
//! - The schema accepts the new `audit_role: auditor` value while still
//!   accepting v0 legacy values during the transition.
//! - The optional `evidence_citations` field validates correctly.
//!
//! These complement the resolver tests in `charter_audit_test.rs` (R10) and
//! the default-range tests (R11(A)). PR 4 will add tests that exercise the
//! CLI using this template; PR 3 only ships the artifact.

use std::path::PathBuf;

const UNIFIED_TEMPLATE: &str = include_str!(
    "../../dist/.straymark/audit-prompts/audit-prompt.md"
);

const AUDIT_OUTPUT_SCHEMA: &str = include_str!(
    "../../dist/.straymark/schemas/audit-output.schema.v0.json"
);

fn template_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("dist")
        .join(".straymark")
        .join("audit-prompts")
        .join("audit-prompt.md")
}

#[test]
fn unified_template_ships_at_canonical_path() {
    assert!(
        template_path().exists(),
        "expected the unified template at {}",
        template_path().display()
    );
    assert!(
        UNIFIED_TEMPLATE.len() > 5_000,
        "template seems suspiciously short ({} bytes); the seven universal \
         sections lifted from Sentinel should be substantial",
        UNIFIED_TEMPLATE.len()
    );
}

#[test]
fn unified_template_has_seven_universal_sections() {
    // Per Propuesta/straymark-audit-cli-flow.md §2 D12, the lift from
    // Sentinel preserves these seven sections integrally. Each is
    // identifiable by a stable heading or distinctive opener that survives
    // markdown rendering in any viewer the operator uses.
    let sections = [
        ("REGLA ABSOLUTA", "## ⛔ REGLA ABSOLUTA — SOLO LECTURA"),
        ("Tu rol", "## Tu rol"),
        ("Reglas de alcance", "### Reglas de alcance"),
        (
            "Paso 2 verificación obligatoria",
            "### Paso 2 — Verificar cada tarea (OBLIGATORIO)",
        ),
        (
            "Paso 5 calibración de severidad",
            "### Paso 5 — Calibrar severidad contra la configuración REAL del proyecto",
        ),
        ("Formato de salida", "## Formato de salida"),
        ("Lo que NO debes hacer", "## Lo que NO debes hacer"),
    ];
    for (label, anchor) in sections {
        assert!(
            UNIFIED_TEMPLATE.contains(anchor),
            "missing universal section '{label}': anchor '{anchor}' not found"
        );
    }
}

#[test]
fn unified_template_declares_expected_placeholders() {
    // The placeholders the resolver substitutes are documented in the HTML
    // comment header AND used in the body. R10 fix already ensures the
    // documentation header is preserved verbatim — these assertions verify
    // the placeholders are present in the file at all.
    let required_placeholders = [
        "{{charter_id}}",
        "{{charter_title}}",
        "{{charter_path}}",
        "{{charter_content}}",
        "{{git_range}}",
        "{{git_diff}}",
        "{{ailog_paths}}",
        "{{ailog_contents}}",
        "{{audit_role}}",
        "{{schema_path}}",
    ];
    for p in required_placeholders {
        assert!(
            UNIFIED_TEMPLATE.contains(p),
            "missing required placeholder {p}"
        );
    }

    // The new project context placeholder (D12 parameterization of
    // Sentinel's hardcoded "monolito modular en Go..." description).
    assert!(
        UNIFIED_TEMPLATE.contains("{{project_context}}"),
        "missing v1 placeholder {{{{project_context}}}}"
    );
}

#[test]
fn unified_template_preserves_etapa_12_didactic_example() {
    // The Etapa 12 example (Pub/Sub stub vs gochannel active) is kept
    // verbatim from Sentinel's audit/SKILL.md as illustration of the
    // anti-inflation discipline. It must be labeled as a real adopter case.
    assert!(
        UNIFIED_TEMPLATE.contains("Etapa 12"),
        "didactic Etapa 12 example must be preserved"
    );
    assert!(
        UNIFIED_TEMPLATE.contains("gochannel"),
        "concrete driver name preserved as part of the example"
    );
    assert!(
        UNIFIED_TEMPLATE.contains("(caso real, proyecto adoptante)")
            || UNIFIED_TEMPLATE.contains("Ejemplo (caso real")
            || UNIFIED_TEMPLATE.contains("ejemplo de un caso real"),
        "the example must be explicitly labeled as a real adopter case so \
         readers don't mistake it for prescription"
    );
}

#[test]
fn unified_template_credits_sentinel_contribution() {
    // Per Propuesta §6 (CHANGELOG note about explicit credit). Both the
    // header HTML comment and the closing italics block credit Sentinel /
    // José Villaseñor Montfort for the source material.
    assert!(
        UNIFIED_TEMPLATE.contains("issue #102"),
        "credit must reference the contribution issue"
    );
    assert!(
        UNIFIED_TEMPLATE.contains("Sentinel"),
        "credit must name Sentinel as the source project"
    );
}

#[test]
fn unified_template_enforces_evidence_discipline() {
    // R11(B): paste-based audits without tool use produce structurally
    // limited findings. The unified template's "Disciplina de evidencia"
    // sub-block (inside Paso 2) enforces the path:line citation rule.
    assert!(
        UNIFIED_TEMPLATE.contains("archivo:línea")
            && UNIFIED_TEMPLATE.contains("tool call"),
        "evidence discipline ('archivo:línea' + 'tool call') must be \
         enunciated explicitly in the template"
    );
}

#[test]
fn schema_accepts_v1_unified_audit_role() {
    // jsonschema lib is in the CLI's deps; using it here would couple the
    // test to internal config. Keep this test minimal: confirm the schema
    // text contains the new enum variant and the explanatory comment.
    let schema: serde_json::Value = serde_json::from_str(AUDIT_OUTPUT_SCHEMA)
        .expect("schema must be valid JSON");
    let auditor_role = schema
        .pointer("/$defs/auditorOutput/properties/audit_role/enum")
        .expect("auditorOutput.audit_role.enum must be present")
        .as_array()
        .expect("audit_role enum must be an array");
    let values: Vec<&str> = auditor_role
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(values.contains(&"auditor"), "v1 unified value must be present");
    assert!(
        values.contains(&"auditor-primary") && values.contains(&"auditor-secondary"),
        "v0 legacy values must still be accepted during transition: got {values:?}"
    );
}

#[test]
fn schema_declares_evidence_citations_optional() {
    let schema: serde_json::Value = serde_json::from_str(AUDIT_OUTPUT_SCHEMA)
        .expect("schema must be valid JSON");
    let prop = schema
        .pointer("/$defs/auditorOutput/properties/evidence_citations")
        .expect("evidence_citations field must be defined");
    assert_eq!(prop.get("type").and_then(|v| v.as_str()), Some("integer"));
    assert_eq!(
        prop.get("minimum").and_then(|v| v.as_u64()),
        Some(0),
        "evidence_citations is a non-negative count"
    );
    // Confirm it's NOT in the required list (it's optional).
    let required = schema
        .pointer("/$defs/auditorOutput/required")
        .and_then(|v| v.as_array())
        .expect("auditorOutput.required must be an array");
    let required_strs: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        !required_strs.contains(&"evidence_citations"),
        "evidence_citations must be optional, not required"
    );
}

#[test]
fn schema_calibrator_supports_n_auditors() {
    // v1: calibrator reconciles N≥2 auditors (no longer fixed to 2).
    let schema: serde_json::Value = serde_json::from_str(AUDIT_OUTPUT_SCHEMA)
        .expect("schema must be valid JSON");
    let auditors = schema
        .pointer("/$defs/calibratorOutput/properties/auditors_reconciled")
        .expect("auditors_reconciled field must be defined");
    assert_eq!(auditors.get("minItems").and_then(|v| v.as_u64()), Some(2));
    assert!(
        auditors.get("maxItems").is_none(),
        "v1 must NOT cap the number of auditors at 2; got maxItems = {:?}",
        auditors.get("maxItems")
    );
}
