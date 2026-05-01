//! Charter — DevTrail's bounded, auditable unit of work.
//!
//! A Charter is the artifact that pairs declarative ex-ante scope with ex-post
//! drift detection and external audit. The schema lives in
//! `dist/.devtrail/schemas/charter.schema.v0.json` (experimental v0). This
//! module provides typed access to a Charter document on disk.
//!
//! Conceptually distinct from `DocType` (governance documents): Charters live
//! at `docs/charters/NN-slug.md` (project-root level), use sequential filenames
//! without a date prefix, and their schema is its own contract. See
//! `Propuesta/que-es-un-charter.md` for the conceptual scope.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Lifecycle status of a Charter. Source of truth — the prose status mirror in
/// the body is decorative.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CharterStatus {
    Declared,
    InProgress,
    Closed,
}

impl CharterStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::InProgress => "in-progress",
            Self::Closed => "closed",
        }
    }
}

/// Time-based effort estimate. Sentinel /plan-audit validated this as predictive
/// in 4/5 cycles; line count is intentionally not tracked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffortEstimate {
    #[serde(rename = "XS")]
    Xs,
    #[serde(rename = "S")]
    S,
    #[serde(rename = "M")]
    M,
    #[serde(rename = "L")]
    L,
}

impl EffortEstimate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Xs => "XS",
            Self::S => "S",
            Self::M => "M",
            Self::L => "L",
        }
    }
}

/// Typed view of a Charter's YAML frontmatter. The JSON Schema in
/// `dist/.devtrail/schemas/charter.schema.v0.json` is the source of truth for
/// shape; this struct is the typed Rust mirror used by CLI logic. Additional
/// fields the schema permits (e.g., `note`, `closed_at`) are not enumerated
/// here — for full schema-aware validation, parse the frontmatter as
/// `serde_yaml::Value` and use `crate::charter_schema::validate_frontmatter`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharterFrontmatter {
    pub charter_id: String,
    pub status: CharterStatus,
    pub effort_estimate: EffortEstimate,
    pub trigger: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub originating_ailogs: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub originating_spec: Option<String>,
}

/// A parsed Charter document from disk.
///
/// For schema-aware access to fields the typed struct does not enumerate
/// (e.g., `note`, `closed_at`), use `read_frontmatter_yaml` instead — it
/// returns the raw `serde_yaml::Value` without typed deserialization.
#[derive(Debug, Clone)]
pub struct Charter {
    pub path: PathBuf,
    pub frontmatter: CharterFrontmatter,
    pub body: String,
}

/// Parse a Charter document from disk. Returns an error if the file has no
/// YAML frontmatter or required fields are missing/wrong-typed.
pub fn parse_charter(path: &Path) -> Result<Charter> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read Charter file {}", path.display()))?;
    parse_charter_str(path, &content)
}

/// Parse a Charter from raw content (split out for testability).
pub fn parse_charter_str(path: &Path, content: &str) -> Result<Charter> {
    let (frontmatter_str, body) = split_frontmatter(content).ok_or_else(|| {
        anyhow!(
            "Charter at {} has no YAML frontmatter (expected --- delimiters at top of file)",
            path.display()
        )
    })?;
    let frontmatter: CharterFrontmatter = serde_yaml::from_str(frontmatter_str)
        .with_context(|| {
            format!(
                "Charter frontmatter at {} is missing required fields or has wrong types. \
                 Required: charter_id, status (declared|in-progress|closed), effort_estimate (XS|S|M|L), trigger.",
                path.display()
            )
        })?;
    Ok(Charter {
        path: path.to_path_buf(),
        frontmatter,
        body: body.to_string(),
    })
}

/// Discover all Charter files in a project. Charters live at `docs/charters/*.md`
/// with filenames `NN-slug.md` (sequential prefix). Returns paths sorted by
/// filename so callers get stable ordering. Files that don't match the
/// `NN-slug.md` pattern are skipped (e.g., a stray `README.md`).
pub fn discover_charters(project_root: &Path) -> Vec<PathBuf> {
    let dir = project_root.join("docs").join("charters");
    if !dir.exists() {
        return Vec::new();
    }
    let mut paths: Vec<PathBuf> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| is_charter_filename(p))
            .collect(),
        Err(_) => return Vec::new(),
    };
    paths.sort();
    paths
}

/// True if the file matches the Charter filename pattern `NN-*.md` where NN is
/// one or more digits.
fn is_charter_filename(p: &Path) -> bool {
    if p.extension().and_then(|e| e.to_str()) != Some("md") {
        return false;
    }
    let name = match p.file_name().and_then(|n| n.to_str()) {
        Some(n) => n,
        None => return false,
    };
    let leading: String = name.chars().take_while(|c| c.is_ascii_digit()).collect();
    !leading.is_empty() && name[leading.len()..].starts_with('-')
}

/// Read and parse just the YAML frontmatter from a Charter file, without
/// applying the typed `CharterFrontmatter` deserialization. Used by the
/// schema validator so shape errors (bad enum, missing required field) are
/// reported by the schema itself with rich hints, rather than being masked
/// by typed-parse failures.
pub fn read_frontmatter_yaml(path: &Path) -> Result<serde_yaml::Value> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let (fm_str, _body) = split_frontmatter(&content).ok_or_else(|| {
        anyhow!(
            "Charter at {} has no YAML frontmatter (expected --- delimiters at top of file)",
            path.display()
        )
    })?;
    serde_yaml::from_str(fm_str)
        .with_context(|| format!("Failed to parse Charter frontmatter at {}", path.display()))
}

/// Discover all Charters in a project and parse each one. Returns the parsed
/// Charters along with a list of `(path, error)` for files that failed to parse.
/// Callers decide whether to fail (status sub-command on a specific ID) or
/// display a degraded view (list sub-command — show the parseable ones, warn
/// about the rest).
pub fn discover_and_parse(project_root: &Path) -> (Vec<Charter>, Vec<(PathBuf, anyhow::Error)>) {
    let mut parsed = Vec::new();
    let mut errors = Vec::new();
    for path in discover_charters(project_root) {
        match parse_charter(&path) {
            Ok(c) => parsed.push(c),
            Err(e) => errors.push((path, e)),
        }
    }
    (parsed, errors)
}

/// Find a Charter in a slice by user-supplied ID. Accepts three forms:
/// - The full charter_id (`CHARTER-01-per-service-anomaly-thresholds`)
/// - The CHARTER-NN prefix (`CHARTER-01`)
/// - Just the numeric NN (`01` or `1`)
///
/// Returns the matching Charter or None. When multiple Charters share the
/// same NN (which should not happen in a healthy repo), the first in the
/// slice wins — call sites typically receive Charters sorted by filename so
/// "first" is the one with the lexicographically lowest slug.
pub fn find_by_id<'a>(charters: &'a [Charter], id_input: &str) -> Option<&'a Charter> {
    let trimmed = id_input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Exact match on charter_id.
    if let Some(c) = charters
        .iter()
        .find(|c| c.frontmatter.charter_id == trimmed)
    {
        return Some(c);
    }

    // CHARTER-NN[-slug] prefix-with-boundary match (CHARTER-01 matches
    // CHARTER-01-anything but not CHARTER-010).
    if trimmed.starts_with("CHARTER-") {
        if let Some(c) = charters.iter().find(|c| {
            let cid = &c.frontmatter.charter_id;
            cid.starts_with(trimmed)
                && (cid.len() == trimmed.len()
                    || cid.as_bytes().get(trimmed.len()) == Some(&b'-'))
        }) {
            return Some(c);
        }
    }

    // Numeric NN match — compares parsed integers, so "10" matches both
    // CHARTER-10-x and CHARTER-010-x (same numerical NN, different padding).
    // String-prefix match (above) is what disambiguates when both exist.
    if let Ok(n) = trimmed.parse::<u32>() {
        if let Some(c) = charters.iter().find(|c| {
            let cid = &c.frontmatter.charter_id;
            let after_prefix = cid.strip_prefix("CHARTER-").unwrap_or(cid);
            let digits: String = after_prefix
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect();
            digits.parse::<u32>().ok() == Some(n)
        }) {
            return Some(c);
        }
    }

    None
}

/// Extract a display title from a Charter's body. Looks for an H1 line of the
/// form `# Charter: <title>` and returns `<title>`. Falls back to the slug
/// portion of the filename if the H1 is missing or doesn't match.
pub fn display_title(charter: &Charter) -> String {
    for line in charter.body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# Charter:") {
            let t = rest.trim();
            if !t.is_empty() && !t.starts_with('[') {
                return t.to_string();
            }
        }
    }
    // Fallback: derive from filename slug.
    charter
        .path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|stem| {
            // Strip leading digits + dash.
            let after_digits = stem.trim_start_matches(|c: char| c.is_ascii_digit());
            after_digits.strip_prefix('-').or(Some(after_digits))
        })
        .map(|s| s.replace('-', " "))
        .unwrap_or_else(|| "(untitled)".to_string())
}

/// Categorize a Charter's origin for filtering. Returns one of "ailog", "spec",
/// or "none".
pub fn origin_kind(fm: &CharterFrontmatter) -> &'static str {
    if fm.originating_ailogs.is_some() {
        "ailog"
    } else if fm.originating_spec.is_some() {
        "spec"
    } else {
        "none"
    }
}

/// Render the origin field for display. Returns the AILOG ID(s) joined or the
/// spec path, or "—" when no origin is set.
pub fn display_origin(fm: &CharterFrontmatter) -> String {
    if let Some(ailogs) = &fm.originating_ailogs {
        ailogs.join(", ")
    } else if let Some(spec) = &fm.originating_spec {
        spec.clone()
    } else {
        "—".to_string()
    }
}

/// Determine the next sequential Charter number for a project. Reads
/// `docs/charters/` and returns `max(NN) + 1`, or 1 if no Charters exist yet.
///
/// Note: this is intentionally racy on parallel branches (R2 in the Phase 1
/// plan). Two contributors running `charter new` simultaneously can both
/// receive NN=03; the conflict surfaces at merge time. Phase 2+ may add
/// atomic numbering via a lock file; v0 documents the caveat.
pub fn next_charter_number(project_root: &Path) -> u32 {
    discover_charters(project_root)
        .iter()
        .filter_map(charter_number_from_path)
        .max()
        .map(|n| n + 1)
        .unwrap_or(1)
}

/// Extract the leading digits from a Charter filename (`05-foo.md` → 5).
fn charter_number_from_path(p: &PathBuf) -> Option<u32> {
    let name = p.file_name()?.to_str()?;
    let prefix: String = name.chars().take_while(|c| c.is_ascii_digit()).collect();
    if prefix.is_empty() {
        return None;
    }
    prefix.parse::<u32>().ok()
}

/// Split a markdown document into (frontmatter, body) at the first pair of `---`
/// delimiters. Returns `None` if the document has no frontmatter block.
/// Accepts both `\n` and `\r\n` line endings on the closing delimiter.
fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    // Opening delimiter: must be at the very start of the file.
    let after_open = content.strip_prefix("---\n").or_else(|| content.strip_prefix("---\r\n"))?;
    // Closing delimiter: find the first occurrence of `\n---\n` or `\r\n---\r\n`.
    let (end, delim_len) = if let Some(idx) = after_open.find("\n---\n") {
        (idx, 5)
    } else if let Some(idx) = after_open.find("\r\n---\r\n") {
        (idx, 7)
    } else if let Some(idx) = after_open.find("\n---\r\n") {
        (idx, 6)
    } else {
        return None;
    };
    let frontmatter = &after_open[..end];
    let body = &after_open[end + delim_len..];
    Some((frontmatter, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    const VALID_FRONTMATTER: &str = r#"---
charter_id: CHARTER-01-test
status: declared
effort_estimate: M
trigger: "test trigger"
---

# Charter: Test
Body content.
"#;

    #[test]
    fn parse_minimal_valid_charter() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("01-test.md");
        write(&p, VALID_FRONTMATTER);
        let charter = parse_charter(&p).unwrap();
        assert_eq!(charter.frontmatter.charter_id, "CHARTER-01-test");
        assert_eq!(charter.frontmatter.status, CharterStatus::Declared);
        assert_eq!(charter.frontmatter.effort_estimate, EffortEstimate::M);
        assert_eq!(charter.frontmatter.trigger, "test trigger");
        assert!(charter.frontmatter.originating_ailogs.is_none());
        assert!(charter.frontmatter.originating_spec.is_none());
        assert!(charter.body.contains("# Charter: Test"));
    }

    #[test]
    fn parse_with_ailogs_origin() {
        let content = r#"---
charter_id: CHARTER-02-with-ailog
status: in-progress
effort_estimate: S
trigger: "follow-up triggered"
originating_ailogs:
  - AILOG-2026-04-28-021
  - AILOG-2026-04-28-022
---

Body.
"#;
        let charter = parse_charter_str(Path::new("02-with-ailog.md"), content).unwrap();
        assert_eq!(
            charter.frontmatter.originating_ailogs.as_deref(),
            Some(&["AILOG-2026-04-28-021".to_string(), "AILOG-2026-04-28-022".to_string()][..])
        );
    }

    #[test]
    fn parse_with_spec_origin() {
        let content = r#"---
charter_id: CHARTER-03-from-spec
status: declared
effort_estimate: L
trigger: "from spec"
originating_spec: specs/001-feature/spec.md
---

Body.
"#;
        let charter = parse_charter_str(Path::new("03-from-spec.md"), content).unwrap();
        assert_eq!(
            charter.frontmatter.originating_spec.as_deref(),
            Some("specs/001-feature/spec.md")
        );
    }

    #[test]
    fn parse_ignores_unknown_fields_silently() {
        // Fields the typed struct does not enumerate (e.g., `note`, `closed_at`)
        // must not cause a parse error — the schema controls their shape via
        // additionalProperties: true. Use read_frontmatter_yaml when you need
        // schema-aware access to those fields.
        let content = r#"---
charter_id: CHARTER-04-extras
status: closed
effort_estimate: XS
trigger: "x"
note: "this is an example"
closed_at: "2026-04-30"
---

Body.
"#;
        let charter = parse_charter_str(Path::new("04-extras.md"), content).unwrap();
        assert_eq!(charter.frontmatter.charter_id, "CHARTER-04-extras");
        assert_eq!(charter.frontmatter.status, CharterStatus::Closed);
    }

    #[test]
    fn read_frontmatter_yaml_preserves_unknown_fields() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("04-extras.md");
        write(
            &p,
            "---\ncharter_id: CHARTER-04-extras\nstatus: closed\neffort_estimate: XS\ntrigger: x\nnote: example\nclosed_at: \"2026-04-30\"\n---\n\nBody.\n",
        );
        let yaml = read_frontmatter_yaml(&p).unwrap();
        let map = yaml.as_mapping().unwrap();
        assert_eq!(
            map.get(serde_yaml::Value::String("note".into()))
                .and_then(|v| v.as_str()),
            Some("example")
        );
    }

    #[test]
    fn parse_fails_without_frontmatter() {
        let content = "# Just a markdown file\n\nNo frontmatter.\n";
        let err = parse_charter_str(Path::new("bad.md"), content).unwrap_err();
        assert!(err.to_string().contains("no YAML frontmatter"));
    }

    #[test]
    fn parse_fails_with_missing_required_field() {
        let content = r#"---
charter_id: CHARTER-05
status: declared
effort_estimate: M
---

Missing trigger.
"#;
        let err = parse_charter_str(Path::new("missing.md"), content).unwrap_err();
        assert!(
            err.to_string().contains("missing")
                || err.to_string().contains("trigger")
                || err.chain().any(|c| c.to_string().contains("trigger"))
        );
    }

    #[test]
    fn parse_fails_with_invalid_status_enum() {
        let content = r#"---
charter_id: CHARTER-06
status: unknown-state
effort_estimate: M
trigger: "x"
---

Body.
"#;
        let err = parse_charter_str(Path::new("bad-status.md"), content).unwrap_err();
        // serde reports "unknown variant" or similar.
        let chain: Vec<String> = err.chain().map(|c| c.to_string()).collect();
        assert!(
            chain.iter().any(|s| s.contains("unknown") || s.contains("variant") || s.contains("status")),
            "unexpected error chain: {:?}",
            chain
        );
    }

    #[test]
    fn discover_returns_empty_when_dir_missing() {
        let tmp = TempDir::new().unwrap();
        assert!(discover_charters(tmp.path()).is_empty());
    }

    #[test]
    fn discover_returns_sorted_charter_files_only() {
        let tmp = TempDir::new().unwrap();
        let charters_dir = tmp.path().join("docs").join("charters");
        write(&charters_dir.join("03-third.md"), VALID_FRONTMATTER);
        write(&charters_dir.join("01-first.md"), VALID_FRONTMATTER);
        write(&charters_dir.join("02-second.md"), VALID_FRONTMATTER);
        // Non-Charter files that should be ignored.
        write(&charters_dir.join("README.md"), "# Charters\n");
        write(&charters_dir.join("notes.txt"), "ignored");
        write(&charters_dir.join("draft.md"), "no leading digits");

        let found = discover_charters(tmp.path());
        let names: Vec<&str> = found
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(names, vec!["01-first.md", "02-second.md", "03-third.md"]);
    }

    #[test]
    fn next_number_is_one_when_empty() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(next_charter_number(tmp.path()), 1);
    }

    #[test]
    fn next_number_is_max_plus_one() {
        let tmp = TempDir::new().unwrap();
        let charters_dir = tmp.path().join("docs").join("charters");
        write(&charters_dir.join("01-a.md"), VALID_FRONTMATTER);
        write(&charters_dir.join("05-b.md"), VALID_FRONTMATTER);
        write(&charters_dir.join("03-c.md"), VALID_FRONTMATTER);
        assert_eq!(next_charter_number(tmp.path()), 6);
    }

    #[test]
    fn next_number_skips_non_numbered_files() {
        let tmp = TempDir::new().unwrap();
        let charters_dir = tmp.path().join("docs").join("charters");
        write(&charters_dir.join("01-a.md"), VALID_FRONTMATTER);
        write(&charters_dir.join("README.md"), "# Charters\n");
        write(&charters_dir.join("draft-no-number.md"), VALID_FRONTMATTER);
        assert_eq!(next_charter_number(tmp.path()), 2);
    }

    #[test]
    fn split_frontmatter_handles_unix_line_endings() {
        let (fm, body) = split_frontmatter("---\nfoo: bar\n---\nbody\n").unwrap();
        assert_eq!(fm, "foo: bar");
        assert_eq!(body, "body\n");
    }

    #[test]
    fn split_frontmatter_returns_none_without_block() {
        assert!(split_frontmatter("# no frontmatter\n").is_none());
        assert!(split_frontmatter("---\nfoo: bar\n").is_none()); // no closing
    }

    #[test]
    fn status_as_str_matches_schema_enum_values() {
        assert_eq!(CharterStatus::Declared.as_str(), "declared");
        assert_eq!(CharterStatus::InProgress.as_str(), "in-progress");
        assert_eq!(CharterStatus::Closed.as_str(), "closed");
    }

    #[test]
    fn effort_as_str_matches_schema_enum_values() {
        assert_eq!(EffortEstimate::Xs.as_str(), "XS");
        assert_eq!(EffortEstimate::S.as_str(), "S");
        assert_eq!(EffortEstimate::M.as_str(), "M");
        assert_eq!(EffortEstimate::L.as_str(), "L");
    }

    fn make_charter(id: &str, status: CharterStatus, origin: Origin, body: &str) -> Charter {
        let fm = CharterFrontmatter {
            charter_id: id.to_string(),
            status,
            effort_estimate: EffortEstimate::M,
            trigger: "x".to_string(),
            originating_ailogs: match &origin {
                Origin::Ailog(ids) => Some(ids.clone()),
                _ => None,
            },
            originating_spec: match &origin {
                Origin::Spec(s) => Some(s.clone()),
                _ => None,
            },
        };
        // Build a realistic filename: NN-slug.md (mirrors what `charter new` produces).
        // Strip the CHARTER- prefix from the ID to derive the on-disk filename.
        let filename = id
            .strip_prefix("CHARTER-")
            .map(|rest| format!("{}.md", rest.to_lowercase()))
            .unwrap_or_else(|| format!("{}.md", id.to_lowercase()));
        Charter {
            path: PathBuf::from(format!("docs/charters/{}", filename)),
            frontmatter: fm,
            body: body.to_string(),
        }
    }

    enum Origin {
        Ailog(Vec<String>),
        Spec(String),
        None,
    }

    #[test]
    fn find_by_id_exact_match() {
        let charters = vec![
            make_charter("CHARTER-01-foo", CharterStatus::Declared, Origin::None, ""),
            make_charter("CHARTER-02-bar", CharterStatus::Closed, Origin::None, ""),
        ];
        let found = find_by_id(&charters, "CHARTER-02-bar").unwrap();
        assert_eq!(found.frontmatter.charter_id, "CHARTER-02-bar");
    }

    #[test]
    fn find_by_id_charter_nn_prefix() {
        let charters = vec![
            make_charter("CHARTER-01-foo", CharterStatus::Declared, Origin::None, ""),
            make_charter("CHARTER-02-bar", CharterStatus::Closed, Origin::None, ""),
        ];
        let found = find_by_id(&charters, "CHARTER-01").unwrap();
        assert_eq!(found.frontmatter.charter_id, "CHARTER-01-foo");
    }

    #[test]
    fn find_by_id_numeric() {
        let charters = vec![
            make_charter("CHARTER-01-foo", CharterStatus::Declared, Origin::None, ""),
            make_charter("CHARTER-02-bar", CharterStatus::Closed, Origin::None, ""),
        ];
        assert_eq!(find_by_id(&charters, "1").unwrap().frontmatter.charter_id, "CHARTER-01-foo");
        assert_eq!(find_by_id(&charters, "01").unwrap().frontmatter.charter_id, "CHARTER-01-foo");
        assert_eq!(find_by_id(&charters, "2").unwrap().frontmatter.charter_id, "CHARTER-02-bar");
    }

    #[test]
    fn find_by_id_no_match_returns_none() {
        let charters = vec![make_charter(
            "CHARTER-01-foo",
            CharterStatus::Declared,
            Origin::None,
            "",
        )];
        assert!(find_by_id(&charters, "CHARTER-99").is_none());
        assert!(find_by_id(&charters, "99").is_none());
        assert!(find_by_id(&charters, "PLAN-01").is_none());
        assert!(find_by_id(&charters, "").is_none());
    }

    #[test]
    fn find_by_id_prefix_does_not_match_longer_number() {
        // CHARTER-01 must NOT match CHARTER-010.
        let charters = vec![make_charter(
            "CHARTER-010-extended",
            CharterStatus::Declared,
            Origin::None,
            "",
        )];
        assert!(find_by_id(&charters, "CHARTER-01").is_none());
        assert!(find_by_id(&charters, "1").is_none());
        assert_eq!(
            find_by_id(&charters, "10").unwrap().frontmatter.charter_id,
            "CHARTER-010-extended"
        );
    }

    #[test]
    fn display_title_extracts_h1() {
        let c = make_charter(
            "CHARTER-01-x",
            CharterStatus::Declared,
            Origin::None,
            "# Charter: My Real Title\n\nbody text\n",
        );
        assert_eq!(display_title(&c), "My Real Title");
    }

    #[test]
    fn display_title_falls_back_to_filename_when_h1_is_placeholder() {
        let c = make_charter(
            "CHARTER-01-foo-bar",
            CharterStatus::Declared,
            Origin::None,
            "# Charter: [BRIEF TITLE]\n\nbody\n",
        );
        // The body H1 still contains the placeholder, so we fall back.
        assert_eq!(display_title(&c), "foo bar");
    }

    #[test]
    fn display_title_falls_back_when_body_lacks_h1() {
        let c = make_charter(
            "CHARTER-01-foo-bar",
            CharterStatus::Declared,
            Origin::None,
            "no h1 in this body\n",
        );
        assert_eq!(display_title(&c), "foo bar");
    }

    #[test]
    fn origin_kind_categorizes_correctly() {
        let with_ailog = make_charter(
            "CHARTER-01-x",
            CharterStatus::Declared,
            Origin::Ailog(vec!["AILOG-2026-04-28-021".into()]),
            "",
        );
        assert_eq!(origin_kind(&with_ailog.frontmatter), "ailog");

        let with_spec = make_charter(
            "CHARTER-02-x",
            CharterStatus::Declared,
            Origin::Spec("specs/001/spec.md".into()),
            "",
        );
        assert_eq!(origin_kind(&with_spec.frontmatter), "spec");

        let with_none = make_charter("CHARTER-03-x", CharterStatus::Declared, Origin::None, "");
        assert_eq!(origin_kind(&with_none.frontmatter), "none");
    }

    #[test]
    fn display_origin_renders_each_kind() {
        let with_ailog = make_charter(
            "CHARTER-01-x",
            CharterStatus::Declared,
            Origin::Ailog(vec!["AILOG-2026-04-28-021".into(), "AILOG-2026-04-28-022".into()]),
            "",
        );
        assert_eq!(
            display_origin(&with_ailog.frontmatter),
            "AILOG-2026-04-28-021, AILOG-2026-04-28-022"
        );

        let with_spec = make_charter(
            "CHARTER-02-x",
            CharterStatus::Declared,
            Origin::Spec("specs/001/spec.md".into()),
            "",
        );
        assert_eq!(display_origin(&with_spec.frontmatter), "specs/001/spec.md");

        let with_none = make_charter("CHARTER-03-x", CharterStatus::Declared, Origin::None, "");
        assert_eq!(display_origin(&with_none.frontmatter), "—");
    }

    #[test]
    fn discover_and_parse_separates_good_and_bad() {
        let tmp = TempDir::new().unwrap();
        let charters_dir = tmp.path().join("docs").join("charters");
        write(&charters_dir.join("01-good.md"), VALID_FRONTMATTER);
        // A file with leading digits + dash but broken frontmatter.
        write(
            &charters_dir.join("02-bad.md"),
            "no frontmatter at all\n",
        );
        let (parsed, errors) = discover_and_parse(tmp.path());
        assert_eq!(parsed.len(), 1);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].0.file_name().unwrap().to_str().unwrap().contains("02-bad"));
    }
}
