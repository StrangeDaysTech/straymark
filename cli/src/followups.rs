//! Follow-ups backlog registry — StrayMark's first-class pending-work artifact.
//!
//! The registry is a single markdown file at `.straymark/follow-ups-backlog.md`
//! aggregating `§Follow-ups` and `R<N> (new, not in Charter)` entries across
//! AILOGs. Schema: `dist/.straymark/schemas/follow-ups-backlog.schema.v1.json`
//! (experimental v1). Convention: `FOLLOW-UPS-BACKLOG-PATTERN.md` and
//! `STRAYMARK.md §16`.
//!
//! Conceptually distinct from `DocType` (governance documents) and from
//! Charters: one registry per project, entries (`FU-NNN`) live inside it as
//! semi-structured markdown blocks under `## Bucket: <name>` headings.
//!
//! Parsing is **lenient by design** (ADR-2026-06-03-001): a missing field is
//! `None`, never an error; an unparseable `### FU-` block becomes a warning,
//! never a failure; v0 registries (no v1 fields) parse identically. Write
//! operations (`drift --apply`, `promote`) are surgical text edits that
//! preserve unknown frontmatter fields and untouched body content — the
//! v0 → v1 upgrade rewrites only the version marker and the counters.
//!
//! Move target: straymark-core (Loom M0) — keep this module free of
//! clap/colored/dialoguer; presentation lives in `commands/followups/`.

use anyhow::{anyhow, Context, Result, bail};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Canonical registry location, relative to the project root.
pub fn registry_path(project_root: &Path) -> PathBuf {
    project_root.join(".straymark").join("follow-ups-backlog.md")
}

/// Entry lifecycle status. `SuspectedClosed` is new in schema v1 — assigned by
/// `drift --apply` when the source AILOG text carries an explicit closure
/// marker (see [`has_closure_marker`]). `Unknown` preserves lenient parsing:
/// an unrecognized value never fails, it just doesn't count toward any bucket
/// of the recomputed counters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuStatus {
    Open,
    InProgress,
    SuspectedClosed,
    Closed,
    Superseded,
    Promoted,
    Unknown,
}

impl FuStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in-progress",
            Self::SuspectedClosed => "suspected-closed",
            Self::Closed => "closed",
            Self::Superseded => "superseded",
            Self::Promoted => "promoted",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str_loose(s: &str) -> Self {
        let exact = |v: &str| match v {
            "open" => Self::Open,
            "in-progress" | "in progress" => Self::InProgress,
            "suspected-closed" | "suspected closed" => Self::SuspectedClosed,
            "closed" => Self::Closed,
            "superseded" => Self::Superseded,
            "promoted" => Self::Promoted,
            _ => Self::Unknown,
        };
        let lower = s.trim().to_lowercase();
        let parsed = exact(&lower);
        if parsed != Self::Unknown {
            return parsed;
        }
        // Leniency fallback (cli-3.19.1, found validating against the
        // Sentinel production registry): operators annotate the status value
        // in place — `open — **OVERDUE** (…)`, `open — mitigation in place`.
        // Take the first whitespace-delimited token and rematch, so the
        // annotation idiom doesn't demote real statuses to Unknown (which
        // would undercount the CLI-owned `total_open` on migration).
        match lower.split_whitespace().next() {
            Some(first) => exact(first),
            None => Self::Unknown,
        }
    }
}

/// Entry severity (v1, optional). `blocking` canonicalizes the `PROD-BLOCKER`
/// prose convention from the reference adopter (issue #214 Signal 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Normal,
    Blocking,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Blocking => "blocking",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        let exact = |v: &str| match v {
            "normal" => Some(Self::Normal),
            "blocking" | "prod-blocker" => Some(Self::Blocking),
            _ => None,
        };
        let lower = s.trim().to_lowercase();
        // Same first-token annotation fallback as FuStatus::from_str_loose.
        exact(&lower).or_else(|| lower.split_whitespace().next().and_then(exact))
    }
}

/// One `### FU-NNN — <description>` block inside a section. Byte spans are
/// offsets into `Registry::body` so write operations can edit surgically.
#[derive(Debug, Clone)]
pub struct Entry {
    pub fu_id: String,
    pub fu_number: u32,
    pub description: String,
    /// Section (bucket) this entry lives under, e.g. "ready". Non-bucket
    /// sections (e.g. "Promoted to TDE") carry their heading text instead.
    pub bucket: String,
    pub origin: Option<String>,
    pub origin_class: Option<String>,
    /// Stable content hash of the source follow-up (`fu_content_hash`), stored
    /// by `drift --apply` so a re-scan can dedupe by content identity instead
    /// of skipping the whole AILOG (#231). Absent on legacy (pre-fix) entries —
    /// drift falls back to recomputing the hash from `origin` + `description`.
    pub source_hash: Option<String>,
    pub status: FuStatus,
    pub status_raw: Option<String>,
    pub severity: Option<Severity>,
    pub trigger: Option<String>,
    pub destination: Option<String>,
    pub cost: Option<String>,
    pub labels: Vec<String>,
    pub notes: Option<String>,
    pub promoted_to: Option<String>,
    /// The load-bearing assumption the entry rests on, stated so it can be
    /// re-checked in seconds at execution (AIDEC-2026-07-18-001). Optional; an
    /// entry is a *dated hypothesis*, and this is the hypothesis to re-test
    /// before acting. Absent on entries authored before the field existed.
    pub premise: Option<String>,
    /// Date the `premise` was last re-verified against reality, stamped by
    /// `followups verify`/`promote --premise-verified`. Absent = never
    /// re-checked since capture (the default, honest state).
    pub verified_at: Option<String>,
    /// Byte offset of the `### ` heading line start, into `Registry::body`.
    pub span_start: usize,
    /// Byte offset one past the entry's last byte (start of the next heading
    /// or end of section), into `Registry::body`.
    pub span_end: usize,
}

/// A `## ` section of the registry body. Sections whose heading starts with
/// `Bucket:` are canonical buckets; other sections (e.g. "Promoted to TDE",
/// "Closed in this scan") still collect entries so counters see everything.
#[derive(Debug, Clone)]
#[allow(dead_code)] // span fields are part of the surgical-edit contract (Loom M0 lifts this struct)
pub struct Section {
    /// Bucket name (`ready`) for `## Bucket: <name>` headings; the full
    /// heading text otherwise.
    pub name: String,
    pub is_bucket: bool,
    /// Byte offset of the heading line start, into `Registry::body`.
    pub start: usize,
    /// Byte offset one past the section's last byte (start of the next `## `
    /// heading, or end of body).
    pub end: usize,
    pub entries: Vec<Entry>,
}

/// Typed view of the registry frontmatter. Every field is optional or
/// defaulted — lenient by design. Unknown fields survive untouched because
/// writes are surgical edits on the raw frontmatter text, never a
/// re-serialization of this struct. Fields the CLI does not read yet stay
/// in the typed mirror anyway — this struct is the contract Loom M0 lifts
/// into straymark-core.
#[derive(Debug, Clone, Default, serde::Deserialize)]
#[allow(dead_code)]
pub struct RegistryFrontmatter {
    #[serde(default)]
    pub schema_version: Option<String>,
    #[serde(default)]
    pub last_scan: Option<String>,
    #[serde(default)]
    pub last_scan_range: Option<String>,
    #[serde(default)]
    pub buckets: Vec<String>,
    #[serde(default)]
    pub fully_extracted_ailogs: Vec<String>,
    #[serde(default)]
    pub total_open: Option<u32>,
    #[serde(default)]
    pub total_promoted: Option<u32>,
    #[serde(default)]
    pub total_closed_in_session: Option<u32>,
    #[serde(default)]
    pub total_phase_blocked: Option<u32>,
    #[serde(default)]
    pub total_suspected_closed: Option<u32>,
}

/// A parsed registry. `frontmatter_raw` and `body` are kept verbatim so write
/// operations preserve everything the parser does not understand.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Registry {
    pub path: PathBuf,
    pub frontmatter: RegistryFrontmatter,
    pub frontmatter_raw: String,
    pub body: String,
    pub sections: Vec<Section>,
    /// Non-fatal parse warnings (malformed `### FU-` headings, etc.).
    pub warnings: Vec<String>,
}

impl Registry {
    /// All entries across all sections, in document order.
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.sections.iter().flat_map(|s| s.entries.iter())
    }

    pub fn is_v0(&self) -> bool {
        !matches!(self.frontmatter.schema_version.as_deref(), Some("v1"))
    }
}

/// Parse the registry from disk. Errors only on IO failure or a missing
/// frontmatter block (a registry without frontmatter has no
/// `fully_extracted_ailogs`, so drift detection cannot run).
pub fn parse_registry(path: &Path) -> Result<Registry> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read registry at {}", path.display()))?;
    parse_registry_str(path, &content)
}

/// Parse from raw content (split out for testability).
pub fn parse_registry_str(path: &Path, content: &str) -> Result<Registry> {
    let (fm_raw, body) = crate::utils::split_frontmatter(content).ok_or_else(|| {
        anyhow!(
            "Registry at {} has no YAML frontmatter (expected --- delimiters at top of file).\n  \
             hint: copy `.straymark/templates/follow-ups-backlog.md` to start a registry.",
            path.display()
        )
    })?;
    let frontmatter: RegistryFrontmatter = serde_yaml::from_str(fm_raw).unwrap_or_default();

    let mut warnings = Vec::new();
    let sections = parse_sections(body, &mut warnings);

    // Structural integrity (#253): every well-formed `### FU-NNN` heading in the
    // body should end up attached to a `## Bucket:` section. If the raw body
    // holds more well-formed headings than the parser could attach, some entries
    // are *invisible* to the counters — the silent-under-count failure mode where
    // the pulse reports N while the file actually has N+k open entries. The two
    // ways an entry goes invisible: its heading is glued to the previous line
    // (no blank line, so it is not a line-start `### `), or it sits before the
    // first `## ` section. Surface it loudly here so `recount`/`status`/`validate`
    // all see it instead of trusting a blind counter.
    let parsed_entries: usize = sections.iter().map(|s| s.entries.len()).sum();
    let wellformed_headings = count_wellformed_entry_headings(body);
    if wellformed_headings > parsed_entries {
        warnings.push(format!(
            "{} `### FU-NNN` heading(s) are not attached to any `## Bucket:` section and are \
             invisible to the counters (counters will under-count). Likely cause: a heading glued \
             to the previous line (missing blank line before `### `) or placed before the first \
             `## ` section. Put each `### FU-NNN` on its own line inside a `## Bucket:` section.",
            wellformed_headings - parsed_entries
        ));
    }

    Ok(Registry {
        path: path.to_path_buf(),
        frontmatter,
        frontmatter_raw: fm_raw.to_string(),
        body: body.to_string(),
        sections,
        warnings,
    })
}

/// Count well-formed `### FU-NNN — desc` entry headings present anywhere in the
/// raw body, including ones glued to a previous line (no leading newline) or
/// sitting outside any `## ` section — i.e. headings [`parse_sections`] cannot
/// see. The "heading text" runs from each `### FU-` occurrence to the end of its
/// line; a malformed heading (no number) is not counted (it is reported
/// separately by [`parse_entries`]), so this never double-counts. Used by
/// [`parse_registry_str`] as a structural integrity check (#253).
fn count_wellformed_entry_headings(body: &str) -> usize {
    let mut count = 0usize;
    let mut search_from = 0usize;
    while let Some(rel) = body[search_from..].find("### FU-") {
        let pos = search_from + rel;
        let line_end = body[pos..].find('\n').map(|n| pos + n).unwrap_or(body.len());
        if parse_entry_heading(&body[pos..line_end]).is_some() {
            count += 1;
        }
        search_from = pos + "### FU-".len();
    }
    count
}

/// Split the body into `## ` sections and parse `### FU-` entries in each.
fn parse_sections(body: &str, warnings: &mut Vec<String>) -> Vec<Section> {
    // Collect (offset, heading_text) for every `## ` line (exactly two #).
    let mut heads: Vec<(usize, String)> = Vec::new();
    let mut offset = 0usize;
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if let Some(rest) = trimmed.strip_prefix("## ") {
            if !trimmed.starts_with("### ") {
                heads.push((offset, rest.trim().to_string()));
            }
        }
        offset += line.len();
    }

    let mut sections = Vec::with_capacity(heads.len());
    for (i, (start, heading)) in heads.iter().enumerate() {
        let end = heads.get(i + 1).map(|(o, _)| *o).unwrap_or(body.len());
        let (name, is_bucket) = match heading.strip_prefix("Bucket:") {
            Some(rest) => {
                // Tolerate trailing annotations: `ready          (1 entry)`.
                let name = rest
                    .trim()
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string();
                (name, true)
            }
            None => (heading.clone(), false),
        };
        let entries = parse_entries(body, *start, end, &name, warnings);
        sections.push(Section {
            name,
            is_bucket,
            start: *start,
            end,
            entries,
        });
    }
    sections
}

/// Parse `### FU-NNN — desc` blocks between `start` and `end` of `body`.
fn parse_entries(
    body: &str,
    start: usize,
    end: usize,
    bucket: &str,
    warnings: &mut Vec<String>,
) -> Vec<Entry> {
    let section = &body[start..end];
    // Locate entry heading offsets (relative to section start).
    let mut heads: Vec<(usize, String)> = Vec::new();
    let mut offset = 0usize;
    for line in section.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed.starts_with("### ") {
            heads.push((offset, trimmed.to_string()));
        }
        offset += line.len();
    }

    let mut entries = Vec::new();
    for (i, (rel_start, heading)) in heads.iter().enumerate() {
        let rel_end = heads.get(i + 1).map(|(o, _)| *o).unwrap_or(section.len());
        let abs_start = start + rel_start;
        let abs_end = start + rel_end;

        let Some((fu_id, fu_number, description)) = parse_entry_heading(heading) else {
            // A `###` heading that isn't an FU entry (e.g. a dated triage
            // subsection under "Closed in this scan") is not a warning —
            // only malformed `### FU-` headings are.
            if heading.starts_with("### FU-") {
                warnings.push(format!(
                    "Malformed entry heading (expected `### FU-NNN — description`): {}",
                    heading
                ));
            }
            continue;
        };

        let block = &section[*rel_start..rel_end];
        let mut entry = Entry {
            fu_id,
            fu_number,
            description,
            bucket: bucket.to_string(),
            origin: None,
            origin_class: None,
            source_hash: None,
            status: FuStatus::Unknown,
            status_raw: None,
            severity: None,
            trigger: None,
            destination: None,
            cost: None,
            labels: Vec::new(),
            notes: None,
            promoted_to: None,
            premise: None,
            verified_at: None,
            span_start: abs_start,
            span_end: abs_end,
        };

        for line in block.lines() {
            let Some((field, value)) = parse_field_line(line) else {
                continue;
            };
            let value = value.trim();
            match field.to_lowercase().as_str() {
                "origin" => entry.origin = some_nonempty(value),
                "origin-class" | "origin class" | "origin_class" => {
                    entry.origin_class = some_nonempty(value)
                }
                "source-hash" | "source hash" | "source_hash" => {
                    entry.source_hash = some_nonempty(value)
                }
                "status" => {
                    entry.status_raw = some_nonempty(value);
                    entry.status = FuStatus::from_str_loose(value);
                }
                "severity" => entry.severity = Severity::from_str_loose(value),
                "trigger" => entry.trigger = some_nonempty(value),
                "destination" => entry.destination = some_nonempty(value),
                "cost" => entry.cost = some_nonempty(value),
                "labels" | "tags" => {
                    entry.labels = value
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                }
                "notes" => entry.notes = some_nonempty(value),
                "promoted to" | "promoted-to" | "promoted_to" => {
                    entry.promoted_to = some_nonempty(value)
                }
                "premise" => entry.premise = some_nonempty(value),
                "verified-at" | "verified at" | "verified_at" => {
                    entry.verified_at = some_nonempty(value)
                }
                _ => {} // unknown field — lenient, preserved in the raw body
            }
        }
        entries.push(entry);
    }
    entries
}

fn some_nonempty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// Parse `### FU-NNN — desc` (em dash, hyphen, or colon separator).
/// Returns (fu_id, number, description) or None when the heading isn't an
/// FU entry.
fn parse_entry_heading(heading: &str) -> Option<(String, u32, String)> {
    let rest = heading.strip_prefix("### ")?.trim();
    let after_fu = rest.strip_prefix("FU-")?;
    let digits: String = after_fu.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let number: u32 = digits.parse().ok()?;
    let fu_id = format!("FU-{}", digits);
    let after_digits = &after_fu[digits.len()..];
    let description = after_digits
        .trim_start_matches([' ', '\t'])
        .trim_start_matches(['—', '–', '-', ':'])
        .trim()
        .to_string();
    Some((fu_id, number, description))
}

/// Parse a `- **Field**: value` bullet line. Returns (field, value).
fn parse_field_line(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("- **")?;
    let close = rest.find("**")?;
    let field = &rest[..close];
    let after = rest[close + 2..].trim_start();
    let value = after.strip_prefix(':')?;
    Some((field, value))
}

/// Find an entry by user-supplied id: `FU-085`, `085`, or `85`.
pub fn find_entry<'a>(registry: &'a Registry, id_input: &str) -> Option<&'a Entry> {
    let trimmed = id_input.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(e) = registry.entries().find(|e| e.fu_id == trimmed) {
        return Some(e);
    }
    let digits = trimmed.strip_prefix("FU-").unwrap_or(trimmed);
    if let Ok(n) = digits.parse::<u32>() {
        return registry.entries().find(|e| e.fu_number == n);
    }
    None
}

/// Every entry an id resolves to. Normally one; more than one means the
/// registry carries duplicate ids and any write addressed by that id is a
/// coin flip.
pub fn find_entries<'a>(registry: &'a Registry, id_input: &str) -> Vec<&'a Entry> {
    let trimmed = id_input.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let exact: Vec<&Entry> = registry.entries().filter(|e| e.fu_id == trimmed).collect();
    if !exact.is_empty() {
        return exact;
    }
    let digits = trimmed.strip_prefix("FU-").unwrap_or(trimmed);
    match digits.parse::<u32>() {
        Ok(n) => registry.entries().filter(|e| e.fu_number == n).collect(),
        Err(_) => Vec::new(),
    }
}

/// Resolve an id to exactly one entry, or fail.
///
/// GH #415: every mutating command resolved by taking the **first** match.
/// With duplicate ids in the registry — which parallel branches and triage
/// pruning both produce — `note` wrote into the wrong entry and `set-status`
/// answered `already closed — nothing to change`, which reads as success. The
/// operator's actual follow-up stayed open and unannotated, and nothing said so.
///
/// Refusing is the only safe answer: the command cannot know which entry was
/// meant, and guessing is what caused the silent write.
pub fn find_entry_unique<'a>(registry: &'a Registry, id_input: &str) -> Result<&'a Entry> {
    let matches = find_entries(registry, id_input);
    match matches.len() {
        1 => Ok(matches[0]),
        0 => bail!(
            "{} not found in {}",
            id_input,
            registry.path.display()
        ),
        n => {
            let mut detail = String::new();
            for e in &matches {
                detail.push_str(&format!(
                    "\n    - {} [{}] {} — {}",
                    e.fu_id,
                    e.status.as_str(),
                    e.origin.as_deref().unwrap_or("origin unknown"),
                    crate::utils::truncate_visual(&e.description, 60)
                ));
            }
            bail!(
                "{id_input} is ambiguous: {n} entries share that id in {}.{detail}\n  \
                 hint: ids are positional, so a merge of parallel branches — or triage pruning a \
                 closed entry's heading — can hand the same number to two follow-ups. Renumber \
                 all but one to a free id (`straymark followups status` shows the highest in \
                 use), then re-run. Addressing a duplicate id would write into whichever entry \
                 happens to come first in the file.",
                registry.path.display()
            )
        }
    }
}

/// Next sequential FU number: one past the highest number the registry has
/// *ever* used, not one past the highest entry still standing.
///
/// GH #415: the high-water mark used to come from parsed entries alone, so an
/// id stopped being reserved the moment its entry lost its `### FU-NNN`
/// heading — which is exactly what triage does when it prunes a closed entry
/// down to a provenance bullet. The next `drift --apply` then handed that
/// number to an unrelated follow-up, and the registry ended up with two
/// different items answering to one id. Reported from Sentinel on 2026-06-04
/// (`FU-123` reused after supersession) and again in #415.
///
/// So the scan is over the raw body, not the entry list: any `FU-NNN` mention
/// anywhere — a pruned bullet, a `Notes` back-reference, a superseded entry —
/// keeps its number retired. Numbers are cheap; a collision is not.
pub fn next_fu_number(registry: &Registry) -> u32 {
    let from_entries = registry.entries().map(|e| e.fu_number).max().unwrap_or(0);
    let from_body = highest_fu_mention(&registry.body);
    from_entries.max(from_body) + 1
}

/// Highest `FU-NNN` number mentioned anywhere in the registry body, including
/// ids that no longer have an entry of their own. Author-scoped ids like
/// `FU-058-022` contribute their first group only — the registry-assigned part.
fn highest_fu_mention(body: &str) -> u32 {
    let mut highest = 0u32;
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while let Some(pos) = body[i..].find("FU-") {
        let start = i + pos + 3;
        let mut end = start;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end > start {
            if let Ok(n) = body[start..end].parse::<u32>() {
                highest = highest.max(n);
            }
        }
        i = start.max(i + pos + 1);
    }
    highest
}

/// Counters recomputed from actual entry statuses — the CLI-owned source of
/// truth since schema v1 (issue #214 Signal 2). Semantics:
/// - `open` / `in_progress` / `suspected_closed` / `promoted` — entries with
///   that exact `Status`.
/// - `closed_cumulative` — `closed` + `superseded` (the registry keeps full
///   history; "in session" granularity is not derivable from the file).
/// - `phase_blocked_open` — `open` entries living in the `phase-blocked` bucket.
/// - `blocking_open` — `open` or `in-progress` entries with `Severity: blocking`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Counters {
    pub open: u32,
    pub in_progress: u32,
    pub suspected_closed: u32,
    pub closed_cumulative: u32,
    pub promoted: u32,
    pub phase_blocked_open: u32,
    pub blocking_open: u32,
    pub total: u32,
}

pub fn compute_counters(registry: &Registry) -> Counters {
    let mut c = Counters::default();
    for e in registry.entries() {
        c.total += 1;
        match e.status {
            FuStatus::Open => c.open += 1,
            FuStatus::InProgress => c.in_progress += 1,
            FuStatus::SuspectedClosed => c.suspected_closed += 1,
            FuStatus::Closed | FuStatus::Superseded => c.closed_cumulative += 1,
            FuStatus::Promoted => c.promoted += 1,
            FuStatus::Unknown => {}
        }
        if e.status == FuStatus::Open && e.bucket == "phase-blocked" {
            c.phase_blocked_open += 1;
        }
        if matches!(e.status, FuStatus::Open | FuStatus::InProgress)
            && e.severity == Some(Severity::Blocking)
        {
            c.blocking_open += 1;
        }
    }
    c
}

// ───────────────────────── surgical frontmatter edits ─────────────────────────
//
// Writes never re-serialize `RegistryFrontmatter` — that would drop unknown
// fields and reorder keys. Instead these helpers edit the raw frontmatter
// text line by line, preserving everything they don't touch.

/// Replace the value of a top-level scalar `key: value` line, or append the
/// line at the end of the frontmatter when the key is absent.
pub fn fm_set_scalar(fm: &str, key: &str, value: &str) -> String {
    let prefix = format!("{}:", key);
    let mut out: Vec<String> = Vec::new();
    let mut replaced = false;
    for line in fm.lines() {
        if !replaced && line.starts_with(&prefix) {
            out.push(format!("{} {}", prefix, value));
            replaced = true;
        } else {
            out.push(line.to_string());
        }
    }
    if !replaced {
        out.push(format!("{} {}", prefix, value));
    }
    out.join("\n")
}

/// Append items to a top-level YAML block list `key:`. Handles the
/// `key: []` empty-flow form by converting it to a block list. Item
/// indentation mirrors the existing items (default two spaces).
pub fn fm_append_list_items(fm: &str, key: &str, items: &[String]) -> String {
    if items.is_empty() {
        return fm.to_string();
    }
    let prefix = format!("{}:", key);
    let lines: Vec<&str> = fm.lines().collect();
    let Some(key_idx) = lines.iter().position(|l| l.starts_with(&prefix)) else {
        // Key absent — append key + items at the end.
        let mut out = fm.to_string();
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&prefix);
        out.push('\n');
        for item in items {
            out.push_str(&format!("  - {}\n", item));
        }
        return out.trim_end_matches('\n').to_string();
    };

    let mut out: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    let key_line = lines[key_idx];
    let after_colon = key_line[prefix.len()..].trim();

    if after_colon == "[]" {
        // Convert empty flow list to a block list.
        out[key_idx] = prefix.clone();
        let mut insert_at = key_idx + 1;
        for item in items {
            out.insert(insert_at, format!("  - {}", item));
            insert_at += 1;
        }
        return out.join("\n");
    }

    // Find the end of the block list: last consecutive line after key_idx
    // that is an indented `- ` item (or a comment inside the block).
    let mut indent = "  ".to_string();
    let mut last_item = key_idx;
    for (i, line) in lines.iter().enumerate().skip(key_idx + 1) {
        let t = line.trim_start();
        if t.starts_with("- ") && line.starts_with(' ') {
            indent = line[..line.len() - t.len()].to_string();
            last_item = i;
        } else if t.starts_with('#') && last_item > key_idx {
            // comment inside the list — keep scanning
            continue;
        } else {
            break;
        }
    }
    let mut insert_at = last_item + 1;
    for item in items {
        out.insert(insert_at, format!("{}- {}", indent, item));
        insert_at += 1;
    }
    out.join("\n")
}

/// Apply the recomputed counters + the v1 marker to a frontmatter text.
/// Sets `schema_version: v1` and every `total_*` counter (adding missing
/// counter lines). This is the entire v0 → v1 upgrade — idempotent and
/// non-destructive (all v1 entry fields are optional).
pub fn fm_apply_counters_and_v1(fm: &str, counters: &Counters) -> String {
    let mut out = fm_set_scalar(fm, "schema_version", "v1");
    out = fm_set_scalar(&out, "total_open", &counters.open.to_string());
    out = fm_set_scalar(&out, "total_promoted", &counters.promoted.to_string());
    out = fm_set_scalar(
        &out,
        "total_closed_in_session",
        &counters.closed_cumulative.to_string(),
    );
    out = fm_set_scalar(
        &out,
        "total_phase_blocked",
        &counters.phase_blocked_open.to_string(),
    );
    out = fm_set_scalar(
        &out,
        "total_suspected_closed",
        &counters.suspected_closed.to_string(),
    );
    out
}

/// Reassemble a registry file from (frontmatter, body). Line endings are
/// normalized to `\n` (split_frontmatter accepts CRLF input).
pub fn assemble(fm: &str, body: &str) -> String {
    format!("---\n{}\n---\n{}", fm.trim_end_matches('\n'), body)
}

// ───────────────────────── AILOG extraction (drift) ─────────────────────────

/// A follow-up bullet extracted from an AILOG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedFu {
    /// The display title for the registry heading — a high-fidelity summary of
    /// the bullet (un-wrapped lead, markup stripped, first sentence, capped).
    /// See [`entry_title`]. Decoupled from `description` so improving titles
    /// never perturbs the dedup hash (#365).
    pub title: String,
    /// The bullet's raw first line, cleaned of list markers. Kept **stable** as
    /// the seed of the dedup `Source-hash` (`fu_content_hash`) — it must not
    /// change when `title` extraction improves, or every already-extracted
    /// entry in an adopter's registry would re-flag as a duplicate (#365).
    pub description: String,
    /// Origin pointer suffix, e.g. "§Follow-ups" or "§R3 (new, not in Charter)".
    pub origin_section: String,
    /// True when the bullet text carries an in-Charter closure marker —
    /// extracted as `suspected-closed` instead of `open` (#214 Signal 1).
    pub suspected_closed: bool,
}

/// The `## Follow-ups` section heading in the shipped locales (#263). A
/// Spanish/Chinese-first adopter translates the heading; keying on the English
/// literal silently skips the whole section, so `drift` extracts nothing and
/// those follow-ups never reach the backlog. ASCII variants match
/// case-insensitively; the es/zh forms have no case folding.
const FOLLOWUP_HEADINGS: &[&str] = &["Follow-ups", "Follow-Ups", "Seguimientos", "后续工作", "后续"];

/// Extract follow-up content from an AILOG body: top-level bullets of every
/// `## Follow-ups*` section plus any *structural* `R<N> (new, not in Charter)`
/// risk declaration.
///
/// #346 (adopter field report) hardened three heuristic failures:
/// - **Under-capture**: an explicit `## Follow-ups (auditoría externa)` heading
///   was skipped because the old matcher required an exact heading equality. We
///   now match any heading that *starts with* a follow-ups token, and collect
///   **all** such sections (a plain `## Follow-ups` and an audit-scoped one can
///   coexist).
/// - **Over-capture**: a prose summary line that merely *mentioned* the phrase
///   ("Riesgos R1–R5 mitigados… Emergió R6 (new, not in Charter)…") was
///   extracted as a follow-up. We now require the line to be a *structural*
///   risk declaration (heading or list item that begins with the `R<N>` token).
/// - **Resolved-as-open**: a `## Risk: R<N>` heading whose remediation is
///   documented in the section body ("Corregido a…", a `Mitigaciones aplicadas`
///   sub-block, an AIDEC reference) was extracted as `open`. Closure is now
///   judged over the whole risk section, so it lands as `suspected-closed`.
pub fn extract_followups_from_ailog(content: &str) -> Vec<ExtractedFu> {
    let mut out = Vec::new();
    let sections = split_hash_sections(content);

    // ── `## Follow-ups*` sections (locale-aware, prefix-tolerant #346) ──
    for (heading, body) in &sections {
        let Some(h) = heading else { continue };
        if !is_followup_heading(h) {
            continue;
        }
        // Group top-level bullets with their continuation lines so closure
        // markers on continuation lines are seen.
        let mut current: Option<String> = None;
        for line in body.lines() {
            if let Some(rest) = line.strip_prefix("- ") {
                if let Some(buf) = current.take() {
                    push_extracted(&mut out, &buf, "§Follow-ups");
                }
                current = Some(rest.to_string());
            } else if let Some(buf) = current.as_mut() {
                if line.starts_with("  ") || line.trim().is_empty() {
                    buf.push('\n');
                    buf.push_str(line.trim_start());
                } else {
                    let done = current.take().unwrap();
                    push_extracted(&mut out, &done, "§Follow-ups");
                }
            }
        }
        if let Some(buf) = current.take() {
            push_extracted(&mut out, &buf, "§Follow-ups");
        }
    }

    // ── `R<N> (new, not in Charter)` structural risk declarations (#346) ──
    for (heading, body) in &sections {
        if let Some(rn) = heading.as_deref().and_then(risk_declaration_token) {
            // Heading-style risk: the whole section is one risk, and its
            // remediation (if any) lives in the body — judge closure over both.
            let heading_text = heading.as_deref().unwrap_or_default();
            let combined = format!("{heading_text}\n{body}");
            let desc = clean_risk_desc(heading_text);
            out.push(ExtractedFu {
                title: desc.clone(),
                description: desc,
                origin_section: format!("§{} (new, not in Charter)", rn),
                suspected_closed: risk_section_resolved(&combined),
            });
        } else {
            // Scan the section body for inline (bullet-style) risk declarations.
            extract_inline_risks(body, &mut out);
        }
    }

    out
}

/// Scan a section body for inline, *structural* `R<N> (new, not in Charter)`
/// risk declarations (list items). Prose lines that merely mention the phrase
/// are skipped by [`risk_declaration_token`]. Each declaration is grouped with
/// its indented continuation lines so a closure marker documented on a
/// continuation is seen.
fn extract_inline_risks(body: &str, out: &mut Vec<ExtractedFu>) {
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let Some(rn) = risk_declaration_token(lines[i]) else {
            i += 1;
            continue;
        };
        let mut group = lines[i].to_string();
        let mut j = i + 1;
        while j < lines.len() {
            let l = lines[j];
            if l.starts_with("  ") || l.trim().is_empty() {
                group.push('\n');
                group.push_str(l);
                j += 1;
            } else {
                break;
            }
        }
        let desc = clean_risk_desc(lines[i]);
        out.push(ExtractedFu {
            title: desc.clone(),
            description: desc,
            origin_section: format!("§{} (new, not in Charter)", rn),
            suspected_closed: risk_section_resolved(&group),
        });
        i = j;
    }
}

fn push_extracted(out: &mut Vec<ExtractedFu>, bullet: &str, origin: &str) {
    // `description` is the hash seed and MUST stay the raw first line (#365) —
    // the dedup `Source-hash` derives from it, so it cannot change with title
    // extraction or every prior extraction re-flags as a duplicate.
    let desc = first_line(bullet);
    if desc.is_empty() {
        return;
    }
    out.push(ExtractedFu {
        title: entry_title(bullet),
        description: desc,
        origin_section: origin.to_string(),
        suspected_closed: has_closure_marker(bullet),
    });
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().to_string()
}

/// Derive a high-fidelity display title for an extracted `§Follow-ups` bullet.
///
/// The bullet arrives hard-wrapped: [`extract_followups_from_ailog`] joins a
/// top-level bullet with its continuation lines using `\n`. Taking the first
/// *physical* line (the pre-#365 behaviour) truncated the title at the author's
/// soft-wrap column — mid-sentence, sometimes mid-word — so entries landed with
/// headings like `**Footgun de pack local…**: el pack lee de` cut off at the
/// wrap. Reported by the Weft adopter (StrayMark #365).
///
/// Strategy, in order:
/// 1. **Un-wrap** the lead paragraph (join lines up to the first blank line)
///    into one logical line.
/// 2. If it opens with a substantial `**bold**` span, use that span's content —
///    the observed authoring convention leads a bullet with `**short title**`
///    then prose. This is the highest-fidelity title when present.
/// 3. Otherwise strip inline emphasis (backticks kept) and take the first
///    sentence.
/// 4. Cap at a word boundary with an ellipsis so a long lead never runs away.
///
/// The dedup hash is computed from `description` (the raw first line), never
/// from this title, so sharpening titles is hash-neutral for existing entries.
fn entry_title(bullet: &str) -> String {
    const MAX_CHARS: usize = 120;
    let para = unwrap_lead_paragraph(bullet);
    if let Some(bold) = leading_bold_title(&para) {
        return cap_title(&bold, MAX_CHARS);
    }
    let cleaned = crate::commands::charter::new::strip_inline_markup(&para);
    let sentence = crate::commands::charter::new::leading_sentences(&cleaned, 1);
    cap_title(sentence.trim(), MAX_CHARS)
}

/// Join a bullet's hard-wrapped lines into one logical line, stopping at the
/// first blank line (paragraph break). List markers are already stripped by the
/// caller's grouping.
fn unwrap_lead_paragraph(bullet: &str) -> String {
    let mut para = String::new();
    for line in bullet.lines() {
        let t = line.trim();
        if t.is_empty() {
            if !para.is_empty() {
                break;
            }
            continue;
        }
        if !para.is_empty() {
            para.push(' ');
        }
        para.push_str(t);
    }
    para
}

/// If `para` opens with a `**…**` bold span whose content is substantial
/// (≥ 15 chars, so a bare `**R6**` tag falls through to first-sentence), return
/// that content with inline emphasis stripped (backticks kept). This matches the
/// convention of leading a follow-up bullet with a bolded title (#365).
fn leading_bold_title(para: &str) -> Option<String> {
    let rest = para.strip_prefix("**")?;
    let end = rest.find("**")?;
    let inner = crate::commands::charter::new::strip_inline_markup(rest[..end].trim());
    let inner = inner.trim();
    if inner.chars().count() >= 15 {
        Some(inner.to_string())
    } else {
        None
    }
}

/// Trim to `max` chars at a word boundary, appending an ellipsis when cut.
fn cap_title(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        return s.to_string();
    }
    let truncated: String = s.chars().take(max).collect();
    let cut = truncated.rfind(char::is_whitespace).unwrap_or(truncated.len());
    format!("{}…", truncated[..cut].trim_end())
}

/// Split an AILOG body into `## ` sections. Returns `(heading, body)` pairs
/// where `heading` is the text after `## ` (`None` for the preamble before the
/// first `## `) and `body` is every line up to the next `## ` heading. `###`+
/// sub-headings stay inside the body (only `## ` starts a new section).
fn split_hash_sections(content: &str) -> Vec<(Option<String>, String)> {
    let mut sections: Vec<(Option<String>, String)> = Vec::new();
    let mut heading: Option<String> = None;
    let mut body = String::new();
    for line in content.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            sections.push((heading.take(), std::mem::take(&mut body)));
            heading = Some(h.trim().to_string());
            continue;
        }
        body.push_str(line);
        body.push('\n');
    }
    sections.push((heading, body));
    sections
}

/// True when a `## ` heading is a follow-ups section in any shipped locale.
/// Matches an exact heading (case-insensitive for the ASCII forms) **or** a
/// heading that starts with a follow-ups token followed by a non-alphanumeric
/// boundary — so `## Follow-ups (auditoría externa)` and `## Seguimientos:
/// deuda` are recognized, not just the bare heading (#346 under-capture).
pub fn is_followup_heading(heading: &str) -> bool {
    let h = heading.trim();
    let hl = h.to_lowercase();
    FOLLOWUP_HEADINGS.iter().any(|x| {
        let xl = x.to_lowercase();
        match hl.strip_prefix(&xl) {
            Some("") => true,
            Some(rest) => rest.chars().next().is_some_and(|c| !c.is_alphanumeric()),
            None => false,
        }
    })
}

/// If `line` is a *structural* risk declaration (heading or list item) for an
/// emergent `R<N> (new, not in Charter)` risk, return the `R<N>` token. Prose
/// lines that merely mention the phrase (a summary paragraph) return `None` —
/// they begin with narrative text, not the `R<N>` token (#346 over-capture).
fn risk_declaration_token(line: &str) -> Option<String> {
    if !line.contains("(new, not in Charter)") {
        return None;
    }
    // Strip leading structural markers: heading hashes, list bullets, blockquote.
    let mut s = line
        .trim()
        .trim_start_matches(['#', '>', '-', '*', ' '])
        .trim_start_matches("**")
        .trim_start();
    // Optional "Risk:" / "Riesgo:" label before the token.
    for label in ["Risk:", "Riesgo:", "risk:", "riesgo:"] {
        if let Some(rest) = s.strip_prefix(label) {
            s = rest.trim_start();
            break;
        }
    }
    s = s.trim_start_matches("**").trim_start();
    let token: String = s.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
    match token.strip_prefix('R') {
        Some(rest) if !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()) => Some(token),
        _ => None,
    }
}

/// Clean a risk declaration line into a description: strip leading heading
/// hashes / list markers and bold, keep the first line. Bullet-style lines
/// clean identically to the pre-#346 behavior (hash-stable dedup key); only
/// heading-style declarations gain the leading `## ` strip.
fn clean_risk_desc(line: &str) -> String {
    let s = line.trim_start().trim_start_matches('#').trim_start();
    let cleaned = s.trim_start_matches("- ").replace("**", "");
    first_line(cleaned.trim())
}

/// Closure verbs recognized by the born-resolved idiom family
/// "<verb> … in this PR / in this commit" (#222 Finding 2).
const CLOSURE_VERBS: [&str; 6] = [
    "updated",
    "corrected",
    "remediated",
    "resolved",
    "fixed",
    "closed",
];

/// True when the text carries an explicit in-Charter closure marker:
/// "closed in-Charter" / "closed in Charter" / "resolved in-Charter" /
/// "fixed in batch N", a born-resolved idiom — a closure verb (`updated` /
/// `corrected` / `remediated` / `resolved` / `fixed` / `closed`) followed by
/// "in this PR" or "in this commit" (#222 Finding 2, first external adopter)
/// — all case-insensitive, or a backtick-wrapped commit hash (7-40 hex
/// chars). The signal that drives `suspected-closed` extraction (#214
/// Signal 1 — 20-75% of auto-appended entries per batch were already
/// resolved in-Charter across both documented occurrences).
pub fn has_closure_marker(text: &str) -> bool {
    let lower = text.to_lowercase();
    if lower.contains("closed in-charter")
        || lower.contains("closed in charter")
        || lower.contains("resolved in-charter")
        || lower.contains("resolved in charter")
    {
        return true;
    }
    // "fixed in batch <N>"
    if let Some(idx) = lower.find("fixed in batch ") {
        let rest = &lower[idx + "fixed in batch ".len()..];
        if rest.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return true;
        }
    }
    // Born-resolved idiom: a closure verb anywhere before "in this PR" /
    // "in this commit" (e.g. "Charter row updated atomically in this PR").
    // rfind takes the last context occurrence, maximizing the verb window.
    for ctx in ["in this pr", "in this commit"] {
        if let Some(ctx_idx) = lower.rfind(ctx) {
            let before = &lower[..ctx_idx];
            if CLOSURE_VERBS.iter().any(|v| before.contains(v)) {
                return true;
            }
        }
    }
    // Backtick-wrapped hex run of 7-40 chars (commit hash reference).
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '`' {
            let rest = &text[i + 1..];
            if let Some(close) = rest.find('`') {
                let inner = &rest[..close];
                let len = inner.chars().count();
                if (7..=40).contains(&len)
                    && inner.chars().all(|c| c.is_ascii_hexdigit())
                    && inner.chars().any(|c| c.is_ascii_digit())
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Extra closure signals for an emergent-risk *section* — a heading-style
/// `## Risk: R<N>` whose remediation is documented inline in the section body
/// rather than on the heading line (#346). Broader than [`has_closure_marker`]:
/// Spanish/English remediation participles, a `Mitigaciones aplicadas` /
/// `Mitigations applied` sub-block, and an AIDEC cross-reference all mark the
/// risk as resolved-in-Charter, so it lands as `suspected-closed` instead of
/// polluting the open count.
fn risk_section_resolved(text: &str) -> bool {
    if has_closure_marker(text) {
        return true;
    }
    let lower = text.to_lowercase();
    // Remediation participles (ES + EN) as standalone words.
    const RESOLUTION_WORDS: &[&str] = &[
        "corregido",
        "corregida",
        "mitigado",
        "mitigada",
        "resuelto",
        "resuelta",
        "solucionado",
        "solucionada",
        "remediado",
        "remediada",
    ];
    if RESOLUTION_WORDS.iter().any(|w| contains_word(&lower, w)) {
        return true;
    }
    // A dedicated remediation sub-block, or a decided-remediation AIDEC ref.
    lower.contains("mitigaciones aplicadas")
        || lower.contains("mitigations applied")
        || lower.contains("aidec-")
}

/// True when `word` appears in `haystack` bounded by non-alphanumeric edges —
/// so "resuelto" does not fire inside "irresuelto". `haystack` is assumed
/// already lowercased.
fn contains_word(haystack: &str, word: &str) -> bool {
    let bytes = haystack.as_bytes();
    let wlen = word.len();
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(word) {
        let i = start + pos;
        let before_ok = i == 0
            || !bytes[i - 1].is_ascii_alphanumeric();
        let after = i + wlen;
        let after_ok = after >= bytes.len() || !bytes[after].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
        start = i + wlen;
    }
    false
}

/// Derive the AILOG id from a filename: first five dash-separated tokens
/// (`AILOG-2026-06-03-003-slug.md` → `AILOG-2026-06-03-003`).
pub fn ailog_id_from_path(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    if !stem.starts_with("AILOG-") {
        return None;
    }
    let id: String = stem.split('-').take(5).collect::<Vec<_>>().join("-");
    Some(id)
}

/// Stable content hash identifying a follow-up by its source — the dedup key
/// that lets `drift` re-scan an already-extracted AILOG without re-adding
/// entries (#231). Derived from the source AILOG id, the origin section, and
/// the follow-up description. SHA-256 truncated to 12 hex chars: stable across
/// platforms/runs (unlike `std`'s `DefaultHasher`) and collision-safe enough
/// for a per-project registry. The separator is the ASCII unit separator so
/// it can't appear in any component.
pub fn fu_content_hash(ailog_id: &str, origin_section: &str, description: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ailog_id.as_bytes());
    hasher.update([0x1f]);
    hasher.update(origin_section.as_bytes());
    hasher.update([0x1f]);
    hasher.update(description.as_bytes());
    format!("{:x}", hasher.finalize())[..12].to_string()
}

/// Split a rendered `Origin` value (`<AILOG-id> <§section...>`) into its
/// `(ailog_id, origin_section)` parts at the first whitespace run. Returns
/// `None` when there is no section suffix.
pub fn split_origin(origin: &str) -> Option<(&str, &str)> {
    let trimmed = origin.trim();
    let idx = trimmed.find(char::is_whitespace)?;
    let ailog_id = &trimmed[..idx];
    let section = trimmed[idx..].trim_start();
    if ailog_id.is_empty() || section.is_empty() {
        None
    } else {
        Some((ailog_id, section))
    }
}

/// The set of follow-up content hashes already present in the registry — the
/// dedup key set for drift re-scans (#231). Prefers each entry's stored
/// `source_hash`; for legacy entries that predate the field, recomputes the
/// hash from `origin` + `description` (best-effort — vulnerable only to a
/// triage rewording of the heading, which the stored hash is immune to).
pub fn registry_extracted_hashes(registry: &Registry) -> HashSet<String> {
    let mut set = HashSet::new();
    for entry in registry.entries() {
        if let Some(h) = &entry.source_hash {
            set.insert(h.clone());
        } else if let Some(origin) = &entry.origin {
            if let Some((ailog_id, section)) = split_origin(origin) {
                set.insert(fu_content_hash(ailog_id, section, &entry.description));
            }
        }
    }
    set
}

/// Render a new entry block for `drift --apply`.
pub fn render_new_entry(
    fu_number: u32,
    extracted: &ExtractedFu,
    ailog_id: &str,
    today: &str,
) -> String {
    let status = if extracted.suspected_closed {
        "suspected-closed"
    } else {
        "open"
    };
    let mut notes = format!("Auto-appended by `straymark followups drift --apply` {}.", today);
    if extracted.suspected_closed {
        notes.push_str(" Closure marker detected in the source AILOG — confirm and mark `closed`, or reopen.");
    }
    let source_hash = fu_content_hash(ailog_id, &extracted.origin_section, &extracted.description);
    format!(
        "### FU-{:03} — {}\n\
         - **Origin**: {} {}\n\
         - **Source-hash**: {}\n\
         - **Status**: {}\n\
         - **Trigger**: TBD\n\
         - **Destination**: TBD\n\
         - **Cost**: TBD\n\
         - **Notes**: {}\n",
        fu_number, extracted.title, ailog_id, extracted.origin_section, source_hash, status, notes
    )
}

/// Insert `block` at the end of the `## Bucket: <bucket>` section of `body`.
/// Creates the bucket section at the end of the body when it is absent.
pub fn insert_into_bucket(registry: &Registry, bucket: &str, block: &str) -> String {
    let body = &registry.body;
    let target = registry
        .sections
        .iter()
        .find(|s| s.is_bucket && s.name == bucket);

    match target {
        Some(section) => {
            // Insert before the next section heading, normalizing the blank
            // lines on both sides: one before the new entry, and one after it
            // when a following section exists — otherwise the appended entry
            // butts straight against the next `## Bucket:` heading.
            let head = &body[..section.end];
            let tail = &body[section.end..];
            let head_trimmed = head.trim_end_matches('\n');
            let block_trimmed = block.trim_end_matches('\n');
            if tail.is_empty() {
                format!("{}\n\n{}\n", head_trimmed, block_trimmed)
            } else {
                format!("{}\n\n{}\n\n{}", head_trimmed, block_trimmed, tail)
            }
        }
        None => {
            let trimmed = body.trim_end_matches('\n');
            format!(
                "{}\n\n## Bucket: {}\n\n{}\n",
                trimmed,
                bucket,
                block.trim_end_matches('\n')
            )
        }
    }
}

/// Surgically update one field bullet inside an entry's span. Replaces the
/// existing `- **<field>**: ...` line or appends a new bullet at the end of
/// the entry's bullet list. Returns the updated body.
pub fn set_entry_field(body: &str, entry: &Entry, field: &str, value: &str) -> String {
    let block = &body[entry.span_start..entry.span_end];
    let mut lines: Vec<String> = block.lines().map(|s| s.to_string()).collect();
    let needle = format!("- **{}**", field);
    let mut replaced = false;
    for line in lines.iter_mut() {
        if line.trim_start().starts_with(&needle) {
            *line = format!("- **{}**: {}", field, value);
            replaced = true;
            break;
        }
    }
    if !replaced {
        // Insert after the last `- **` bullet line.
        let last_bullet = lines
            .iter()
            .rposition(|l| l.trim_start().starts_with("- **"));
        let insert_at = last_bullet.map(|i| i + 1).unwrap_or(lines.len());
        lines.insert(insert_at, format!("- **{}**: {}", field, value));
    }
    let mut new_block = lines.join("\n");
    // Preserve the block's trailing newline shape *exactly*. Collapsing
    // "\n\n" to "\n" (the earlier behavior) ate the blank line separating the
    // edited entry from the next heading, so every field write nudged the
    // registry's markdown shape — visible once `note` made field writes routine.
    let trailing = block.len() - block.trim_end_matches('\n').len();
    while new_block.len() - new_block.trim_end_matches('\n').len() < trailing {
        new_block.push('\n');
    }
    let mut out = String::with_capacity(body.len() + 32);
    out.push_str(&body[..entry.span_start]);
    out.push_str(&new_block);
    out.push_str(&body[entry.span_end..]);
    out
}

/// The five canonical buckets (schema v1 `buckets` enum). Stable at N=91
/// entries in the reference adopter — no sixth has been needed.
pub const CANONICAL_BUCKETS: &[&str] = &[
    "ready",
    "time-triggered",
    "charter-triggered",
    "phase-blocked",
    "operational",
];

/// Recompute the CLI-owned counters from `body` and return the frontmatter
/// carrying them (plus the v0 → v1 upgrade every write command performs).
///
/// Counters are derived from the entries *as they will be written*, not from
/// the registry as it was read — which is what makes "the entry edit and the
/// counter update are one step" true rather than a convention the operator has
/// to remember (#355: the hand-edit + `recount` two-step is desyncable).
pub fn recounted_frontmatter(
    path: &Path,
    frontmatter_raw: &str,
    body: &str,
) -> Result<(String, Counters)> {
    let reparsed = parse_registry_str(path, &assemble(frontmatter_raw, body))?;
    let counters = compute_counters(&reparsed);
    Ok((fm_apply_counters_and_v1(frontmatter_raw, &counters), counters))
}

/// Persist `body` to `path` with the counters recomputed from it. The single
/// write path for every mutating verb (`note`, `set-status`, `new`, `verify`).
pub fn write_recounted(path: &Path, frontmatter_raw: &str, body: &str) -> Result<Counters> {
    let (fm, counters) = recounted_frontmatter(path, frontmatter_raw, body)?;
    std::fs::write(path, assemble(&fm, body))
        .with_context(|| format!("Failed to write registry at {}", path.display()))?;
    Ok(counters)
}

/// Compose the new value of an entry's `Notes` bullet when appending an
/// annotation.
///
/// `Notes` is a single-line bullet (the parser reads one line per field), so
/// accumulating annotations means composing, not stacking. Each is stamped
/// `[date]` — and `[date · source]` when the caller passes what motivated it —
/// so an entry's note history stays attributable instead of becoming an
/// undated blur. Pure, so the composition is unit-testable.
pub fn append_note(existing: Option<&str>, text: &str, date: &str, source: Option<&str>) -> String {
    let stamp = match source.map(str::trim).filter(|s| !s.is_empty()) {
        Some(src) => format!("[{} · {}]", date, src),
        None => format!("[{}]", date),
    };
    let addition = format!("{} {}", stamp, text.trim());
    match existing.map(str::trim).filter(|s| !s.is_empty()) {
        Some(prev) => format!("{} · {}", prev, addition),
        None => addition,
    }
}

/// Render an entry declared **ex-ante** — at Charter-declaration time, before
/// any execution (#360).
///
/// Deliberately not [`render_new_entry`]: that one serves `drift --apply` and
/// stamps a `Source-hash` derived from the originating AILOG. An ex-ante entry
/// has no AILOG to hash (the Charter precedes execution by design), and
/// inventing a hash would make a later `drift --apply` believe it had already
/// extracted something it never saw. Omitting the field is the honest encoding:
/// the dedup key is absent because the source it keys on does not exist yet.
#[allow(clippy::too_many_arguments)]
pub fn render_declared_entry(
    fu_number: u32,
    title: &str,
    origin: &str,
    status: &str,
    trigger: Option<&str>,
    destination: Option<&str>,
    cost: Option<&str>,
    premise: Option<&str>,
    notes: &str,
) -> String {
    let field = |v: Option<&str>| {
        v.map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("TBD")
            .to_string()
    };
    let mut block = format!(
        "### FU-{:03} — {}\n\
         - **Origin**: {}\n\
         - **Origin-class**: ex-ante-planning\n\
         - **Status**: {}\n\
         - **Trigger**: {}\n\
         - **Destination**: {}\n\
         - **Cost**: {}\n",
        fu_number,
        title.trim(),
        origin.trim(),
        status,
        field(trigger),
        field(destination),
        field(cost),
    );
    if let Some(p) = premise.map(str::trim).filter(|s| !s.is_empty()) {
        block.push_str(&format!("- **Premise**: {}\n", p));
    }
    block.push_str(&format!("- **Notes**: {}\n", notes.trim()));
    block
}

// ── Registry merging (GH #391) ─────────────────────────────────────────────
//
// The registry is CLI-owned, so a textual three-way merge of it during a git
// conflict resolution is always wrong — and the sanctioned fallback (take
// `main`'s file, re-run `drift --apply`) silently reverted the closures the
// branch had made, because statuses live only in the file.
//
// This module powers `straymark followups merge-driver`, a git merge driver
// that resolves the registry structurally: entries are matched across sides
// by title (ids are positional and get renumbered, titles survive), and a
// non-open status wins over open so closures made on either side are kept.

/// Resolution rank of a status — higher means "closer to resolved". Used by
/// the merge driver to decide which side's status survives when both sides
/// touched the same entry.
pub fn status_rank(s: FuStatus) -> u8 {
    match s {
        FuStatus::Open | FuStatus::Unknown => 0,
        FuStatus::InProgress => 1,
        FuStatus::SuspectedClosed => 2,
        FuStatus::Closed | FuStatus::Superseded | FuStatus::Promoted => 3,
    }
}

/// Canonical identity of an entry for cross-side matching. Ids are positional
/// (a regeneration renumbers them), so the title is the only stable key.
pub fn normalize_title(description: &str) -> String {
    description
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Outcome of [`merge_registries`].
#[derive(Debug, Default)]
pub struct MergeReport {
    /// Entries where theirs' more-advanced status was applied onto ours.
    pub statuses_preserved: usize,
    /// Entries present only in theirs, appended to the result.
    pub appended: usize,
    /// Entries theirs deleted (present in base, absent in theirs) that ours
    /// had not modified — respected as deletions.
    pub deletions_respected: usize,
    /// Same-rank status disagreements (e.g. closed vs superseded) — ours is
    /// kept; the operator should eyeball these.
    pub conflicts: Vec<String>,
}

/// Three-way merge of parsed registries. Returns the merged file content
/// (frontmatter + body) and a report. `ours` is the textual base of the
/// result (its ids, ordering, and unknown fields survive verbatim); entries
/// are matched across sides by [`normalize_title`].
///
/// Rules:
/// - status: the higher-rank one wins; equal-rank disagreements keep ours
///   and are reported as conflicts;
/// - notes: theirs wins when ours is empty or a prefix of theirs (the
///   append-only `followups note` shape), else ours is kept;
/// - entries only in theirs: appended (renumbered on id collision);
///   an entry present in base but absent from ours was deleted by ours and
///   is only re-added when theirs advanced its status past base's;
/// - entries deleted by theirs (in base, gone in theirs) are dropped from
///   ours unless ours changed their status (modify/delete → kept + conflict).
pub fn merge_registries(
    base: &Registry,
    ours: &Registry,
    theirs: &Registry,
) -> Result<(String, MergeReport)> {
    let mut report = MergeReport::default();

    let base_by_key: std::collections::HashMap<String, &Entry> = base
        .entries()
        .map(|e| (normalize_title(&e.description), e))
        .collect();
    let ours_by_key: std::collections::HashMap<String, &Entry> = ours
        .entries()
        .map(|e| (normalize_title(&e.description), e))
        .collect();
    let theirs_by_key: std::collections::HashMap<String, &Entry> = theirs
        .entries()
        .map(|e| (normalize_title(&e.description), e))
        .collect();

    // 1. Deletions made by theirs: present in base and ours, absent in
    //    theirs. Respect them unless ours modified the status (kept + conflict).
    let mut removed_spans: Vec<(usize, usize, String)> = Vec::new();
    for o in ours.entries() {
        let key = normalize_title(&o.description);
        if theirs_by_key.contains_key(&key) || !base_by_key.contains_key(&key) {
            continue;
        }
        let modified = base_by_key[&key].status != o.status;
        if modified {
            report.conflicts.push(format!(
                "{}: deleted on theirs but status changed on ours ({})",
                o.fu_id,
                o.status.as_str()
            ));
            continue;
        }
        removed_spans.push((o.span_start, o.span_end, o.fu_id.clone()));
    }
    let mut body = ours.body.clone();
    removed_spans.sort_by(|a, b| b.0.cmp(&a.0)); // remove from the end
    for (start, end, _id) in &removed_spans {
        body.replace_range(*start..*end, "");
        report.deletions_respected += 1;
    }

    // 2. Field reconciliation for entries present on both sides. Re-parse
    //    after each surgical edit so spans stay valid.
    for t in theirs.entries() {
        let key = normalize_title(&t.description);
        let Some(o) = ours_by_key.get(&key) else { continue };
        if removed_spans.iter().any(|(_, _, id)| id == &o.fu_id) {
            continue;
        }
        {
            let current =
                parse_registry_str(&ours.path, &assemble(&ours.frontmatter_raw, &body))?;
            let found = current.entries().find(|e| e.fu_id == o.fu_id);
            if let Some(current_entry) = found {
                if t.status != current_entry.status {
                    if status_rank(t.status) > status_rank(current_entry.status) {
                        let value = t
                            .status_raw
                            .clone()
                            .unwrap_or_else(|| t.status.as_str().to_string());
                        body = set_entry_field(&body, current_entry, "Status", &value);
                        report.statuses_preserved += 1;
                    } else if status_rank(t.status) == status_rank(current_entry.status) {
                        report.conflicts.push(format!(
                            "{}: status {} (ours) vs {} (theirs) — kept ours",
                            o.fu_id,
                            current_entry.status.as_str(),
                            t.status.as_str()
                        ));
                    }
                }
            }
        }

        // Notes: theirs wins when it is an extension of ours (append-only notes).
        let current = parse_registry_str(&ours.path, &assemble(&ours.frontmatter_raw, &body))?;
        let found = current.entries().find(|e| e.fu_id == o.fu_id);
        if let Some(current_entry) = found {
            match (&current_entry.notes, &t.notes) {
                (Some(o_notes), Some(t_notes)) if t_notes != o_notes && t_notes.starts_with(o_notes) => {
                    body = set_entry_field(&body, current_entry, "Notes", t_notes);
                }
                (None, Some(t_notes)) => {
                    body = set_entry_field(&body, current_entry, "Notes", t_notes);
                }
                _ => {}
            }
        }
    }

    // 3. Entries only in theirs: append (or re-add a deletion only when
    //    theirs advanced the status past base's).
    let mut taken_numbers: std::collections::HashSet<u32> =
        ours.entries().map(|e| e.fu_number).collect();
    let mut next_n = next_fu_number(ours);
    for t in theirs.entries() {
        let key = normalize_title(&t.description);
        if ours_by_key.contains_key(&key) {
            continue;
        }
        if let Some(b) = base_by_key.get(&key) {
            // Ours deleted it. Re-add only when theirs advanced its status.
            if status_rank(t.status) <= status_rank(b.status) {
                continue;
            }
        }
        let mut block = theirs.body[t.span_start..t.span_end].to_string();
        let number = if taken_numbers.contains(&t.fu_number) {
            while taken_numbers.contains(&next_n) {
                next_n += 1;
            }
            let n = next_n;
            next_n += 1;
            block = block.replacen(
                &format!("### {} ", t.fu_id),
                &format!("### FU-{:03} ", n),
                1,
            );
            n
        } else {
            t.fu_number
        };
        taken_numbers.insert(number);

        let current = parse_registry_str(&ours.path, &assemble(&ours.frontmatter_raw, &body))?;
        let bucket = if current.sections.iter().any(|s| s.is_bucket && s.name == t.bucket) {
            t.bucket.clone()
        } else {
            "ready".to_string()
        };
        body = insert_into_bucket(&current, &bucket, &block);
        report.appended += 1;
    }

    // 4. Frontmatter: union fully_extracted_ailogs, take the newest
    //    last_scan, then recompute the CLI-owned counters from the merged body.
    let already: std::collections::HashSet<&str> = ours
        .frontmatter
        .fully_extracted_ailogs
        .iter()
        .map(|s| s.as_str())
        .collect();
    let new_ids: Vec<String> = theirs
        .frontmatter
        .fully_extracted_ailogs
        .iter()
        .filter(|id| !already.contains(id.as_str()))
        .cloned()
        .collect();
    let mut fm = fm_append_list_items(&ours.frontmatter_raw, "fully_extracted_ailogs", &new_ids);
    let newest_scan = match (&ours.frontmatter.last_scan, &theirs.frontmatter.last_scan) {
        (Some(a), Some(b)) => std::cmp::max(a.as_str(), b.as_str()).to_string(),
        (Some(a), None) => a.clone(),
        (None, Some(b)) => b.clone(),
        (None, None) => String::new(),
    };
    if !newest_scan.is_empty() {
        fm = fm_set_scalar(&fm, "last_scan", &newest_scan);
    }
    let (fm, _counters) = recounted_frontmatter(&ours.path, &fm, &body)?;

    Ok((assemble(&fm, &body), report))
}

#[cfg(test)]
mod tests {
    use super::*;

    const V0_REGISTRY: &str = r#"---
last_scan: 2026-05-06
schema_version: v0
total_open: 47
total_promoted: 3
total_closed_in_session: 2
total_phase_blocked: 1
buckets:
  - ready
  - time-triggered
  - charter-triggered
  - phase-blocked
  - operational
fully_extracted_ailogs:
  - AILOG-2026-04-11-001
  - AILOG-2026-04-12-001
custom_field: kept
---

# Follow-ups Backlog

## Bucket: ready

### FU-001 — Wire the retry budget into the sync loop
- **Origin**: AILOG-2026-04-11-001 §Follow-ups
- **Status**: open
- **Trigger**: ready
- **Destination**: operations
- **Cost**: S
- **Notes**: first entry

### FU-002 — Validate flake on month boundary
- **Origin**: AILOG-2026-04-12-001 §R5 (new, not in Charter)
- **Status**: open
- **Trigger**: when next month boundary passes in CI
- **Destination**: TBD
- **Cost**: TBD

## Bucket: phase-blocked

### FU-003 — Phase 6 dashboard hook
- **Origin**: AILOG-2026-04-12-001 §Follow-ups
- **Status**: open
- **Trigger**: when Phase 6 exists
- **Destination**: Phase 6+
- **Cost**: M

## Promoted to TDE

### FU-004 — Transversal auth debt
- **Origin**: AILOG-2026-04-11-001 §R2 (new, not in Charter)
- **Status**: promoted
- **Promoted to**: TDE-2026-05-01-001
"#;

    const V1_ENTRY: &str = r#"---
schema_version: v1
last_scan: 2026-06-03
buckets: [ready]
fully_extracted_ailogs: []
---

## Bucket: ready

### FU-010 — Harden staging probe
- **Origin**: AILOG-2026-06-01-002 §Follow-ups
- **Origin-class**: staging
- **Status**: open
- **Severity**: blocking
- **Trigger**: ready
- **Destination**: mini-charter
- **Cost**: M
- **Labels**: staging-hardening, reliability
- **Notes**: PROD path
"#;

    fn parse(content: &str) -> Registry {
        parse_registry_str(Path::new("follow-ups-backlog.md"), content).unwrap()
    }

    #[test]
    fn parses_v0_registry_leniently() {
        let reg = parse(V0_REGISTRY);
        assert!(reg.is_v0());
        assert_eq!(reg.frontmatter.fully_extracted_ailogs.len(), 2);
        assert_eq!(reg.entries().count(), 4);
        let fu1 = find_entry(&reg, "FU-001").unwrap();
        assert_eq!(fu1.status, FuStatus::Open);
        assert_eq!(fu1.bucket, "ready");
        // v1 fields absent → None / empty, never an error.
        assert!(fu1.severity.is_none());
        assert!(fu1.origin_class.is_none());
        assert!(fu1.labels.is_empty());
        assert!(reg.warnings.is_empty());
    }

    #[test]
    fn parses_v1_entry_dimensions() {
        let reg = parse(V1_ENTRY);
        assert!(!reg.is_v0());
        let e = find_entry(&reg, "10").unwrap();
        assert_eq!(e.severity, Some(Severity::Blocking));
        assert_eq!(e.origin_class.as_deref(), Some("staging"));
        assert_eq!(e.labels, vec!["staging-hardening", "reliability"]);
        assert_eq!(e.destination.as_deref(), Some("mini-charter"));
    }

    #[test]
    fn parses_and_sets_premise_and_verified_at() {
        // Parse the optional #365-Part-1 fields (lenient aliases too).
        let content = "---\nschema_version: v1\nfully_extracted_ailogs: []\n---\n\n\
            ## Bucket: ready\n\n### FU-020 — Loro parity gate\n\
            - **Status**: open\n\
            - **Premise**: yrs has an independent reference (Yjs)\n\
            - **Verified at**: 2026-07-18\n";
        let reg = parse(content);
        let e = find_entry(&reg, "20").unwrap();
        assert_eq!(e.premise.as_deref(), Some("yrs has an independent reference (Yjs)"));
        assert_eq!(e.verified_at.as_deref(), Some("2026-07-18"));

        // set_entry_field round-trips both (insert-if-absent + replace).
        let e = e.clone();
        let body2 = set_entry_field(&reg.body, &e, "Premise", "the reference cannot exist (WASM of same core)");
        let reg2 = parse(&assemble(&reg.frontmatter_raw, &body2));
        let e2 = find_entry(&reg2, "20").unwrap();
        assert_eq!(e2.premise.as_deref(), Some("the reference cannot exist (WASM of same core)"));
        assert_eq!(e2.verified_at.as_deref(), Some("2026-07-18")); // untouched
    }

    #[test]
    fn malformed_fu_heading_is_warning_not_error() {
        let content = r#"---
schema_version: v0
fully_extracted_ailogs: []
---

## Bucket: ready

### FU- — missing number
- **Status**: open

### FU-007 — good entry
- **Status**: open
"#;
        let reg = parse(content);
        assert_eq!(reg.entries().count(), 1);
        assert_eq!(reg.warnings.len(), 1);
        assert!(reg.warnings[0].contains("Malformed"));
    }

    #[test]
    fn glued_entry_heading_is_invisible_but_surfaces_a_warning() {
        // FU-158's heading is glued to FU-157's last line (no blank line before
        // `### `), exactly the silent-under-count failure mode of #253: the
        // parser sees one entry, the file holds two.
        let content = "---\nschema_version: v1\nfully_extracted_ailogs: []\ntotal_open: 1\n---\n\n## Bucket: operational\n\n### FU-157 — first\n- **Status**: open\n- Notes: ends without a blank line.### FU-158 — glued second\n- **Status**: open\n";
        let reg = parse(content);
        // The glued FU-158 is invisible to the section parser …
        assert_eq!(reg.entries().count(), 1);
        // … but the structural integrity check surfaces it.
        assert_eq!(reg.warnings.len(), 1);
        assert!(reg.warnings[0].contains("invisible to the counters"));
        assert!(reg.warnings[0].starts_with("1 `### FU-NNN`"));
    }

    #[test]
    fn entry_before_first_section_surfaces_a_warning() {
        // A well-formed entry placed before any `## ` section is also invisible.
        let content = "---\nschema_version: v1\nfully_extracted_ailogs: []\ntotal_open: 0\n---\n\n### FU-200 — orphan above all sections\n- **Status**: open\n\n## Bucket: ready\n\n### FU-201 — in a section\n- **Status**: open\n";
        let reg = parse(content);
        assert_eq!(reg.entries().count(), 1);
        assert_eq!(reg.warnings.len(), 1);
        assert!(reg.warnings[0].contains("invisible to the counters"));
    }

    #[test]
    fn well_formed_registry_has_no_integrity_warning() {
        // Sanity: the integrity check must not false-positive on a clean file.
        let reg = parse(V0_REGISTRY);
        assert!(
            reg.warnings.is_empty(),
            "unexpected warnings: {:?}",
            reg.warnings
        );
    }

    #[test]
    fn non_bucket_sections_still_collect_entries() {
        let reg = parse(V0_REGISTRY);
        let promoted = reg
            .sections
            .iter()
            .find(|s| !s.is_bucket && s.name == "Promoted to TDE")
            .unwrap();
        assert_eq!(promoted.entries.len(), 1);
        assert_eq!(promoted.entries[0].status, FuStatus::Promoted);
    }

    #[test]
    fn bucket_heading_tolerates_trailing_annotation() {
        let content = "---\nschema_version: v0\nfully_extracted_ailogs: []\n---\n\n## Bucket: ready          (1 entry)\n\n### FU-001 — x\n- **Status**: open\n";
        let reg = parse(content);
        assert_eq!(reg.sections[0].name, "ready");
        assert!(reg.sections[0].is_bucket);
    }

    #[test]
    fn find_entry_accepts_bare_and_padded_numbers() {
        let reg = parse(V0_REGISTRY);
        assert_eq!(find_entry(&reg, "FU-002").unwrap().fu_number, 2);
        assert_eq!(find_entry(&reg, "002").unwrap().fu_number, 2);
        assert_eq!(find_entry(&reg, "2").unwrap().fu_number, 2);
        assert!(find_entry(&reg, "99").is_none());
        assert!(find_entry(&reg, "").is_none());
    }

    #[test]
    fn next_fu_number_is_max_plus_one() {
        let reg = parse(V0_REGISTRY);
        assert_eq!(next_fu_number(&reg), 5);
    }

    #[test]
    fn counters_recompute_from_statuses_not_frontmatter() {
        // Frontmatter claims total_open: 47 — the real count is 3 (#214 Signal 2).
        let reg = parse(V0_REGISTRY);
        let c = compute_counters(&reg);
        assert_eq!(c.open, 3);
        assert_eq!(c.promoted, 1);
        assert_eq!(c.phase_blocked_open, 1);
        assert_eq!(c.total, 4);
    }

    #[test]
    fn blocking_open_counts_open_and_in_progress_blocking() {
        let reg = parse(V1_ENTRY);
        let c = compute_counters(&reg);
        assert_eq!(c.blocking_open, 1);
    }

    #[test]
    fn fm_set_scalar_replaces_and_appends() {
        let fm = "schema_version: v0\ntotal_open: 47";
        let out = fm_set_scalar(fm, "schema_version", "v1");
        assert!(out.contains("schema_version: v1"));
        let out = fm_set_scalar(&out, "total_suspected_closed", "0");
        assert!(out.ends_with("total_suspected_closed: 0"));
    }

    #[test]
    fn fm_append_list_items_extends_block_list() {
        let fm = "schema_version: v0\nfully_extracted_ailogs:\n  - AILOG-2026-04-11-001\nbuckets:\n  - ready";
        let out = fm_append_list_items(fm, "fully_extracted_ailogs", &["AILOG-2026-06-03-001".to_string()]);
        let idx_old = out.find("AILOG-2026-04-11-001").unwrap();
        let idx_new = out.find("AILOG-2026-06-03-001").unwrap();
        assert!(idx_new > idx_old);
        // The new item must land inside the list, before `buckets:`.
        assert!(idx_new < out.find("buckets:").unwrap());
    }

    #[test]
    fn fm_append_list_items_converts_empty_flow_list() {
        let fm = "schema_version: v1\nfully_extracted_ailogs: []";
        let out = fm_append_list_items(fm, "fully_extracted_ailogs", &["AILOG-2026-06-03-001".to_string()]);
        assert!(out.contains("fully_extracted_ailogs:\n  - AILOG-2026-06-03-001"));
        // Result must be parseable YAML with one item.
        let parsed: RegistryFrontmatter = serde_yaml::from_str(&out).unwrap();
        assert_eq!(parsed.fully_extracted_ailogs.len(), 1);
    }

    #[test]
    fn v0_upgrade_preserves_unknown_fields() {
        let reg = parse(V0_REGISTRY);
        let counters = compute_counters(&reg);
        let fm = fm_apply_counters_and_v1(&reg.frontmatter_raw, &counters);
        assert!(fm.contains("schema_version: v1"));
        assert!(fm.contains("custom_field: kept"), "unknown fields must survive");
        assert!(fm.contains("total_open: 3"));
        // Idempotent: applying again changes nothing.
        let fm2 = fm_apply_counters_and_v1(&fm, &counters);
        assert_eq!(fm, fm2);
    }

    #[test]
    fn extract_followups_finds_section_bullets_and_risk_lines() {
        let ailog = r#"# AILOG-2026-06-03-003 — staging run

## Risk

- **R3 (new, not in Charter)**: bus handler writes escape the unit suite.

## Follow-ups

- Extend E2E coverage to write-path-A
- Formal validation run — closed in-Charter (commit `ab12cd34ef`), 5/6 pass

## Outcome

Done.
"#;
        let found = extract_followups_from_ailog(ailog);
        assert_eq!(found.len(), 3);
        let bullets: Vec<&str> = found.iter().map(|f| f.description.as_str()).collect();
        assert!(bullets.iter().any(|b| b.contains("Extend E2E coverage")));
        let closed = found
            .iter()
            .find(|f| f.description.contains("Formal validation"))
            .unwrap();
        assert!(closed.suspected_closed, "closure marker must be detected");
        let open = found
            .iter()
            .find(|f| f.description.contains("Extend E2E"))
            .unwrap();
        assert!(!open.suspected_closed);
        let risk = found
            .iter()
            .find(|f| f.origin_section.contains("R3"))
            .unwrap();
        assert!(risk.origin_section.contains("(new, not in Charter)"));
    }

    #[test]
    fn extract_followups_recognizes_localized_section_headings() {
        // A Spanish or Chinese AILOG heading must still yield the bullets (#263):
        // an English-literal match would extract nothing for those adopters.
        let es = "# AILOG\n\n## Seguimientos\n\n- Extender cobertura E2E\n- Documentar el runbook\n";
        let zh = "# AILOG\n\n## 后续工作\n\n- 扩展端到端测试覆盖\n- 编写部署手册\n";
        assert_eq!(extract_followups_from_ailog(es).len(), 2);
        assert_eq!(extract_followups_from_ailog(zh).len(), 2);
    }

    // ── #346: extraction-fidelity field report ────────────────────────────

    #[test]
    fn extract_captures_suffixed_followups_heading() {
        // Under-capture: an explicit `## Follow-ups (auditoría externa)` was
        // skipped by the old exact-equality matcher.
        let ailog = "# AILOG\n\n## Follow-ups (auditoría externa)\n\n- Deferral: optional capability X\n- real_debt: wire the retry path\n";
        let found = extract_followups_from_ailog(ailog);
        assert_eq!(found.len(), 2, "suffixed heading must still be captured");
        assert!(found.iter().all(|f| f.origin_section == "§Follow-ups"));
    }

    #[test]
    fn extract_collects_multiple_followups_sections() {
        // A plain section and an audit-scoped one coexist — both extracted.
        let ailog = "# AILOG\n\n## Follow-ups\n\n- item A\n\n## Follow-ups (external audit)\n\n- item B\n- item C\n";
        let found = extract_followups_from_ailog(ailog);
        let descs: Vec<&str> = found.iter().map(|f| f.description.as_str()).collect();
        assert!(descs.contains(&"item A"));
        assert!(descs.contains(&"item B"));
        assert!(descs.contains(&"item C"));
        assert_eq!(found.len(), 3);
    }

    #[test]
    fn extract_skips_prose_mentioning_risk_phrase() {
        // Over-capture: a prose summary line that merely mentions the phrase is
        // NOT a follow-up (FU-001 in the report).
        let ailog = "# AILOG\n\n## Summary\n\nRiesgos R1–R5 mitigados en el Charter. Emergió R6 (new, not in Charter) durante la ejecución.\n";
        let found = extract_followups_from_ailog(ailog);
        assert!(
            found.is_empty(),
            "prose line must not be extracted, got: {found:?}"
        );
    }

    #[test]
    fn extract_heading_risk_with_inline_remediation_is_suspected_closed() {
        // Resolved-as-open: a `## Risk: R6` heading fixed in the same charter,
        // with remediation documented in the body, must land suspected-closed.
        let ailog = "# AILOG\n\n## Risk: R6 (new, not in Charter): UTF-16 offset\n\nEl offset se calculaba en UTF-16. Corregido a code points en este batch.\n\n### Mitigaciones aplicadas\n\n- test de regresión añadido\n";
        let found = extract_followups_from_ailog(ailog);
        let r6 = found
            .iter()
            .find(|f| f.origin_section.contains("R6"))
            .expect("R6 risk extracted");
        assert!(
            r6.suspected_closed,
            "in-charter remediation must mark the risk suspected-closed"
        );
    }

    #[test]
    fn extract_bullet_risk_still_extracted_open_when_unresolved() {
        // Parity: an unresolved bullet-style risk stays open, hash-stable desc.
        let ailog = "# AILOG\n\n## Risk\n\n- **R7 (new, not in Charter)**: Loro export mode needs review.\n";
        let found = extract_followups_from_ailog(ailog);
        assert_eq!(found.len(), 1);
        assert!(found[0].origin_section.contains("R7"));
        assert!(!found[0].suspected_closed);
        assert_eq!(
            found[0].description,
            "R7 (new, not in Charter): Loro export mode needs review."
        );
    }

    #[test]
    fn entry_title_uses_leading_bold_and_unwraps_hardwrapped_bullets() {
        // The three real Weft shapes (#365): each leads with a bold title and
        // hard-wraps the prose. The pre-#365 first-line title truncated at the
        // wrap column ("…el pack lee de", "…**dos afirmaciones falsas**, y ya").
        let ailog = "# AILOG\n\n## Follow-ups\n\n\
            - **Comentario obsoleto en `ci.yml:76-81`** — **dos afirmaciones falsas**, y ya\n  \
            demostró que hace daño: (1) dice que el job es continue-on-error.\n\
            - **Footgun de pack local contaminado con `test-hooks`**: el pack lee de\n  \
            `native/target/<triple>/release/`. Quien compile con la feature.\n\
            - **Guards de CI para las sub-clases del anti-patrón** (paso 5 del walkthrough). El\n  \
            gap #1 es mecánicamente detectable.\n";
        let found = extract_followups_from_ailog(ailog);
        let titles: Vec<&str> = found.iter().map(|f| f.title.as_str()).collect();
        assert_eq!(
            titles,
            vec![
                "Comentario obsoleto en `ci.yml:76-81`",
                "Footgun de pack local contaminado con `test-hooks`",
                "Guards de CI para las sub-clases del anti-patrón",
            ]
        );
        // The hash seed (`description`) stays the raw first line — unchanged, so
        // dedup is unperturbed by the nicer title (#365).
        assert_eq!(
            found[1].description,
            "**Footgun de pack local contaminado con `test-hooks`**: el pack lee de"
        );
    }

    #[test]
    fn entry_title_falls_back_to_first_sentence_and_caps() {
        // No bold lead → strip markup, take the first sentence.
        let ailog = "# AILOG\n\n## Follow-ups\n\n\
            - Reponer el job de CI del adaptador Redis con un service container.\n  \
            Corre local con Valkey y se omite en CI hoy.\n";
        let found = extract_followups_from_ailog(ailog);
        assert_eq!(
            found[0].title,
            "Reponer el job de CI del adaptador Redis con un service container."
        );

        // A runaway lead with no early sentence break caps at a word boundary.
        let long_word = "palabra ".repeat(40);
        let ailog2 = format!("# AILOG\n\n## Follow-ups\n\n- {long_word}fin\n");
        let found2 = extract_followups_from_ailog(&ailog2);
        assert!(found2[0].title.ends_with('…'));
        assert!(found2[0].title.chars().count() <= 121); // 120 + the ellipsis
        assert!(!found2[0].title.contains("  ")); // trimmed cleanly at a boundary
    }

    #[test]
    fn entry_title_ignores_trivial_bold_tag() {
        // A bare `**R6**`-style tag is too short to be a title — fall through to
        // the first sentence so the title carries context.
        let ailog = "# AILOG\n\n## Follow-ups\n\n- **R6** endurecer el decoder ante amplificación de memoria.\n";
        let found = extract_followups_from_ailog(ailog);
        assert_eq!(
            found[0].title,
            "R6 endurecer el decoder ante amplificación de memoria."
        );
    }

    #[test]
    fn is_followup_heading_matches_prefix_not_substring() {
        assert!(is_followup_heading("Follow-ups"));
        assert!(is_followup_heading("Follow-ups (auditoría externa)"));
        assert!(is_followup_heading("Seguimientos: deuda"));
        assert!(is_followup_heading("后续工作"));
        // Not a follow-ups heading — token is a proper substring, not a prefix.
        assert!(!is_followup_heading("Pending Follow-ups"));
        assert!(!is_followup_heading("Outcome"));
    }

    #[test]
    fn risk_declaration_token_distinguishes_structure_from_prose() {
        assert_eq!(
            risk_declaration_token("## Risk: R6 (new, not in Charter): x"),
            Some("R6".to_string())
        );
        assert_eq!(
            risk_declaration_token("- **R3 (new, not in Charter)**: y"),
            Some("R3".to_string())
        );
        // Prose that mentions the phrase but does not begin with the token.
        assert_eq!(
            risk_declaration_token("Emergió R6 (new, not in Charter) al final."),
            None
        );
        // No phrase at all.
        assert_eq!(risk_declaration_token("- **R3**: no phrase here"), None);
    }

    #[test]
    fn contains_word_respects_boundaries() {
        assert!(contains_word("riesgo corregido a code points", "corregido"));
        assert!(!contains_word("el punto no fue irresuelto aun", "resuelto"));
    }

    #[test]
    fn closure_markers_detected_case_insensitively() {
        assert!(has_closure_marker("Closed in-Charter by the runbook rewrite"));
        assert!(has_closure_marker("fixed in batch 3"));
        assert!(has_closure_marker("resolved in Charter close"));
        assert!(has_closure_marker("see commit `deadbeef42` for the fix"));
        assert!(!has_closure_marker("should be closed when X lands"));
        assert!(!has_closure_marker("fixed in batch processing generally"));
        // Backtick word that isn't a hash (no digit / not hex).
        assert!(!has_closure_marker("see `feedface` once")); // hex but no digit
        assert!(!has_closure_marker("run `cargo test` first"));
    }

    #[test]
    fn closure_markers_born_resolved_idiom_family() {
        // #222 Finding 2 — the exact lnxdrive phrasing that landed as `open`.
        assert!(has_closure_marker(
            "Charter `## Files to modify` row updated atomically in this PR."
        ));
        assert!(has_closure_marker("remediated in this PR"));
        assert!(has_closure_marker("the regression was Corrected in this commit"));
        assert!(has_closure_marker("scope drift recognized and fixed in this PR"));
        // Verb required — context phrase alone is not a closure.
        assert!(!has_closure_marker("discussed in this PR"));
        assert!(!has_closure_marker("tracked in this PR for visibility"));
        // Context phrase required — future-tense / follow-up phrasing is not.
        assert!(!has_closure_marker("will be updated in a follow-up PR"));
        assert!(!has_closure_marker("should be corrected in the next commit"));
    }

    #[test]
    fn ailog_id_from_path_takes_five_tokens() {
        assert_eq!(
            ailog_id_from_path(Path::new("AILOG-2026-06-03-003-staging-incident.md")).as_deref(),
            Some("AILOG-2026-06-03-003")
        );
        assert_eq!(
            ailog_id_from_path(Path::new("AILOG-2026-06-03-004.md")).as_deref(),
            Some("AILOG-2026-06-03-004")
        );
        assert!(ailog_id_from_path(Path::new("README.md")).is_none());
    }

    #[test]
    fn insert_into_bucket_appends_at_section_end() {
        let reg = parse(V0_REGISTRY);
        let block = render_new_entry(
            5,
            &ExtractedFu {
                title: "New thing".to_string(),
                description: "New thing".to_string(),
                origin_section: "§Follow-ups".to_string(),
                suspected_closed: false,
            },
            "AILOG-2026-06-03-001",
            "2026-06-04",
        );
        let new_body = insert_into_bucket(&reg, "ready", &block);
        // FU-005 must land inside the ready section: after FU-002, before
        // `## Bucket: phase-blocked`.
        let idx_new = new_body.find("### FU-005").unwrap();
        assert!(idx_new > new_body.find("### FU-002").unwrap());
        assert!(idx_new < new_body.find("## Bucket: phase-blocked").unwrap());
        // Re-parse: 5 entries now.
        let reparsed = parse(&assemble(&reg.frontmatter_raw, &new_body));
        assert_eq!(reparsed.entries().count(), 5);
        assert_eq!(find_entry(&reparsed, "FU-005").unwrap().bucket, "ready");
    }

    #[test]
    fn insert_into_bucket_creates_missing_section() {
        let content = "---\nschema_version: v1\nfully_extracted_ailogs: []\n---\n\n# Registry\n";
        let reg = parse(content);
        let new_body = insert_into_bucket(&reg, "ready", "### FU-001 — x\n- **Status**: open\n");
        assert!(new_body.contains("## Bucket: ready"));
        let reparsed = parse(&assemble(&reg.frontmatter_raw, &new_body));
        assert_eq!(reparsed.entries().count(), 1);
    }

    #[test]
    fn set_entry_field_replaces_existing_bullet() {
        let reg = parse(V0_REGISTRY);
        let entry = find_entry(&reg, "FU-001").unwrap().clone();
        let new_body = set_entry_field(&reg.body, &entry, "Status", "promoted");
        let reparsed = parse(&assemble(&reg.frontmatter_raw, &new_body));
        assert_eq!(
            find_entry(&reparsed, "FU-001").unwrap().status,
            FuStatus::Promoted
        );
        // Other entries untouched.
        assert_eq!(find_entry(&reparsed, "FU-002").unwrap().status, FuStatus::Open);
    }

    #[test]
    fn set_entry_field_appends_missing_bullet() {
        let reg = parse(V0_REGISTRY);
        let entry = find_entry(&reg, "FU-001").unwrap().clone();
        let new_body = set_entry_field(&reg.body, &entry, "Promoted to", "TDE-2026-06-04-001");
        let reparsed = parse(&assemble(&reg.frontmatter_raw, &new_body));
        assert_eq!(
            find_entry(&reparsed, "FU-001").unwrap().promoted_to.as_deref(),
            Some("TDE-2026-06-04-001")
        );
    }

    #[test]
    fn set_entry_field_preserves_the_blank_line_before_the_next_heading() {
        // A field write used to collapse the entry's trailing "\n\n" to "\n",
        // gluing the edited entry to the following heading — invisible until
        // `note` made field writes routine (CHARTER-01).
        let reg = parse(V0_REGISTRY);
        // FU-002 is the last entry of the `ready` section, so the next line is
        // a section heading — the case the collapse used to break.
        let entry = find_entry(&reg, "FU-002").unwrap().clone();
        let new_body = set_entry_field(&reg.body, &entry, "Cost", "M");
        assert!(
            new_body.contains("- **Cost**: M\n\n## Bucket: phase-blocked"),
            "blank line before the next section must survive:\n{new_body}"
        );
    }

    #[test]
    fn insert_into_bucket_keeps_a_blank_line_before_the_following_section() {
        let reg = parse(V0_REGISTRY);
        let block = "### FU-050 — Appended\n- **Status**: open\n";
        let new_body = insert_into_bucket(&reg, "ready", block);
        assert!(
            new_body.contains("### FU-050 — Appended\n- **Status**: open\n\n## Bucket: phase-blocked"),
            "appended entry must not butt against the next heading:\n{new_body}"
        );
    }

    #[test]
    fn append_note_stamps_and_composes() {
        // No prior notes: the annotation stands alone, dated.
        assert_eq!(
            append_note(None, "part-a shipped", "2026-07-26", None),
            "[2026-07-26] part-a shipped"
        );
        // With a source: attributable to what motivated it.
        assert_eq!(
            append_note(None, "part-a shipped", "2026-07-26", Some("CHARTER-04")),
            "[2026-07-26 · CHARTER-04] part-a shipped"
        );
        // Existing notes are composed onto, never replaced — Notes is a
        // single-line field, so history accumulates in one bullet.
        assert_eq!(
            append_note(Some("Extracted 2026-07-20."), "part-b deferred", "2026-07-26", None),
            "Extracted 2026-07-20. · [2026-07-26] part-b deferred"
        );
        // Whitespace-only existing notes are treated as absent.
        assert_eq!(
            append_note(Some("   "), "first", "2026-07-26", None),
            "[2026-07-26] first"
        );
        // A blank --source falls back to the bare date stamp.
        assert_eq!(
            append_note(None, "text", "2026-07-26", Some("  ")),
            "[2026-07-26] text"
        );
    }

    #[test]
    fn render_declared_entry_is_ex_ante_and_hash_less() {
        let block = render_declared_entry(
            12,
            "Redis CI job deferred",
            "CHARTER-06 §Scope",
            "open",
            None,
            None,
            Some("S"),
            Some("the Redis adapter has no CI coverage today"),
            "Created by `straymark followups new` 2026-07-26.",
        );
        assert!(block.starts_with("### FU-012 — Redis CI job deferred\n"));
        assert!(block.contains("- **Origin**: CHARTER-06 §Scope\n"));
        assert!(block.contains("- **Origin-class**: ex-ante-planning\n"));
        assert!(block.contains("- **Premise**: the Redis adapter has no CI coverage today\n"));
        // Unset optional fields become TBD, matching the template's convention.
        assert!(block.contains("- **Trigger**: TBD\n"));
        assert!(block.contains("- **Destination**: TBD\n"));
        assert!(block.contains("- **Cost**: S\n"));
        // No AILOG to hash — see the doc comment on why absence is the honest encoding.
        assert!(!block.contains("Source-hash"));

        // Round-trips through the parser as a well-formed entry.
        let reg = parse(V0_REGISTRY);
        let body = insert_into_bucket(&reg, "charter-triggered", &block);
        let reparsed = parse(&assemble(&reg.frontmatter_raw, &body));
        let entry = find_entry(&reparsed, "FU-012").unwrap();
        assert_eq!(entry.status, FuStatus::Open);
        assert_eq!(entry.origin.as_deref(), Some("CHARTER-06 §Scope"));
        assert_eq!(entry.origin_class.as_deref(), Some("ex-ante-planning"));
        assert!(entry.source_hash.is_none());
    }

    #[test]
    fn render_declared_entry_omits_premise_when_absent() {
        let block = render_declared_entry(1, "T", "CHARTER-01 §Scope", "open", None, None, None, None, "n");
        assert!(!block.contains("Premise"));
    }

    #[test]
    fn render_new_entry_marks_suspected_closed() {
        let block = render_new_entry(
            92,
            &ExtractedFu {
                title: "Formal run".to_string(),
                description: "Formal run".to_string(),
                origin_section: "§Follow-ups".to_string(),
                suspected_closed: true,
            },
            "AILOG-2026-06-03-001",
            "2026-06-04",
        );
        assert!(block.contains("### FU-092 — Formal run"));
        assert!(block.contains("- **Status**: suspected-closed"));
        assert!(block.contains("Closure marker detected"));
    }

    #[test]
    fn fu_content_hash_is_stable_12_hex_and_discriminating() {
        let h = fu_content_hash("AILOG-2026-06-09-001", "§Follow-ups", "Wire X into Y");
        assert_eq!(h.len(), 12);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
        // Deterministic across calls.
        assert_eq!(
            h,
            fu_content_hash("AILOG-2026-06-09-001", "§Follow-ups", "Wire X into Y")
        );
        // Any component change moves the hash.
        assert_ne!(
            h,
            fu_content_hash("AILOG-2026-06-09-002", "§Follow-ups", "Wire X into Y")
        );
        assert_ne!(
            h,
            fu_content_hash("AILOG-2026-06-09-001", "§Follow-ups", "Wire X into Z")
        );
    }

    #[test]
    fn split_origin_splits_id_and_section() {
        assert_eq!(
            split_origin("AILOG-2026-06-09-001 §Follow-ups"),
            Some(("AILOG-2026-06-09-001", "§Follow-ups"))
        );
        assert_eq!(
            split_origin("AILOG-2026-06-09-001 §R3 (new, not in Charter)"),
            Some(("AILOG-2026-06-09-001", "§R3 (new, not in Charter)"))
        );
        // No section suffix → None.
        assert_eq!(split_origin("AILOG-2026-06-09-001"), None);
    }

    #[test]
    fn render_new_entry_embeds_matching_source_hash() {
        let fu = ExtractedFu {
            title: "Backfill the missing migration".to_string(),
            description: "Backfill the missing migration".to_string(),
            origin_section: "§Follow-ups".to_string(),
            suspected_closed: false,
        };
        let block = render_new_entry(7, &fu, "AILOG-2026-06-09-001", "2026-06-10");
        let expected = fu_content_hash("AILOG-2026-06-09-001", "§Follow-ups", &fu.description);
        assert!(block.contains(&format!("- **Source-hash**: {}", expected)));
    }

    #[test]
    fn parse_reads_source_hash_field() {
        let content = "---\nschema_version: v1\nfully_extracted_ailogs: []\n---\n\n\
            ## Bucket: ready\n\n\
            ### FU-001 — x\n- **Origin**: AILOG-2026-06-09-001 §Follow-ups\n- **Source-hash**: abc123def456\n- **Status**: open\n";
        let reg = parse(content);
        assert_eq!(
            find_entry(&reg, "FU-001").unwrap().source_hash.as_deref(),
            Some("abc123def456")
        );
    }

    #[test]
    fn registry_extracted_hashes_prefers_stored_then_falls_back_to_legacy() {
        let aid = "AILOG-2026-06-09-001";
        let h_first = fu_content_hash(aid, "§Follow-ups", "First FU");
        // FU-001 carries a stored Source-hash; FU-002 is legacy (no hash).
        let content = format!(
            "---\nschema_version: v1\nfully_extracted_ailogs:\n  - {aid}\n---\n\n\
             ## Bucket: ready\n\n\
             ### FU-001 — First FU\n- **Origin**: {aid} §Follow-ups\n- **Source-hash**: {h_first}\n- **Status**: open\n\n\
             ### FU-002 — Second FU\n- **Origin**: {aid} §Follow-ups\n- **Status**: open\n"
        );
        let reg = parse(&content);
        let set = registry_extracted_hashes(&reg);
        // Stored hash present, legacy hash recomputed from origin + description.
        assert!(set.contains(&h_first));
        assert!(set.contains(&fu_content_hash(aid, "§Follow-ups", "Second FU")));
        // A genuinely new follow-up on the same AILOG is NOT in the set — this
        // is the #231 case (content appended after first extraction).
        assert!(!set.contains(&fu_content_hash(aid, "§Follow-ups", "Third FU")));
    }

    #[test]
    fn registry_without_frontmatter_errors_with_hint() {
        let err = parse_registry_str(Path::new("x.md"), "# no frontmatter\n").unwrap_err();
        assert!(err.to_string().contains("no YAML frontmatter"));
    }

    #[test]
    fn status_tolerates_inline_annotations_after_value() {
        // Real idiom from the Sentinel production registry (cli-3.19.1):
        // the status value carries an in-place annotation after an em dash.
        assert_eq!(
            FuStatus::from_str_loose("open — **OVERDUE** (15-Apr-2026 was 22 days ago)"),
            FuStatus::Open
        );
        assert_eq!(
            FuStatus::from_str_loose("open — mitigation in place (`-timeout` default 600s)"),
            FuStatus::Open
        );
        assert_eq!(
            FuStatus::from_str_loose("suspected-closed — confirm at triage"),
            FuStatus::SuspectedClosed
        );
        assert_eq!(
            FuStatus::from_str_loose("promoted (see TDE-2026-05-01-001)"),
            FuStatus::Promoted
        );
        // Genuinely unknown values stay Unknown — the annotation fallback
        // must not over-match.
        assert_eq!(FuStatus::from_str_loose("reopened — by audit"), FuStatus::Unknown);
        assert_eq!(FuStatus::from_str_loose(""), FuStatus::Unknown);
    }

    #[test]
    fn severity_tolerates_inline_annotations_after_value() {
        assert_eq!(
            Severity::from_str_loose("blocking — must land before prod cutover"),
            Some(Severity::Blocking)
        );
        assert_eq!(Severity::from_str_loose("normal (default)"), Some(Severity::Normal));
        assert_eq!(Severity::from_str_loose("urgent — not a vocab value"), None);
    }

    #[test]
    fn annotated_statuses_count_into_recomputed_counters() {
        // The undercount this fix prevents: an annotated `open` must count
        // as open, or the CLI-owned total_open writes the wrong number on
        // migration (observed live: Sentinel registry, 58 vs 62).
        let content = r#"---
schema_version: v0
fully_extracted_ailogs: []
---

## Bucket: ready

### FU-001 — annotated open
- **Status**: open — **OVERDUE** (15-Apr-2026)

### FU-002 — plain open
- **Status**: open
"#;
        let reg = parse(content);
        let c = compute_counters(&reg);
        assert_eq!(c.open, 2);
        assert!(reg.entries().all(|e| e.status == FuStatus::Open));
    }
}
