//! Mine existing ADRs for architecture signal: the `## Affected Components`
//! tables and the C4 mermaid diagrams the ADR template ships
//! (`dist/.straymark/templates/TEMPLATE-ADR.md`).
//!
//! Hand-written line parsers (no `pulldown-cmark`, which is `tui`-feature-gated)
//! so `architecture generate` builds with `--no-default-features` — same
//! discipline as `straymark_core::{charter_files, ailog}`. The C4 syntax mined
//! is documented in `dist/.straymark/00-governance/C4-DIAGRAM-GUIDE.md`.

/// One row of an ADR `## Affected Components` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AffectedRow {
    /// First-column text, backticks/emphasis stripped (a label or a path).
    pub component: String,
    pub change: String,
    pub impact: String,
}

/// A C4 element line: `Container(id, "Label", …)`, `Component(id, …)`,
/// `System(id, …)`, a boundary, etc. We keep the alias id and the quoted label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct C4Element {
    pub id: String,
    pub label: String,
}

/// A C4 relationship line: `Rel(from, to, "label", …)` (and its directional
/// variants). Only the endpoints matter for seeding component links.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct C4Rel {
    pub from: String,
    pub to: String,
}

/// Everything mined from a single ADR body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdrMining {
    pub affected: Vec<AffectedRow>,
    pub elements: Vec<C4Element>,
    pub rels: Vec<C4Rel>,
}

/// Mine an ADR body for both the Affected-Components table and C4 diagrams.
pub fn mine_adr_body(body: &str) -> AdrMining {
    let affected = parse_affected_components(body);
    let (elements, rels) = parse_c4(body);
    AdrMining {
        affected,
        elements,
        rels,
    }
}

/// The `## Affected Components` heading (ADR template is English-only).
const AFFECTED_HEADING: &str = "Affected Components";

/// Parse the `## Affected Components` markdown table (3 columns: Component /
/// Type of Change / Impact). Skips the header and separator rows.
pub fn parse_affected_components(body: &str) -> Vec<AffectedRow> {
    let mut out = Vec::new();
    let mut in_section = false;

    for line in body.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("## ") {
            if in_section {
                break; // next heading ends the section
            }
            if trimmed.trim_start_matches('#').trim() == AFFECTED_HEADING {
                in_section = true;
            }
            continue;
        }
        if !in_section || !trimmed.starts_with('|') {
            continue;
        }

        let cols: Vec<&str> = line.split('|').collect();
        if cols.len() < 2 {
            continue;
        }
        let col1 = cols[1].trim();
        // Separator row (only dashes/colons/spaces).
        if !col1.is_empty() && col1.chars().all(|c| matches!(c, '-' | ':' | ' ')) {
            continue;
        }
        let component = clean_cell(col1);
        // Header row.
        if component.eq_ignore_ascii_case("component") || component.is_empty() {
            continue;
        }
        out.push(AffectedRow {
            component,
            change: clean_cell(cols.get(2).map(|c| c.trim()).unwrap_or("")),
            impact: clean_cell(cols.get(3).map(|c| c.trim()).unwrap_or("")),
        });
    }

    out
}

/// Strip surrounding markdown emphasis/backticks and collapse whitespace from a
/// table cell, keeping the first backtick token if the cell is path-quoted.
fn clean_cell(cell: &str) -> String {
    let t = cell.trim().trim_matches('*').trim();
    // If the cell leads with a backtick token, prefer it (the path/name).
    if let Some(start) = t.find('`') {
        let rest = &t[start + 1..];
        if let Some(end) = rest.find('`') {
            return rest[..end].trim().to_string();
        }
    }
    t.to_string()
}

/// C4 element keyword prefixes whose first two args are `(id, "label", …)`.
/// Boundaries are included (they name a system/container grouping).
const C4_ELEMENT_PREFIXES: &[&str] = &[
    "Person_Ext",
    "Person",
    "System_Ext",
    "SystemDb",
    "SystemQueue",
    "System_Boundary",
    "System",
    "ContainerDb",
    "ContainerQueue",
    "Container_Boundary",
    "Container",
    "ComponentDb",
    "ComponentQueue",
    "Component",
    "Boundary",
];

/// Extract C4 elements and relationships from every ```mermaid fenced block
/// whose first content line starts with `C4Context` / `C4Container` /
/// `C4Component`.
pub fn parse_c4(body: &str) -> (Vec<C4Element>, Vec<C4Rel>) {
    let mut elements = Vec::new();
    let mut rels = Vec::new();
    let mut in_fence = false;
    let mut is_c4 = false;
    let mut seen_kind = false;

    for line in body.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if in_fence {
                // closing fence
                in_fence = false;
                is_c4 = false;
                seen_kind = false;
            } else {
                // opening fence; mermaid blocks are ```mermaid
                in_fence = true;
                is_c4 = false;
                seen_kind = false;
                let lang = trimmed.trim_start_matches('`').trim();
                if !lang.eq_ignore_ascii_case("mermaid") {
                    // not mermaid → ignore the whole block
                    in_fence = false;
                }
            }
            continue;
        }
        if !in_fence {
            continue;
        }
        if !seen_kind {
            if trimmed.is_empty() {
                continue;
            }
            seen_kind = true;
            is_c4 = trimmed.starts_with("C4Context")
                || trimmed.starts_with("C4Container")
                || trimmed.starts_with("C4Component");
            continue;
        }
        if !is_c4 {
            continue;
        }
        if let Some(el) = parse_c4_element(trimmed) {
            elements.push(el);
        } else if let Some(rel) = parse_c4_rel(trimmed) {
            rels.push(rel);
        }
    }

    (elements, rels)
}

/// Parse a `Keyword(id, "label", …)` element line, if the keyword is one of
/// [`C4_ELEMENT_PREFIXES`]. Boundary lines may end in `{` — that's fine, the
/// args are still inside the first `(...)`.
fn parse_c4_element(line: &str) -> Option<C4Element> {
    let kw = C4_ELEMENT_PREFIXES
        .iter()
        .find(|p| line.starts_with(&format!("{p}(")))?;
    let args = split_args(arg_slice(line, kw.len())?);
    let id = args.first()?.trim().to_string();
    if id.is_empty() {
        return None;
    }
    let label = args
        .get(1)
        .map(|s| unquote(s))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| id.clone());
    Some(C4Element { id, label })
}

/// Parse a `Rel…(from, to, …)` relationship line (any directional variant).
fn parse_c4_rel(line: &str) -> Option<C4Rel> {
    if !line.starts_with("Rel") && !line.starts_with("BiRel") {
        return None;
    }
    let paren = line.find('(')?;
    // Guard: the token before `(` must be only Rel-ish identifier chars.
    let kw = &line[..paren];
    if !kw.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    let args = split_args(&line[paren + 1..rfind_close(line, paren)?]);
    let from = args.first()?.trim().to_string();
    let to = args.get(1)?.trim().to_string();
    if from.is_empty() || to.is_empty() {
        return None;
    }
    Some(C4Rel { from, to })
}

/// The slice between the first `(` (right after a keyword of length `kw_len`)
/// and its matching `)`.
fn arg_slice(line: &str, kw_len: usize) -> Option<&str> {
    let open = kw_len; // `(` sits right after the keyword
    debug_assert_eq!(&line[open..open + 1], "(");
    let close = rfind_close(line, open)?;
    Some(&line[open + 1..close])
}

/// Index of the closing `)` matching the `(` at `open` (last `)` on the line —
/// good enough for single-line C4 statements).
fn rfind_close(line: &str, open: usize) -> Option<usize> {
    line[open..].rfind(')').map(|i| open + i)
}

/// Split top-level comma-separated args, respecting double quotes.
fn split_args(inside: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    for c in inside.chars() {
        match c {
            '"' => {
                in_quote = !in_quote;
                cur.push(c);
            }
            ',' if !in_quote => {
                out.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// Strip surrounding double quotes from a C4 arg.
fn unquote(s: &str) -> String {
    s.trim().trim_matches('"').trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn affected_components_table() {
        let body = r#"## Decision

text

## Affected Components

| Component | Type of Change | Impact |
|-----------|----------------|--------|
| `straymark-core` (`core/`) | New crate | High |
| repo root | New (`/Cargo.toml`) | Medium |

## Consequences
"#;
        let rows = parse_affected_components(body);
        assert_eq!(rows.len(), 2);
        // backtick token preferred for the first cell
        assert_eq!(rows[0].component, "straymark-core");
        assert_eq!(rows[0].change, "New crate");
        assert_eq!(rows[0].impact, "High");
        // plain-text label kept when no backtick
        assert_eq!(rows[1].component, "repo root");
    }

    #[test]
    fn affected_components_absent() {
        assert!(parse_affected_components("## Context\n\nnothing here\n").is_empty());
    }

    #[test]
    fn c4_container_elements_and_rels() {
        let body = r#"## Architecture Diagram

```mermaid
C4Container
    title Container Diagram

    Person(customer, "Customer", "Browses")

    System_Boundary(ecommerce, "E-Commerce Platform") {
        Container(webapp, "Web Application", "React", "shopping UI")
        Container(api, "API Service", "Rust", "business logic")
        ContainerDb(db, "Database", "PostgreSQL", "stores data")
    }

    Rel(customer, webapp, "Uses", "HTTPS")
    Rel(webapp, api, "Calls", "JSON")
    Rel(api, db, "Reads/Writes", "SQL")
```
"#;
        let (elements, rels) = parse_c4(body);
        let ids: Vec<&str> = elements.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"webapp"));
        assert!(ids.contains(&"api"));
        assert!(ids.contains(&"db"));
        assert!(ids.contains(&"ecommerce")); // boundary counts
        let webapp = elements.iter().find(|e| e.id == "webapp").unwrap();
        assert_eq!(webapp.label, "Web Application");
        assert_eq!(rels.len(), 3);
        assert_eq!(rels[0], C4Rel { from: "customer".into(), to: "webapp".into() });
        assert_eq!(rels[1], C4Rel { from: "webapp".into(), to: "api".into() });
    }

    #[test]
    fn non_c4_mermaid_block_ignored() {
        let body = "```mermaid\nflowchart TD\n  A --> B\n```\n";
        let (elements, rels) = parse_c4(body);
        assert!(elements.is_empty());
        assert!(rels.is_empty());
    }

    #[test]
    fn non_mermaid_fence_ignored() {
        let body = "```rust\nfn Container(x) {}\n```\n";
        let (elements, _) = parse_c4(body);
        assert!(elements.is_empty());
    }

    #[test]
    fn split_args_respects_quotes() {
        let args = split_args(r#"api, "API, Service", "Rust, Tokio""#);
        assert_eq!(args, vec!["api", "\"API, Service\"", "\"Rust, Tokio\""]);
    }
}
