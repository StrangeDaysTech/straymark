use std::path::{Path, PathBuf};

use crate::config::StrayMarkConfig;
use straymark_core::document::{self, StrayMarkDocument, DocType};

/// Severity of a validation issue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// A single validation issue found in a document
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub file: PathBuf,
    pub rule: String,
    pub message: String,
    pub severity: Severity,
    pub fix_hint: Option<String>,
}

/// Result of validating one or more documents
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

impl ValidationResult {
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }

    fn add(&mut self, issue: ValidationIssue) {
        match issue.severity {
            Severity::Error => self.errors.push(issue),
            Severity::Warning => self.warnings.push(issue),
        }
    }
}

/// Valid status values per DOCUMENTATION-POLICY.md §3 lifecycle + §6 per-type defaults.
///
/// - `identified` is the canonical TDE entry state (agent-driven discovery, awaits
///   human prioritization); functionally equivalent to `draft` for lifecycle gating
///   but semantically distinct in adopter analytics (regression #130).
/// - `resolved` is the canonical TDE terminal state for "the debt described here
///   was addressed; document is kept on disk as audit history". Neither `accepted`
///   ("we accept this debt continues to exist"), `superseded` ("another TDE took
///   its place"), nor `deprecated` ("the TDE concept itself is no longer relevant")
///   captures this semantics correctly. Issue #149 surfaced the gap empirically
///   in Sentinel post-CHARTER-17 housekeeping.
///
/// **Per-doc-type lifecycle vocabulary** (Option B in issue #149) is the principled
/// next evolution: promote this flat enum to a `HashMap<DocType, Vec<&str>>` so each
/// doc type has its own canonical state machine and the validator inspects
/// `doc.doc_type` before deciding which set to apply. Deferred deliberately —
/// shipping Option A (flat enum + `resolved`) keeps the validator's surface stable
/// and avoids per-type branching until a second doc type needs a non-standard
/// terminal. Adding `resolved` flat-globally is mildly permissive (an ADR with
/// `status: resolved` would pass) but the damage is bounded: `DOCUMENTATION-POLICY`
/// documents `resolved` as TDE-specific, and the per-type expansion in Option B
/// will tighten the validation without breaking any existing TDE adopters.
const VALID_STATUSES: &[&str] = &[
    "draft",
    "identified",
    "review",
    "accepted",
    "resolved",
    "superseded",
    "deprecated",
];

/// Valid risk levels
const VALID_RISK_LEVELS: &[&str] = &["low", "medium", "high", "critical"];

/// Valid confidence levels
const VALID_CONFIDENCES: &[&str] = &["low", "medium", "high"];

/// Patterns that indicate sensitive information.
/// These are checked against the document body. Patterns should be specific enough
/// to avoid false positives in documentation that describes auth flows.
const SENSITIVE_PATTERNS: &[&str] = &[
    "password:", "api_key:", "secret:", "private_key:",
    "credentials:", "AWS_SECRET", "PRIVATE KEY",
];

/// Patterns that are suspicious but common in documentation describing auth flows.
/// These produce warnings instead of errors.
const SOFT_SENSITIVE_PATTERNS: &[&str] = &[
    "token:", "Bearer ",
];

/// True if the project's regional_scope (loaded from `.straymark/config.yml` at the
/// project root that contains the given `.straymark/` directory) includes "china".
fn china_in_scope(straymark_dir: &Path) -> bool {
    let project_root = straymark_dir.parent().unwrap_or(straymark_dir);
    let config = StrayMarkConfig::load(project_root).unwrap_or_default();
    config.has_region("china")
}

/// Validate all Charters in a project against the Charter JSON Schema and
/// referential integrity rules:
/// - Schema (shape, enums, required fields, mutual exclusion of origin types).
/// - `originating_ailogs` IDs resolve to real AILOG files under
///   `.straymark/07-ai-audit/agent-logs/`.
/// - `originating_spec` path exists relative to the project root.
/// - `CHARTER-FILES-EXIST`: every non-`new` path declared in `## Files to
///   modify` exists on disk (finding #210 — Charter authored against assumed
///   code). Warning-only; separate from `charter drift`'s OMISSION check.
///
/// Returns the result + number of Charters considered (parsed + parse-failed).
/// If the schema file itself cannot be loaded, emits a single warning and
/// skips schema-level checks; referential integrity is still attempted.
pub fn validate_charters(project_root: &Path, straymark_dir: &Path) -> (ValidationResult, usize) {
    let mut result = ValidationResult::default();

    // Try to load the schema. Missing schema is a warning, not a hard failure
    // (the project may have been initialized before the schema shipped, or
    // the file may have been removed).
    let schema = match crate::charter_schema::CharterSchema::load(straymark_dir) {
        Ok(s) => Some(s),
        Err(e) => {
            result.warnings.push(ValidationIssue {
                file: straymark_dir.join(crate::charter_schema::SCHEMA_RELATIVE_PATH),
                rule: "CHARTER-SCHEMA-MISSING".to_string(),
                message: format!("Charter schema not loadable: {e}"),
                severity: Severity::Warning,
                fix_hint: Some(
                    "Run `straymark repair` to restore framework files.".to_string(),
                ),
            });
            None
        }
    };

    let paths = straymark_core::charter::discover_charters(project_root);
    let charter_count = paths.len();

    for path in &paths {
        // Step 1: read raw YAML frontmatter (without typed deserialization).
        // This preserves schema-level errors (bad enum, missing required) so
        // the schema validator sees them and emits rich hints, rather than
        // letting a typed-parse failure mask the actual cause.
        let raw_yaml = match straymark_core::charter::read_frontmatter_yaml(path) {
            Ok(y) => y,
            Err(e) => {
                result.errors.push(ValidationIssue {
                    file: path.clone(),
                    rule: "CHARTER-PARSE".to_string(),
                    message: format!("Failed to read Charter: {e}"),
                    severity: Severity::Error,
                    fix_hint: Some(
                        "Check that the file has valid YAML frontmatter between --- delimiters."
                            .to_string(),
                    ),
                });
                continue;
            }
        };

        // Step 2: schema validation. Catches shape errors (enum mismatch,
        // missing required, mutual exclusion of origin types) with friendly
        // hints from `crate::charter_schema::hint_for`.
        if let Some(schema) = &schema {
            for issue in schema.validate(&raw_yaml, path) {
                result.errors.push(issue);
            }
        }

        // Step 3: typed parse for referential-integrity checks. If schema
        // validation already caught problems, the typed parse may also fail —
        // in that case we skip ref checks (cannot trust the structure) but
        // we don't double-report (errors already in result via schema).
        let typed: Option<straymark_core::charter::CharterFrontmatter> =
            serde_yaml::from_value(raw_yaml).ok();
        let typed = match typed {
            Some(t) => t,
            None => continue,
        };

        // CHARTER-AILOG-REF: every AILOG ID (origin and close-time execution,
        // #215 Gap 2) must resolve to a file.
        let ailog_refs = typed
            .originating_ailogs
            .iter()
            .flatten()
            .map(|id| ("originating_ailogs", id))
            .chain(
                typed
                    .execution_ailogs
                    .iter()
                    .flatten()
                    .map(|id| ("execution_ailogs", id)),
            );
        for (field, ailog_id) in ailog_refs {
            if !ailog_exists(straymark_dir, ailog_id) {
                result.errors.push(ValidationIssue {
                    file: path.clone(),
                    rule: "CHARTER-AILOG-REF".to_string(),
                    message: format!("{field} references missing AILOG: {ailog_id}"),
                    severity: Severity::Error,
                    fix_hint: Some(format!(
                        "Either create the AILOG (e.g., `straymark new --doc-type ailog`) or \
                         remove '{ailog_id}' from {field} if it was a typo."
                    )),
                });
            }
        }

        // CHARTER-SPEC-REF: the originating_spec / context_spec path must exist.
        for (field, spec_path) in [
            ("originating_spec", typed.originating_spec.as_ref()),
            ("context_spec", typed.context_spec.as_ref()),
        ] {
            let Some(spec_path) = spec_path else { continue };
            if !project_root.join(spec_path).exists() {
                result.errors.push(ValidationIssue {
                    file: path.clone(),
                    rule: "CHARTER-SPEC-REF".to_string(),
                    message: format!("{field} references missing file: {spec_path}"),
                    severity: Severity::Error,
                    fix_hint: Some(format!(
                        "Pass a path that exists under the project root (e.g., \
                         specs/001-feature/spec.md), or remove {field} if it was a typo."
                    )),
                });
            }
        }

        // CHARTER-FILES-EXIST (finding #210): every path declared in the
        // `## Files to modify` section that is NOT tagged "new" must exist on
        // disk. A declared path that never existed is a *Charter authoring bug*
        // (the Charter was written against assumed, un-read code) — distinct
        // from the *implementation drift* the `charter drift` command catches
        // (declared but not modified in a git range). Keeping the two checks in
        // different commands with different rule codes is the separation #210.3
        // asks for. Emitted as a Warning (not Error): adopters mid-migration may
        // legitimately list not-yet-tagged new files, and warn-only matches the
        // REF-001 / REVIEW-PENDING precedent.
        if let Ok(charter) = straymark_core::charter::parse_charter(path) {
            for declared in straymark_core::charter_files::parse_files_to_modify(&charter.body) {
                // Skip files created here (`new`) and marked cross-repo / removed /
                // relocated paths (#215 Gap 3), plus wildcard git-range patterns.
                if declared.is_existence_exempt()
                    || straymark_core::charter_files::is_wildcard(&declared.path)
                {
                    continue;
                }
                if !project_root.join(&declared.path).exists() {
                    result.add(ValidationIssue {
                        file: path.clone(),
                        rule: "CHARTER-FILES-EXIST".to_string(),
                        message: format!(
                            "`## Files to modify` declares a path that does not exist on disk: {} \
                             (Charter mis-declared — authored against assumed code).",
                            declared.path
                        ),
                        severity: Severity::Warning,
                        fix_hint: Some(
                            "Read the path before declaring it. If this Charter creates the file, \
                             mark its Change column 'New' (or tag the path '(new)') — the validator \
                             skips existence-checking new files."
                                .to_string(),
                        ),
                    });
                }
            }
        }
    }

    (result, charter_count)
}

/// True if an AILOG file matching the given ID exists under
/// `.straymark/07-ai-audit/agent-logs/`. The match is by filename prefix:
/// `AILOG-2026-04-28-021` matches `AILOG-2026-04-28-021-anything.md` but not
/// `AILOG-2026-04-28-0210-something.md` (boundary: next char must be `-` or
/// `.md` extension).
fn ailog_exists(straymark_dir: &Path, ailog_id: &str) -> bool {
    let agent_logs = straymark_dir.join("07-ai-audit").join("agent-logs");
    if !agent_logs.exists() {
        return false;
    }
    let id = ailog_id.trim_end_matches(".md");
    let entries = match std::fs::read_dir(&agent_logs) {
        Ok(e) => e,
        Err(_) => return false,
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = match name.to_str() {
            Some(s) => s,
            None => continue,
        };
        if let Some(rest) = name.strip_prefix(id) {
            // Boundary: either the file extension follows immediately or a
            // dash separator before the slug.
            if rest == ".md" || rest.starts_with('-') {
                return true;
            }
        }
    }
    false
}

/// Validate all documents found under a .straymark/ directory
pub fn validate_all(straymark_dir: &Path) -> (ValidationResult, usize) {
    let paths = document::discover_documents(straymark_dir);
    let doc_count = paths.len();
    let mut result = ValidationResult::default();
    let china = china_in_scope(straymark_dir);

    for path in &paths {
        match document::parse_document(path) {
            Ok(doc) => {
                result.merge(validate_document(&doc, straymark_dir, china));
            }
            Err(e) => {
                result.errors.push(ValidationIssue {
                    file: path.clone(),
                    rule: "PARSE-001".to_string(),
                    message: format!("Failed to parse document: {e}"),
                    severity: Severity::Error,
                    fix_hint: Some("Check that the file has valid YAML frontmatter between --- delimiters".to_string()),
                });
            }
        }
    }

    // REF-002: Detect orphan documents (no traceability links)
    check_orphan_documents(&mut result, &paths, straymark_dir);

    (result, doc_count)
}

/// Validate a specific set of document paths (used for --staged mode).
/// Skips orphan document checking since that is not meaningful for partial validation.
/// Surface documents whose `review_required: true` is older than a threshold
/// and still has no `review_outcome`. Per DOCUMENTATION-POLICY §3.5: warn-only,
/// never errors. Adopters opt in via `straymark validate --check-pending-reviews`.
///
/// Returns one `ValidationIssue` per pending document, all `Severity::Warning`.
pub fn check_pending_reviews(straymark_dir: &Path, max_pending_days: i64) -> Vec<ValidationIssue> {
    use chrono::{Local, NaiveDate};

    let mut issues = Vec::new();
    let today = Local::now().date_naive();
    let paths = document::discover_documents(straymark_dir);

    for path in paths {
        let doc = match document::parse_document(&path) {
            Ok(d) => d,
            Err(_) => continue, // parse errors surface via the regular validate path
        };
        if !doc.frontmatter.review_required.unwrap_or(false) {
            continue;
        }
        if doc.frontmatter.review_outcome.is_some() {
            continue; // already reviewed
        }
        let created = match doc
            .frontmatter
            .created
            .as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok())
        {
            Some(d) => d,
            None => continue, // missing/invalid created date is a separate validate rule
        };
        let age_days = (today - created).num_days();
        if age_days < max_pending_days {
            continue;
        }
        let id = doc
            .frontmatter
            .id
            .clone()
            .unwrap_or_else(|| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(String::from)
                    .unwrap_or_default()
            });
        issues.push(ValidationIssue {
            file: path,
            rule: "REVIEW-PENDING".to_string(),
            message: format!(
                "{} has `review_required: true` and no `review_outcome` ({} days since creation)",
                id, age_days
            ),
            severity: Severity::Warning,
            fix_hint: Some(format!(
                "Run `straymark approve {} --outcome <approved|revisions_requested|rejected> --reviewer <id>` once a human has reviewed.",
                id
            )),
        });
    }
    issues
}

pub fn validate_paths(paths: &[PathBuf], straymark_dir: &Path) -> (ValidationResult, usize) {
    let mut result = ValidationResult::default();
    let mut doc_count = 0;
    let china = china_in_scope(straymark_dir);

    for path in paths {
        if !path.exists() {
            continue;
        }
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if document::detect_doc_type(filename).is_none() {
            continue;
        }
        match document::parse_document(path) {
            Ok(doc) => {
                doc_count += 1;
                result.merge(validate_document(&doc, straymark_dir, china));
            }
            Err(e) => {
                doc_count += 1;
                result.errors.push(ValidationIssue {
                    file: path.clone(),
                    rule: "PARSE-001".to_string(),
                    message: format!("Failed to parse document: {e}"),
                    severity: Severity::Error,
                    fix_hint: Some(
                        "Check that the file has valid YAML frontmatter between --- delimiters"
                            .to_string(),
                    ),
                });
            }
        }
    }

    (result, doc_count)
}

/// REF-002: Check for documents with no traceability links.
/// A document is orphan if it has no `related` field AND is not referenced
/// by any other document's `related` field.
fn check_orphan_documents(result: &mut ValidationResult, paths: &[PathBuf], _straymark_dir: &Path) {
    let parsed: Vec<StrayMarkDocument> = paths
        .iter()
        .filter_map(|p| document::parse_document(p).ok())
        .collect();

    // Build a set of all filenames referenced in any document's `related` field
    let mut referenced: std::collections::HashSet<String> = std::collections::HashSet::new();
    for doc in &parsed {
        if let Some(related) = &doc.frontmatter.related {
            for rel_id in related {
                if !rel_id.is_empty() {
                    referenced.insert(rel_id.clone());
                }
            }
        }
    }

    // Skip orphan check when there are very few documents (not meaningful)
    if parsed.len() <= 2 {
        return;
    }

    // Types that are naturally standalone (don't require traceability)
    let standalone_types = [
        DocType::Eth,
        DocType::Inc,
        DocType::Tde,
        DocType::Sec,
        DocType::Mcard,
        DocType::Dpia,
        DocType::Sbom,
    ];

    for doc in &parsed {
        if standalone_types.contains(&doc.doc_type) {
            continue;
        }

        let has_related = doc
            .frontmatter
            .related
            .as_ref()
            .is_some_and(|r| r.iter().any(|s| !s.is_empty()));

        let is_referenced = referenced.iter().any(|r| doc.filename.starts_with(r.as_str()));

        if !has_related && !is_referenced {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "REF-002".to_string(),
                message: "Document has no traceability links (not in any related field and has no related of its own)".to_string(),
                severity: Severity::Warning,
                fix_hint: Some("Add a 'related' field linking to relevant documents for audit traceability".to_string()),
            });
        }
    }
}

/// Validate a single parsed document
fn validate_document(
    doc: &StrayMarkDocument,
    straymark_dir: &Path,
    china_in_scope: bool,
) -> ValidationResult {
    let mut result = ValidationResult::default();

    check_naming(&mut result, doc);
    check_required_meta(&mut result, doc);
    check_id_matches_filename(&mut result, doc);
    check_valid_status(&mut result, doc);
    check_cross_rules(&mut result, doc);
    check_type_specific(&mut result, doc);
    check_date_consistency(&mut result, doc);
    check_related_exist(&mut result, doc, straymark_dir);
    check_sensitive_info(&mut result, doc);
    check_observability(&mut result, doc);

    if china_in_scope {
        check_china_cross_rules(&mut result, doc);
        check_china_type_specific(&mut result, doc);
    }

    result
}

/// NAMING-001: Verify filename follows TYPE-YYYY-MM-DD-NNN-description.md
fn check_naming(result: &mut ValidationResult, doc: &StrayMarkDocument) {
    let name = &doc.filename;
    let prefix = doc.doc_type.prefix();

    // Check: PREFIX-YYYY-MM-DD-NNN-*.md
    let after_prefix = match name.strip_prefix(&format!("{}-", prefix)) {
        Some(rest) => rest,
        None => {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "NAMING-001".to_string(),
                message: format!("Filename should start with '{}-'", prefix),
                severity: Severity::Error,
                fix_hint: None,
            });
            return;
        }
    };

    // Check date part. We only slice by bytes once we've confirmed the
    // first 10 characters are ASCII, so this is always UTF-8-safe.
    let head: String = after_prefix.chars().take(10).collect();
    if head.chars().count() < 10 {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "NAMING-001".to_string(),
            message: "Filename missing date component (expected YYYY-MM-DD after prefix)".to_string(),
            severity: Severity::Error,
            fix_hint: None,
        });
        return;
    }
    if !head.is_ascii() {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "NAMING-001".to_string(),
            message: format!("Invalid date in filename: '{}'", head),
            severity: Severity::Error,
            fix_hint: None,
        });
        return;
    }

    let date_part = head.as_str(); // exactly 10 ASCII bytes
    let bytes = date_part.as_bytes();
    let valid_date = bytes[4] == b'-'
        && bytes[7] == b'-'
        && date_part[..4].bytes().all(|b| b.is_ascii_digit())
        && date_part[5..7].bytes().all(|b| b.is_ascii_digit())
        && date_part[8..10].bytes().all(|b| b.is_ascii_digit());

    if !valid_date {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "NAMING-001".to_string(),
            message: format!("Invalid date in filename: '{}'", date_part),
            severity: Severity::Error,
            fix_hint: None,
        });
        return;
    }

    // Skip past the 10-byte date prefix (safe: we validated it's ASCII).
    let after_date = &after_prefix[10..];
    if !after_date.starts_with('-') {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "NAMING-001".to_string(),
            message: "Missing sequence number after date (expected -NNN-)".to_string(),
            severity: Severity::Error,
            fix_hint: None,
        });
        return;
    }

    // NAMING-002: Validate sequence number is exactly 3 digits
    let after_dash = &after_date[1..]; // skip the leading '-'
    let seq_end = after_dash.find('-').unwrap_or(after_dash.len());
    let seq_part = &after_dash[..seq_end];
    if seq_part.len() != 3 || !seq_part.chars().all(|c| c.is_ascii_digit()) {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "NAMING-002".to_string(),
            message: format!(
                "Sequence number should be exactly 3 digits (e.g., 001), found '{}'",
                seq_part
            ),
            severity: Severity::Warning,
            fix_hint: Some(format!(
                "Rename with zero-padded sequence: {:0>3}",
                seq_part
            )),
        });
    }

    // NAMING-003: Validate description is kebab-case
    if seq_end < after_dash.len() {
        let desc_with_ext = &after_dash[seq_end + 1..]; // skip the '-' after sequence
        let desc = desc_with_ext.strip_suffix(".md").unwrap_or(desc_with_ext);
        if !desc.is_empty()
            && !desc
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "NAMING-003".to_string(),
                message: format!(
                    "Description should be kebab-case (lowercase, digits, hyphens only), found '{}'",
                    desc
                ),
                severity: Severity::Warning,
                fix_hint: None,
            });
        }
    }
}

/// META-001: Check presence of required fields
fn check_required_meta(result: &mut ValidationResult, doc: &StrayMarkDocument) {
    let fm = &doc.frontmatter;
    let file = &doc.path;

    let required: &[(&str, bool)] = &[
        ("id", fm.id.is_some()),
        ("title", fm.title.is_some()),
        ("status", fm.status.is_some()),
        ("created", fm.created.is_some()),
        ("agent", fm.agent.is_some()),
        ("confidence", fm.confidence.is_some()),
        ("review_required", fm.review_required.is_some()),
        ("risk_level", fm.risk_level.is_some()),
    ];

    for (field, present) in required {
        if !present {
            result.add(ValidationIssue {
                file: file.clone(),
                rule: "META-001".to_string(),
                message: format!("Missing required field: {}", field),
                severity: Severity::Error,
                fix_hint: Some(format!("Add '{}' to the frontmatter", field)),
            });
        }
    }
}

/// META-002: Check that frontmatter id matches filename prefix
fn check_id_matches_filename(result: &mut ValidationResult, doc: &StrayMarkDocument) {
    if let Some(id) = &doc.frontmatter.id {
        let expected_prefix = doc.doc_type.prefix();
        if !id.starts_with(expected_prefix) {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "META-002".to_string(),
                message: format!(
                    "Frontmatter id '{}' does not match filename prefix '{}'",
                    id, expected_prefix
                ),
                severity: Severity::Error,
                fix_hint: Some(format!("Change id to start with '{}-'", expected_prefix)),
            });
        }
    }
}

/// Common non-canonical status values observed in the field, mapped to the
/// canonical lifecycle value (#215 minor note). These are *semantic synonyms*
/// (e.g. `done` → `accepted`), not typos — edit distance alone would mis-suggest
/// them, so they get an explicit alias table. Keys are lowercased.
const STATUS_ALIASES: &[(&str, &str)] = &[
    ("complete", "accepted"),
    ("completed", "accepted"),
    ("done", "accepted"),
    ("closed", "accepted"),
    ("final", "accepted"),
    ("finished", "accepted"),
    ("merged", "accepted"),
    ("in-progress", "accepted"),
    ("in_progress", "accepted"),
    ("wip", "accepted"),
    ("ongoing", "accepted"),
    ("todo", "draft"),
    ("open", "draft"),
    ("new", "draft"),
    ("wontfix", "deprecated"),
    ("rejected", "deprecated"),
    ("abandoned", "deprecated"),
    ("obsolete", "superseded"),
    ("replaced", "superseded"),
    ("fixed", "resolved"),
    ("addressed", "resolved"),
];

/// Levenshtein edit distance (iterative, two-row). No dependency for this in the
/// repo, and it's only used to suggest a near-miss canonical status.
fn levenshtein(a: &str, b: &str) -> usize {
    let b_chars: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b_chars.len()).collect();
    let mut curr = vec![0usize; b_chars.len() + 1];
    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, &cb) in b_chars.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b_chars.len()]
}

/// Suggest the canonical status nearest to an invalid one: first a known semantic
/// alias, then a typo within edit distance 2 of a canonical value.
fn suggest_status(invalid: &str) -> Option<&'static str> {
    let lc = invalid.trim().to_lowercase();
    if let Some((_, canonical)) = STATUS_ALIASES.iter().find(|(alias, _)| *alias == lc) {
        return Some(canonical);
    }
    VALID_STATUSES
        .iter()
        .map(|&v| (v, levenshtein(&lc, v)))
        .filter(|&(_, d)| d <= 2)
        .min_by_key(|&(_, d)| d)
        .map(|(v, _)| v)
}

/// META-003: Check that status has a valid value
fn check_valid_status(result: &mut ValidationResult, doc: &StrayMarkDocument) {
    if let Some(status) = &doc.frontmatter.status {
        if !VALID_STATUSES.contains(&status.as_str()) {
            let valid = VALID_STATUSES.join(", ");
            let (message, fix_hint) = match suggest_status(status) {
                Some(suggestion) => (
                    format!("Invalid status '{status}'. Did you mean '{suggestion}'? Valid values: {valid}"),
                    Some(format!("Set status to '{suggestion}' (the canonical value for this lifecycle state).")),
                ),
                None => (
                    format!("Invalid status '{status}'. Valid values: {valid}"),
                    Some(format!("Use one of the canonical lifecycle values: {valid}.")),
                ),
            };
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "META-003".to_string(),
                message,
                severity: Severity::Error,
                fix_hint,
            });
        }
    }
}

/// CROSS-001, CROSS-002, CROSS-003: Cross-field validation rules
fn check_cross_rules(result: &mut ValidationResult, doc: &StrayMarkDocument) {
    let fm = &doc.frontmatter;

    // CROSS-001: high/critical risk_level requires review_required: true
    if let Some(risk) = &fm.risk_level {
        if (risk == "high" || risk == "critical") && fm.review_required != Some(true) {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "CROSS-001".to_string(),
                message: format!(
                    "risk_level is '{}' but review_required is not true",
                    risk
                ),
                severity: Severity::Error,
                fix_hint: Some("Set review_required: true".to_string()),
            });
        }
    }

    // Validate risk_level value
    if let Some(risk) = &fm.risk_level {
        if !VALID_RISK_LEVELS.contains(&risk.as_str()) {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "META-003".to_string(),
                message: format!(
                    "Invalid risk_level '{}'. Valid values: {}",
                    risk,
                    VALID_RISK_LEVELS.join(", ")
                ),
                severity: Severity::Error,
                fix_hint: None,
            });
        }
    }

    // Validate confidence value
    if let Some(conf) = &fm.confidence {
        if !VALID_CONFIDENCES.contains(&conf.as_str()) {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "META-003".to_string(),
                message: format!(
                    "Invalid confidence '{}'. Valid values: {}",
                    conf,
                    VALID_CONFIDENCES.join(", ")
                ),
                severity: Severity::Error,
                fix_hint: None,
            });
        }
    }

    // CROSS-002: eu_ai_act_risk: high requires review_required: true
    if let Some(eu_risk) = &fm.eu_ai_act_risk {
        if eu_risk == "high" && fm.review_required != Some(true) {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "CROSS-002".to_string(),
                message: "eu_ai_act_risk is 'high' but review_required is not true".to_string(),
                severity: Severity::Error,
                fix_hint: Some("Set review_required: true".to_string()),
            });
        }
    }

    // CROSS-003: SEC, MCARD, DPIA always require review
    let always_review_types = [DocType::Sec, DocType::Mcard, DocType::Dpia];
    if always_review_types.contains(&doc.doc_type) && fm.review_required != Some(true) {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "CROSS-003".to_string(),
            message: format!(
                "{} documents must have review_required: true",
                doc.doc_type
            ),
            severity: Severity::Error,
            fix_hint: Some("Set review_required: true".to_string()),
        });
    }
}

/// TYPE-001, TYPE-002: Type-specific field requirements
fn check_type_specific(result: &mut ValidationResult, doc: &StrayMarkDocument) {
    let fm = &doc.frontmatter;

    // TYPE-001: INC must have severity
    if doc.doc_type == DocType::Inc && fm.severity.is_none() {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "TYPE-001".to_string(),
            message: "INC documents must have a 'severity' field (SEV1/SEV2/SEV3/SEV4)".to_string(),
            severity: Severity::Error,
            fix_hint: Some("Add 'severity: SEV3' to the frontmatter".to_string()),
        });
    }

    // TYPE-002: ETH should have gdpr_legal_basis if body contains "Data Privacy"
    if doc.doc_type == DocType::Eth
        && (doc.body.contains("Data Privacy") || doc.body.contains("Privacidad de Datos"))
        && fm.gdpr_legal_basis.is_none()
    {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "TYPE-002".to_string(),
            message: "ETH document mentions Data Privacy but lacks 'gdpr_legal_basis' field".to_string(),
            severity: Severity::Warning,
            fix_hint: Some("Add 'gdpr_legal_basis: consent' (or appropriate basis) to the frontmatter".to_string()),
        });
    }
}

/// Returns true when `related` includes any entry whose ID starts with `prefix`.
fn related_has_prefix(doc: &StrayMarkDocument, prefix: &str) -> bool {
    doc.frontmatter
        .related
        .as_ref()
        .is_some_and(|rels| rels.iter().any(|r| r.starts_with(prefix)))
}

/// CROSS-004…CROSS-011: cross-field validation rules for the China regulatory profile.
/// Only invoked when `regional_scope` includes "china".
fn check_china_cross_rules(result: &mut ValidationResult, doc: &StrayMarkDocument) {
    let fm = &doc.frontmatter;

    // CROSS-004: tc260_risk_level high|very_high|extremely_severe ⇒ review_required: true
    if let Some(level) = &fm.tc260_risk_level {
        if matches!(
            level.as_str(),
            "high" | "very_high" | "extremely_severe"
        ) && fm.review_required != Some(true)
        {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "CROSS-004".to_string(),
                message: format!(
                    "tc260_risk_level is '{}' but review_required is not true",
                    level
                ),
                severity: Severity::Error,
                fix_hint: Some("Set review_required: true".to_string()),
            });
        }
    }

    // CROSS-005: pipl_sensitive_data: true ⇒ document is a PIPIA or links one via related
    if fm.pipl_sensitive_data == Some(true)
        && doc.doc_type != DocType::Pipia
        && !related_has_prefix(doc, "PIPIA-")
    {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "CROSS-005".to_string(),
            message: "pipl_sensitive_data is true but no PIPIA is linked in 'related'".to_string(),
            severity: Severity::Error,
            fix_hint: Some("Create a PIPIA and add 'PIPIA-...' to related".to_string()),
        });
    }

    // CROSS-006: cac_filing_status approved ⇒ cac_filing_number populated
    if let Some(status) = &fm.cac_filing_status {
        let approved =
            matches!(status.as_str(), "provincial_approved" | "national_approved");
        let has_number = fm
            .cac_filing_number
            .as_deref()
            .is_some_and(|n| !n.is_empty());
        if approved && !has_number {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "CROSS-006".to_string(),
                message: format!(
                    "cac_filing_status is '{}' but cac_filing_number is missing",
                    status
                ),
                severity: Severity::Error,
                fix_hint: Some(
                    "Populate cac_filing_number with the filing reference issued by CAC"
                        .to_string(),
                ),
            });
        }
    }

    // CROSS-007: cac_filing_required: true ⇒ document is a CACFILE or links one via related
    if fm.cac_filing_required == Some(true)
        && doc.doc_type != DocType::Cacfile
        && !related_has_prefix(doc, "CACFILE-")
    {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "CROSS-007".to_string(),
            message: "cac_filing_required is true but no CACFILE is linked in 'related'"
                .to_string(),
            severity: Severity::Error,
            fix_hint: Some("Create a CACFILE and add 'CACFILE-...' to related".to_string()),
        });
    }

    // CROSS-008: csl_severity_level: particularly_serious ⇒ csl_report_deadline_hours: 1
    if fm.csl_severity_level.as_deref() == Some("particularly_serious")
        && fm.csl_report_deadline_hours != Some(1)
    {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "CROSS-008".to_string(),
            message: "csl_severity_level 'particularly_serious' requires csl_report_deadline_hours: 1"
                .to_string(),
            severity: Severity::Error,
            fix_hint: Some(
                "CSL 2026: particularly serious incidents must be reported within 1 hour"
                    .to_string(),
            ),
        });
    }

    // CROSS-009: csl_severity_level: relatively_major ⇒ csl_report_deadline_hours: 4
    if fm.csl_severity_level.as_deref() == Some("relatively_major")
        && fm.csl_report_deadline_hours != Some(4)
    {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "CROSS-009".to_string(),
            message: "csl_severity_level 'relatively_major' requires csl_report_deadline_hours: 4"
                .to_string(),
            severity: Severity::Error,
            fix_hint: Some(
                "CSL 2026: relatively major incidents must be reported within 4 hours"
                    .to_string(),
            ),
        });
    }

    // CROSS-010: gb45438_applicable: true ⇒ document is an AILABEL or links one via related
    if fm.gb45438_applicable == Some(true)
        && doc.doc_type != DocType::Ailabel
        && !related_has_prefix(doc, "AILABEL-")
    {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "CROSS-010".to_string(),
            message: "gb45438_applicable is true but no AILABEL is linked in 'related'"
                .to_string(),
            severity: Severity::Error,
            fix_hint: Some(
                "Create an AILABEL describing explicit + implicit labeling per GB 45438"
                    .to_string(),
            ),
        });
    }

    // CROSS-011: pipl_cross_border_transfer: true ⇒ PIPIA documents the security review reference
    if doc.doc_type == DocType::Pipia
        && fm.pipl_cross_border_transfer == Some(true)
        && !doc.body.to_lowercase().contains("security_assessment")
        && !doc.body.to_lowercase().contains("security review")
        && !doc.body.to_lowercase().contains("standard_contract")
        && !doc.body.to_lowercase().contains("standard contract")
        && !doc.body.to_lowercase().contains("certification")
    {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "CROSS-011".to_string(),
            message: "PIPIA with cross-border transfer should document the chosen mechanism (security assessment / certification / standard contract)".to_string(),
            severity: Severity::Warning,
            fix_hint: Some(
                "Complete the 'Cross-Border Transfer Analysis' section of the PIPIA".to_string(),
            ),
        });
    }
}

/// TYPE-003…TYPE-006: type-specific rules for China-only document types.
/// Only invoked when `regional_scope` includes "china".
fn check_china_type_specific(result: &mut ValidationResult, doc: &StrayMarkDocument) {
    let fm = &doc.frontmatter;

    // TYPE-003: PIPIA must have pipl_retention_until ≥ created + 3 years
    if doc.doc_type == DocType::Pipia {
        let ok = match (fm.created.as_deref(), fm.pipl_retention_until.as_deref()) {
            (Some(c), Some(u)) => retention_satisfies_three_years(c, u),
            _ => false,
        };
        if !ok {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "TYPE-003".to_string(),
                message: "PIPIA must declare pipl_retention_until at least 3 years after 'created' (PIPL Art. 56)".to_string(),
                severity: Severity::Error,
                fix_hint: Some(
                    "Set pipl_retention_until: <created + 3 years or later> in YYYY-MM-DD format"
                        .to_string(),
                ),
            });
        }
    }

    // TYPE-004: CACFILE must have cac_filing_status set
    if doc.doc_type == DocType::Cacfile && fm.cac_filing_status.is_none() {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "TYPE-004".to_string(),
            message: "CACFILE documents must have a 'cac_filing_status' field".to_string(),
            severity: Severity::Error,
            fix_hint: Some(
                "Add 'cac_filing_status: pending' (or the current state) to the frontmatter"
                    .to_string(),
            ),
        });
    }

    // TYPE-005: TC260RA must have all three grading criteria populated
    if doc.doc_type == DocType::Tc260ra {
        let missing: Vec<&str> = [
            ("tc260_application_scenario", fm.tc260_application_scenario.is_some()),
            ("tc260_intelligence_level", fm.tc260_intelligence_level.is_some()),
            ("tc260_application_scale", fm.tc260_application_scale.is_some()),
        ]
        .into_iter()
        .filter_map(|(name, ok)| (!ok).then_some(name))
        .collect();
        if !missing.is_empty() {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "TYPE-005".to_string(),
                message: format!(
                    "TC260RA documents must populate all three grading criteria. Missing: {}",
                    missing.join(", ")
                ),
                severity: Severity::Error,
                fix_hint: Some(
                    "Set tc260_application_scenario, tc260_intelligence_level, and tc260_application_scale"
                        .to_string(),
                ),
            });
        }
    }

    // TYPE-006: AILABEL must declare at least one content type
    if doc.doc_type == DocType::Ailabel {
        let count = fm
            .gb45438_content_types
            .as_ref()
            .map(|v| v.len())
            .unwrap_or(0);
        if count == 0 {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "TYPE-006".to_string(),
                message: "AILABEL documents must declare at least one entry in 'gb45438_content_types'".to_string(),
                severity: Severity::Error,
                fix_hint: Some(
                    "Set gb45438_content_types to a subset of: text, image, audio, video, virtual_scene"
                        .to_string(),
                ),
            });
        }
    }
}

/// Parse YYYY-MM-DD into (year, month, day). Returns None on malformed input.
fn parse_iso_date(s: &str) -> Option<(i32, u32, u32)> {
    if s.len() < 10 {
        return None;
    }
    let y: i32 = s[..4].parse().ok()?;
    let m: u32 = s[5..7].parse().ok()?;
    let d: u32 = s[8..10].parse().ok()?;
    Some((y, m, d))
}

/// Returns true when `until_date` (YYYY-MM-DD) is at least 3 years after `created` (YYYY-MM-DD).
fn retention_satisfies_three_years(created: &str, until_date: &str) -> bool {
    let (cy, cm, cd) = match parse_iso_date(created) {
        Some(t) => t,
        None => return false,
    };
    let (uy, um, ud) = match parse_iso_date(until_date) {
        Some(t) => t,
        None => return false,
    };
    (uy, um, ud) >= (cy + 3, cm, cd)
}

/// REF-001: Check that documents listed in related: exist
/// Only validates references that look like StrayMark document IDs (e.g., AILOG-2025-01-27-001).
/// Skips task IDs (T025), requirement IDs (FR-019, US2), risk IDs (RISK-001),
/// external paths, and other non-document references to avoid false positives.
fn check_related_exist(result: &mut ValidationResult, doc: &StrayMarkDocument, straymark_dir: &Path) {
    if let Some(related) = &doc.frontmatter.related {
        for rel_id in related {
            if rel_id.is_empty() {
                continue;
            }
            // Only validate references that look like StrayMark document IDs
            // (start with a known document type prefix followed by a dash)
            if !looks_like_straymark_id(rel_id) {
                continue;
            }
            // Search for a file matching this id
            if !find_document_by_id(straymark_dir, rel_id) {
                result.add(ValidationIssue {
                    file: doc.path.clone(),
                    rule: "REF-001".to_string(),
                    message: format!("Related document '{}' not found in .straymark/", rel_id),
                    severity: Severity::Warning,
                    fix_hint: None,
                });
            }
        }
    }
}

/// Check if a reference looks like a StrayMark document ID.
/// Matches patterns like "AILOG-2025-01-27-001" or "ADR-2025-01-27-001-title".
/// Returns false for task IDs (T025), requirement IDs (FR-019, US2), paths, etc.
fn looks_like_straymark_id(id: &str) -> bool {
    DocType::ALL_PREFIXES.iter().any(|prefix| {
        id.starts_with(prefix) && id.get(prefix.len()..prefix.len() + 1) == Some("-")
    })
}

/// META-004: Check that filename date matches created field
fn check_date_consistency(result: &mut ValidationResult, doc: &StrayMarkDocument) {
    let Some(created) = &doc.frontmatter.created else {
        return;
    };

    // Extract date from filename: after prefix dash, take 10 chars (YYYY-MM-DD)
    let prefix = doc.doc_type.prefix();
    let after_prefix = match doc.filename.strip_prefix(&format!("{}-", prefix)) {
        Some(rest) => rest,
        _ => return,
    };
    let filename_date: String = after_prefix.chars().take(10).collect();
    if filename_date.chars().count() < 10 {
        return;
    }

    // The created field may be a full datetime or just a date — take the
    // first 10 chars safely (never slice by bytes on arbitrary input).
    let created_date: String = created.chars().take(10).collect();

    if filename_date != created_date {
        result.add(ValidationIssue {
            file: doc.path.clone(),
            rule: "META-004".to_string(),
            message: format!(
                "Filename date '{}' does not match created field '{}'",
                filename_date, created_date
            ),
            severity: Severity::Warning,
            fix_hint: None,
        });
    }
}



/// Search for a document whose filename starts with the given id
fn find_document_by_id(straymark_dir: &Path, id: &str) -> bool {
    let docs = document::discover_documents(straymark_dir);
    docs.iter().any(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with(id))
            .unwrap_or(false)
    })
}

/// SEC-001: Check for sensitive information patterns
fn check_sensitive_info(result: &mut ValidationResult, doc: &StrayMarkDocument) {
    let full_content = doc.body.to_string();
    for pattern in SENSITIVE_PATTERNS {
        if full_content.contains(pattern) {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "SEC-001".to_string(),
                message: format!("Possible sensitive information detected: '{}'", pattern.trim()),
                severity: Severity::Error,
                fix_hint: Some("Remove or redact sensitive information before committing".to_string()),
            });
        }
    }
    // Soft patterns: common in auth documentation, warn instead of error
    for pattern in SOFT_SENSITIVE_PATTERNS {
        if full_content.contains(pattern) {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "SEC-001".to_string(),
                message: format!("Review for sensitive information: '{}' (may be documentation context)", pattern.trim()),
                severity: Severity::Warning,
                fix_hint: Some("Verify this is documentation context, not an actual secret".to_string()),
            });
        }
    }
}

/// Lowercase substrings that count as observability-related content for OBS-001.
/// Kept broad on purpose: the rule only fires on docs already tagged
/// `observabilidad`/`observability`, so the goal is to avoid false positives on
/// docs that genuinely discuss instrumentation in mixed ES/EN vocabulary (#215).
const OBS_KEYWORDS: &[&str] = &[
    "## observability",
    "## observabilidad",
    "instrumentation",
    "instrumentación",
    "opentelemetry",
    "observability_scope",
    "otel",
    "telemetr", // telemetry / telemetría
    "metric",
    "métrica",
    "span",
    "trace",
    "dashboard",
    "collector",
    "alert",
    "slog",
    "histogram",
];

/// OBS-001: If document has tag 'observabilidad' or 'observability', check for relevant sections
fn check_observability(result: &mut ValidationResult, doc: &StrayMarkDocument) {
    let has_obs_tag = doc.frontmatter.tags.as_ref().is_some_and(|tags| {
        tags.iter().any(|t| t == "observabilidad" || t == "observability")
    });

    if has_obs_tag {
        // The tag itself is the signal (the adopter rule motivating it is "record
        // instrumentation-pipeline changes in an AILOG tagged observabilidad"), so
        // the content check exists only to catch a tag pasted onto an unrelated doc.
        // Match case-insensitively against a broad vocabulary: a narrow, case-sensitive
        // literal set produced 19/19 false positives in the field (#215, Gap 1) — docs
        // that talked about OTel / metrics / spans / dashboards but not the exact words.
        let body_lc = doc.body.to_lowercase();
        let has_obs_section = OBS_KEYWORDS.iter().any(|kw| body_lc.contains(kw));

        if !has_obs_section {
            result.add(ValidationIssue {
                file: doc.path.clone(),
                rule: "OBS-001".to_string(),
                message: "Document tagged with 'observabilidad'/'observability' but lacks observability-related content".to_string(),
                severity: Severity::Warning,
                fix_hint: Some("Add a section describing the instrumentation scope or observability risks".to_string()),
            });
        }
    }
}

/// Apply automatic fixes to a document
pub fn apply_fixes(doc: &StrayMarkDocument) -> Option<String> {
    let content = match std::fs::read_to_string(&doc.path) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let mut modified = false;
    let mut new_content = content.clone();

    // Fix: Add review_required: true for high-risk documents
    let needs_review = doc.frontmatter.risk_level.as_deref() == Some("high")
        || doc.frontmatter.risk_level.as_deref() == Some("critical")
        || doc.frontmatter.eu_ai_act_risk.as_deref() == Some("high")
        || matches!(doc.doc_type, DocType::Sec | DocType::Mcard | DocType::Dpia);

    if needs_review && doc.frontmatter.review_required != Some(true) {
        if new_content.contains("review_required: false") {
            new_content = new_content.replacen("review_required: false", "review_required: true", 1);
            modified = true;
        } else if doc.frontmatter.review_required.is_none() {
            // Insert review_required after risk_level
            if let Some(pos) = new_content.find("risk_level:") {
                if let Some(line_end) = new_content[pos..].find('\n') {
                    let insert_pos = pos + line_end + 1;
                    new_content.insert_str(insert_pos, "review_required: true\n");
                    modified = true;
                }
            }
        }
    }

    // Fix: Correct id if it doesn't match filename prefix
    if let Some(id) = &doc.frontmatter.id {
        let expected_prefix = doc.doc_type.prefix();
        if !id.starts_with(expected_prefix) {
            // Extract date-seq from filename. `dash_pos` comes from `find`
            // so it's a valid char boundary; the 14-char slice below is
            // taken via `chars().take()` to stay safe if `after_type`
            // contains multi-byte characters.
            let name_no_ext = doc.filename.strip_suffix(".md").unwrap_or(&doc.filename);
            if let Some(dash_pos) = name_no_ext.find('-') {
                let after_type = &name_no_ext[dash_pos + 1..];
                let head: String = after_type.chars().take(14).collect();
                if head.chars().count() == 14 {
                    let new_id = format!("{}-{}", expected_prefix, head);
                    let old_id_line = format!("id: {}", id);
                    let new_id_line = format!("id: {}", new_id);
                    if new_content.contains(&old_id_line) {
                        new_content = new_content.replacen(&old_id_line, &new_id_line, 1);
                        modified = true;
                    }
                }
            }
        }
    }

    if modified {
        Some(new_content)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use straymark_core::document::Frontmatter;

    fn make_doc(filename: &str, doc_type: DocType, fm: Frontmatter, body: &str) -> StrayMarkDocument {
        StrayMarkDocument {
            path: PathBuf::from(format!(".straymark/test/{}", filename)),
            filename: filename.to_string(),
            doc_type,
            frontmatter: fm,
            body: body.to_string(),
        }
    }

    #[test]
    fn test_cross_001_high_risk_needs_review() {
        let fm = Frontmatter {
            id: Some("AILOG-2025-01-01-001".into()),
            risk_level: Some("high".into()),
            review_required: Some(false),
            ..Default::default()
        };
        let doc = make_doc("AILOG-2025-01-01-001-test.md", DocType::Ailog, fm, "");
        let mut result = ValidationResult::default();
        check_cross_rules(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "CROSS-001"));
    }

    #[test]
    fn test_cross_003_sec_needs_review() {
        let fm = Frontmatter {
            id: Some("SEC-2025-01-01-001".into()),
            review_required: Some(false),
            ..Default::default()
        };
        let doc = make_doc("SEC-2025-01-01-001-test.md", DocType::Sec, fm, "");
        let mut result = ValidationResult::default();
        check_cross_rules(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "CROSS-003"));
    }

    #[test]
    fn test_sec_001_sensitive_info() {
        let fm = Frontmatter::default();
        let doc = make_doc("AILOG-2025-01-01-001-test.md", DocType::Ailog, fm, "The api_key: sk-12345 was used");
        let mut result = ValidationResult::default();
        check_sensitive_info(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "SEC-001"));
    }

    #[test]
    fn test_type_001_inc_needs_severity() {
        let fm = Frontmatter {
            id: Some("INC-2025-01-01-001".into()),
            ..Default::default()
        };
        let doc = make_doc("INC-2025-01-01-001-test.md", DocType::Inc, fm, "");
        let mut result = ValidationResult::default();
        check_type_specific(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "TYPE-001"));
    }

    // ----- China regulatory rules -----

    #[test]
    fn test_cross_004_tc260_high_needs_review() {
        let fm = Frontmatter {
            id: Some("ETH-2026-04-25-001".into()),
            tc260_risk_level: Some("very_high".into()),
            review_required: Some(false),
            ..Default::default()
        };
        let doc = make_doc("ETH-2026-04-25-001-test.md", DocType::Eth, fm, "");
        let mut result = ValidationResult::default();
        check_china_cross_rules(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "CROSS-004"));
    }

    #[test]
    fn test_cross_005_pipl_sensitive_needs_pipia_link() {
        let fm = Frontmatter {
            id: Some("MCARD-2026-04-25-001".into()),
            pipl_sensitive_data: Some(true),
            related: Some(vec!["ETH-2026-04-25-001".into()]),
            ..Default::default()
        };
        let doc = make_doc("MCARD-2026-04-25-001-test.md", DocType::Mcard, fm, "");
        let mut result = ValidationResult::default();
        check_china_cross_rules(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "CROSS-005"));
    }

    #[test]
    fn test_cross_005_pipia_doc_itself_does_not_trigger() {
        let fm = Frontmatter {
            id: Some("PIPIA-2026-04-25-001".into()),
            pipl_sensitive_data: Some(true),
            ..Default::default()
        };
        let doc = make_doc("PIPIA-2026-04-25-001-test.md", DocType::Pipia, fm, "");
        let mut result = ValidationResult::default();
        check_china_cross_rules(&mut result, &doc);
        assert!(!result.errors.iter().any(|e| e.rule == "CROSS-005"));
    }

    #[test]
    fn test_cross_006_approved_needs_filing_number() {
        let fm = Frontmatter {
            id: Some("CACFILE-2026-04-25-001".into()),
            cac_filing_status: Some("national_approved".into()),
            cac_filing_number: None,
            ..Default::default()
        };
        let doc = make_doc("CACFILE-2026-04-25-001-test.md", DocType::Cacfile, fm, "");
        let mut result = ValidationResult::default();
        check_china_cross_rules(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "CROSS-006"));
    }

    #[test]
    fn test_cross_007_filing_required_needs_cacfile() {
        let fm = Frontmatter {
            id: Some("MCARD-2026-04-25-001".into()),
            cac_filing_required: Some(true),
            ..Default::default()
        };
        let doc = make_doc("MCARD-2026-04-25-001-test.md", DocType::Mcard, fm, "");
        let mut result = ValidationResult::default();
        check_china_cross_rules(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "CROSS-007"));
    }

    #[test]
    fn test_cross_008_particularly_serious_must_be_one_hour() {
        let fm = Frontmatter {
            id: Some("INC-2026-04-25-001".into()),
            csl_severity_level: Some("particularly_serious".into()),
            csl_report_deadline_hours: Some(4),
            ..Default::default()
        };
        let doc = make_doc("INC-2026-04-25-001-test.md", DocType::Inc, fm, "");
        let mut result = ValidationResult::default();
        check_china_cross_rules(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "CROSS-008"));
    }

    #[test]
    fn test_cross_009_relatively_major_must_be_four_hours() {
        let fm = Frontmatter {
            id: Some("INC-2026-04-25-001".into()),
            csl_severity_level: Some("relatively_major".into()),
            csl_report_deadline_hours: Some(24),
            ..Default::default()
        };
        let doc = make_doc("INC-2026-04-25-001-test.md", DocType::Inc, fm, "");
        let mut result = ValidationResult::default();
        check_china_cross_rules(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "CROSS-009"));
    }

    #[test]
    fn test_cross_010_gb45438_applicable_needs_ailabel() {
        let fm = Frontmatter {
            id: Some("MCARD-2026-04-25-001".into()),
            gb45438_applicable: Some(true),
            ..Default::default()
        };
        let doc = make_doc("MCARD-2026-04-25-001-test.md", DocType::Mcard, fm, "");
        let mut result = ValidationResult::default();
        check_china_cross_rules(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "CROSS-010"));
    }

    #[test]
    fn test_type_003_pipia_retention_three_years() {
        let fm = Frontmatter {
            id: Some("PIPIA-2026-04-25-001".into()),
            created: Some("2026-04-25".into()),
            pipl_retention_until: Some("2027-04-25".into()), // only 1 year — must fail
            ..Default::default()
        };
        let doc = make_doc("PIPIA-2026-04-25-001-test.md", DocType::Pipia, fm, "");
        let mut result = ValidationResult::default();
        check_china_type_specific(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "TYPE-003"));
    }

    #[test]
    fn test_type_003_pipia_retention_three_years_ok() {
        let fm = Frontmatter {
            id: Some("PIPIA-2026-04-25-001".into()),
            created: Some("2026-04-25".into()),
            pipl_retention_until: Some("2029-04-25".into()), // exactly 3 years
            ..Default::default()
        };
        let doc = make_doc("PIPIA-2026-04-25-001-test.md", DocType::Pipia, fm, "");
        let mut result = ValidationResult::default();
        check_china_type_specific(&mut result, &doc);
        assert!(!result.errors.iter().any(|e| e.rule == "TYPE-003"));
    }

    #[test]
    fn test_type_005_tc260ra_requires_three_criteria() {
        let fm = Frontmatter {
            id: Some("TC260RA-2026-04-25-001".into()),
            tc260_application_scenario: Some("healthcare".into()),
            // missing intelligence_level and application_scale
            ..Default::default()
        };
        let doc = make_doc("TC260RA-2026-04-25-001-test.md", DocType::Tc260ra, fm, "");
        let mut result = ValidationResult::default();
        check_china_type_specific(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "TYPE-005"));
    }

    #[test]
    fn test_type_006_ailabel_needs_content_type() {
        let fm = Frontmatter {
            id: Some("AILABEL-2026-04-25-001".into()),
            gb45438_content_types: Some(vec![]),
            ..Default::default()
        };
        let doc = make_doc("AILABEL-2026-04-25-001-test.md", DocType::Ailabel, fm, "");
        let mut result = ValidationResult::default();
        check_china_type_specific(&mut result, &doc);
        assert!(result.errors.iter().any(|e| e.rule == "TYPE-006"));
    }

    // ----- OBS-001: observability content (#215 Gap 1) -----

    fn obs_doc(body: &str) -> StrayMarkDocument {
        let fm = Frontmatter {
            id: Some("AILOG-2026-04-25-001".into()),
            tags: Some(vec!["observabilidad".into()]),
            ..Default::default()
        };
        make_doc("AILOG-2026-04-25-001-test.md", DocType::Ailog, fm, body)
    }

    #[test]
    fn test_obs_001_flags_tagged_doc_without_content() {
        let doc = obs_doc("This change refactors the parser and adds a CLI flag.");
        let mut result = ValidationResult::default();
        check_observability(&mut result, &doc);
        assert!(result.warnings.iter().any(|w| w.rule == "OBS-001"));
    }

    #[test]
    fn test_obs_001_accepts_broadened_vocabulary() {
        // Mixed ES/EN observability terms that the old literal set missed (#215).
        for body in [
            "Added OTel spans to the request path.",
            "New Grafana dashboard for p99 latency.",
            "Configuré el otel-collector y métricas de histograma.",
            "Emits a histogram metric and a trace per job.",
            "slog structured logging wired to Cloud Monitoring alert policies.",
        ] {
            let doc = obs_doc(body);
            let mut result = ValidationResult::default();
            check_observability(&mut result, &doc);
            assert!(
                !result.warnings.iter().any(|w| w.rule == "OBS-001"),
                "OBS-001 should not fire for body: {body:?}"
            );
        }
    }

    #[test]
    fn test_obs_001_skips_untagged_doc() {
        let fm = Frontmatter {
            id: Some("AILOG-2026-04-25-002".into()),
            ..Default::default()
        };
        let doc = make_doc("AILOG-2026-04-25-002-test.md", DocType::Ailog, fm, "no tag here");
        let mut result = ValidationResult::default();
        check_observability(&mut result, &doc);
        assert!(!result.warnings.iter().any(|w| w.rule == "OBS-001"));
    }

    // ----- META-003: status vocabulary suggestions (#215 Gap 4) -----

    #[test]
    fn test_suggest_status_semantic_aliases() {
        assert_eq!(suggest_status("done"), Some("accepted"));
        assert_eq!(suggest_status("completed"), Some("accepted"));
        assert_eq!(suggest_status("in-progress"), Some("accepted"));
        assert_eq!(suggest_status("WIP"), Some("accepted")); // case-insensitive
        assert_eq!(suggest_status("obsolete"), Some("superseded"));
    }

    #[test]
    fn test_suggest_status_typo_fallback() {
        assert_eq!(suggest_status("acepted"), Some("accepted")); // distance 1
        assert_eq!(suggest_status("draftt"), Some("draft")); // distance 1
        assert_eq!(suggest_status("zzzzzzzz"), None); // nothing close
    }

    #[test]
    fn test_meta_003_invalid_status_carries_suggestion() {
        let fm = Frontmatter {
            id: Some("AILOG-2026-04-25-003".into()),
            status: Some("done".into()),
            ..Default::default()
        };
        let doc = make_doc("AILOG-2026-04-25-003-test.md", DocType::Ailog, fm, "");
        let mut result = ValidationResult::default();
        check_valid_status(&mut result, &doc);
        let issue = result
            .errors
            .iter()
            .find(|e| e.rule == "META-003")
            .expect("META-003 expected");
        assert!(issue.message.contains("Did you mean 'accepted'?"), "msg: {}", issue.message);
        assert!(issue.fix_hint.as_deref().unwrap_or("").contains("accepted"));
    }

    #[test]
    fn test_meta_003_accepts_canonical_status() {
        let fm = Frontmatter {
            id: Some("AILOG-2026-04-25-004".into()),
            status: Some("accepted".into()),
            ..Default::default()
        };
        let doc = make_doc("AILOG-2026-04-25-004-test.md", DocType::Ailog, fm, "");
        let mut result = ValidationResult::default();
        check_valid_status(&mut result, &doc);
        assert!(!result.errors.iter().any(|e| e.rule == "META-003"));
    }
}
