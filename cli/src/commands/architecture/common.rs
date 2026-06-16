//! Shared machinery for `straymark architecture generate|sync|validate`:
//! project-root resolution, the source-tree scan, layer seeding, ADR mining /
//! enrichment, and the canonical `model.yml` renderer. Lives here (not in
//! `generate.rs`) so all three handlers compute components, globs, and YAML the
//! same way — one extractor, no drift between generate and sync.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use straymark_core::architecture::{ArchModel, Component, Layer};
use straymark_core::document::{detect_doc_type, discover_documents, DocType};
use straymark_core::drift::glob_match;

use super::adr_mining;
use crate::utils;

// The source-file walker (`collect_source_files` + its `EXCLUDED_DIRS`/
// `SOURCE_EXTENSIONS`) moved to `core::architecture::gather` in Loom A2.0 so the
// CLI generator and the Loom server share one inventory scan. Re-exported so the
// `common::collect_source_files` call sites (generate/sync/validate) are
// unchanged.
pub(crate) use straymark_core::architecture::collect_source_files;

/// The placeholder layer code-derived components land in until the human
/// assigns real layers.
pub(crate) const UNASSIGNED: &str = "unassigned";

/// Resolve the project root: `.straymark/` install → git root → the given path.
/// Does not require a `.straymark/` install (these commands scan code).
pub(crate) fn resolve_root(path: &str) -> PathBuf {
    if let Some(rp) = utils::resolve_project_root(path) {
        return rp.path;
    }
    let canon = PathBuf::from(path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(path));
    git_root(&canon).unwrap_or(canon)
}

fn git_root(dir: &Path) -> Option<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if root.is_empty() {
        None
    } else {
        Some(PathBuf::from(root))
    }
}

/// `(out_dir, model.yml, plan.drawio)` for a root + optional `--out` override.
pub(crate) fn artifact_paths(root: &Path, out: Option<&str>) -> (PathBuf, PathBuf, PathBuf) {
    let out_dir = match out {
        Some(o) => PathBuf::from(o),
        None => root.join(".straymark").join("architecture"),
    };
    let model = out_dir.join("model.yml");
    let drawio = out_dir.join("plan.drawio");
    (out_dir, model, drawio)
}

/// Distinct first-level directories under `root` that contain ≥1 source file.
pub(crate) fn top_level_source_dirs(root: &Path) -> Vec<String> {
    let mut dirs: BTreeSet<String> = BTreeSet::new();
    for rel in collect_source_files(root) {
        let mut comps = rel.components();
        let first = comps.next();
        // Require depth ≥ 2 so the first component is a directory, not a
        // root-level file.
        if comps.next().is_some() {
            if let Some(c) = first {
                dirs.insert(c.as_os_str().to_string_lossy().to_string());
            }
        }
    }
    dirs.into_iter().collect()
}

/// A component proposed from a top-level source dir (`dir/**`, `unassigned`).
pub(crate) fn component_from_dir(dir: &str) -> Component {
    Component {
        id: kebab(dir),
        label: title_case(dir),
        layer: UNASSIGNED.to_string(),
        globs: vec![format!("{dir}/**")],
        links: Vec::new(),
        docs: Vec::new(),
        external: false,
    }
}

/// Seed layers: the placeholder `unassigned` (order 0) plus the canonical
/// `.straymark/` stage directories (00–09) derived from `DocType::directory()`,
/// which the human renames to match their real architecture.
pub(crate) fn seed_layers() -> Vec<Layer> {
    let mut stages: BTreeMap<u32, String> = BTreeMap::new();
    for dt in DocType::ALL {
        let top = dt.directory().split('/').next().unwrap_or("");
        if let Some((order, name)) = split_stage(top) {
            stages.entry(order).or_insert_with(|| name.to_string());
        }
    }
    let mut layers = vec![Layer {
        id: UNASSIGNED.to_string(),
        label: "Unassigned".to_string(),
        order: 0,
    }];
    layers.extend(stages.into_iter().map(|(order, name)| Layer {
        id: format!("{order:02}-{name}"),
        label: title_case(&name),
        order,
    }));
    layers
}

/// Split a stage dir like `07-ai-audit` into `(7, "ai-audit")`.
fn split_stage(top: &str) -> Option<(u32, String)> {
    let (num, rest) = top.split_once('-')?;
    let order: u32 = num.parse().ok()?;
    Some((order, rest.to_string()))
}

/// What ADR mining changed, for the run summary.
#[derive(Default)]
pub(crate) struct Enrichment {
    pub adrs_scanned: usize,
    pub labels_improved: usize,
    pub links_added: usize,
    pub unmatched: Vec<String>,
}

/// Mine ADRs and fold the signal into the model (better labels + links).
/// Unmatched ADR components are reported, not appended (keeps the seed clean —
/// the human adds real ones during refinement).
pub(crate) fn enrich_from_adrs(root: &Path, model: &mut ArchModel) -> Enrichment {
    let mut e = Enrichment::default();
    let adrs = discover_adrs(root);
    e.adrs_scanned = adrs.len();

    // id → component index, for matching mined names.
    let by_id: BTreeMap<String, usize> = model
        .components
        .iter()
        .enumerate()
        .map(|(i, c)| (c.id.clone(), i))
        .collect();
    // C4 alias id → resolved component id (for translating Rel endpoints).
    let mut alias_to_component: BTreeMap<String, String> = BTreeMap::new();
    let mut unmatched: BTreeSet<String> = BTreeSet::new();

    for adr in &adrs {
        let Ok(content) = std::fs::read_to_string(adr) else {
            continue;
        };
        let body = utils::split_frontmatter(&content)
            .map(|(_, b)| b)
            .unwrap_or(&content);
        let mined = adr_mining::mine_adr_body(body);

        for el in &mined.elements {
            match match_component(&by_id, &model.components, &el.id) {
                Some(idx) => {
                    alias_to_component.insert(el.id.clone(), model.components[idx].id.clone());
                    if improve_label(&mut model.components[idx], &el.label) {
                        e.labels_improved += 1;
                    }
                }
                None => {
                    unmatched.insert(el.label.clone());
                }
            }
        }
        for row in &mined.affected {
            if match_component(&by_id, &model.components, &row.component).is_none() {
                unmatched.insert(row.component.clone());
            }
        }
        for rel in &mined.rels {
            let from = alias_to_component
                .get(&rel.from)
                .cloned()
                .or_else(|| by_id.contains_key(&kebab(&rel.from)).then(|| kebab(&rel.from)));
            let to = alias_to_component
                .get(&rel.to)
                .cloned()
                .or_else(|| by_id.contains_key(&kebab(&rel.to)).then(|| kebab(&rel.to)));
            if let (Some(from), Some(to)) = (from, to) {
                if from != to {
                    if let Some(&idx) = by_id.get(&from) {
                        if !model.components[idx].links.contains(&to) {
                            model.components[idx].links.push(to);
                            e.links_added += 1;
                        }
                    }
                }
            }
        }
    }

    e.unmatched = unmatched.into_iter().collect();
    e
}

/// Find the component a mined name/path refers to: by kebab id, else (when the
/// candidate is path-like) by glob match against component globs.
fn match_component(
    by_id: &BTreeMap<String, usize>,
    components: &[Component],
    candidate: &str,
) -> Option<usize> {
    if let Some(&idx) = by_id.get(&kebab(candidate)) {
        return Some(idx);
    }
    if candidate.contains('/') || candidate.contains('.') {
        let cand = candidate.trim_start_matches("./").trim_end_matches('/');
        for (i, c) in components.iter().enumerate() {
            if c.globs.iter().any(|g| glob_match(g, cand)) {
                return Some(i);
            }
        }
    }
    None
}

/// Adopt a richer ADR label when it differs from the dir-derived one.
fn improve_label(c: &mut Component, label: &str) -> bool {
    let label = label.trim();
    if label.is_empty() || label == c.label {
        return false;
    }
    c.label = label.to_string();
    true
}

/// Print the enrichment summary (shared by generate and sync).
pub(crate) fn report_enrichment(e: &Enrichment) {
    if e.adrs_scanned == 0 {
        utils::info("No ADRs found — seeded from codebase structure only.");
        return;
    }
    utils::info(&format!(
        "Mined {} ADR{}: {} label{} improved, {} link{} added.",
        e.adrs_scanned,
        plural(e.adrs_scanned),
        e.labels_improved,
        plural(e.labels_improved),
        e.links_added,
        plural(e.links_added),
    ));
    if !e.unmatched.is_empty() {
        utils::info(&format!(
            "{} ADR component{} had no code match (add by hand if relevant): {}",
            e.unmatched.len(),
            plural(e.unmatched.len()),
            e.unmatched.join(", ")
        ));
    }
}

/// ADRs under the adopter `.straymark/` tree, or this repo's `docs/decisions/`.
pub(crate) fn discover_adrs(root: &Path) -> Vec<PathBuf> {
    let straymark = root.join(".straymark");
    let base = if straymark.is_dir() {
        straymark
    } else {
        root.join("docs").join("decisions")
    };
    if !base.is_dir() {
        return Vec::new();
    }
    discover_documents(&base)
        .into_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .and_then(detect_doc_type)
                == Some(DocType::Adr)
        })
        .collect()
}

/// Render the model as canonical, comment-headed YAML (hand-written so the seed
/// can carry guidance the human reads; `serde_yaml` can't emit comments).
pub(crate) fn render_model_yaml(model: &ArchModel) -> String {
    let mut out = String::new();
    out.push_str(
        "# StrayMark architecture model (Loom Spec 002) — EXPERIMENTAL.\n\
         # First draft from `straymark architecture generate`; refine by hand:\n\
         #   - reassign components from the `unassigned` layer to a real layer;\n\
         #   - rename the seeded layers (the .straymark stages 00-09) to match\n\
         #     your architecture (e.g. Frontend / Core / Persistence);\n\
         #   - tighten globs and add links/docs as needed.\n",
    );
    out.push_str(&format!("version: {}\n", model.version));

    out.push_str("layers:\n");
    for l in &model.layers {
        out.push_str(&format!(
            "  - {{ id: {}, label: {}, order: {} }}\n",
            yaml_scalar(&l.id),
            yaml_scalar(&l.label),
            l.order
        ));
    }

    out.push_str("components:\n");
    for c in &model.components {
        out.push_str(&render_component_block(c));
    }
    out
}

/// Render one component as its `components:` list entry (shared so `sync`
/// appends byte-identically to what `generate` writes).
pub(crate) fn render_component_block(c: &Component) -> String {
    let mut out = String::new();
    out.push_str(&format!("  - id: {}\n", yaml_scalar(&c.id)));
    out.push_str(&format!("    label: {}\n", yaml_scalar(&c.label)));
    out.push_str(&format!("    layer: {}\n", yaml_scalar(&c.layer)));
    out.push_str(&format!("    globs: {}\n", yaml_flow_list(&c.globs)));
    out.push_str(&format!("    links: {}\n", yaml_flow_list(&c.links)));
    out.push_str(&format!("    docs: {}\n", yaml_flow_list(&c.docs)));
    out.push_str(&format!("    external: {}\n", c.external));
    out
}

/// Quote a scalar as a double-quoted YAML string (always safe).
fn yaml_scalar(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Render a string list in flow style: `["a", "b"]` or `[]`.
fn yaml_flow_list(items: &[String]) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }
    let inner: Vec<String> = items.iter().map(|s| yaml_scalar(s)).collect();
    format!("[{}]", inner.join(", "))
}

/// kebab-case an identifier: lowercase, non-alphanumerics → `-`, collapsed.
pub(crate) fn kebab(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in s.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

/// Title-case a dir/identifier: split on `-`/`_`/space, capitalize words.
pub(crate) fn title_case(s: &str) -> String {
    s.split(['-', '_', ' '])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut cs = w.chars();
            match cs.next() {
                Some(f) => f.to_ascii_uppercase().to_string() + cs.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use straymark_core::architecture::parse_model_str;

    #[test]
    fn kebab_and_title_case() {
        assert_eq!(kebab("straymark-core"), "straymark-core");
        assert_eq!(kebab("Web App!"), "web-app");
        assert_eq!(kebab("  experimento  "), "experimento");
        assert_eq!(title_case("ai-audit"), "Ai Audit");
        assert_eq!(title_case("cli"), "Cli");
    }

    #[test]
    fn seed_layers_has_unassigned_first_and_stages() {
        let layers = seed_layers();
        assert_eq!(layers[0].id, UNASSIGNED);
        assert_eq!(layers[0].order, 0);
        let ids: Vec<&str> = layers.iter().map(|l| l.id.as_str()).collect();
        assert!(ids.contains(&"01-requirements"));
        assert!(ids.contains(&"07-ai-audit"));
        let orders: Vec<u32> = layers.iter().map(|l| l.order).collect();
        let mut sorted = orders.clone();
        sorted.sort_unstable();
        assert_eq!(orders, sorted);
    }

    #[test]
    fn component_from_dir_shape() {
        let c = component_from_dir("cli");
        assert_eq!(c.id, "cli");
        assert_eq!(c.globs, vec!["cli/**"]);
        assert_eq!(c.layer, UNASSIGNED);
    }

    #[test]
    fn generated_model_passes_core_validation() {
        let model = ArchModel {
            version: 0,
            layers: seed_layers(),
            components: vec![Component {
                id: "cli".into(),
                label: "CLI".into(),
                layer: UNASSIGNED.into(),
                globs: vec!["cli/**".into()],
                links: vec!["core".into()],
                docs: vec![],
                external: false,
            }],
        };
        let yaml = render_model_yaml(&model);
        let parsed = parse_model_str(&yaml).expect("rendered YAML must parse");
        assert_eq!(parsed, model);
    }

    #[test]
    fn yaml_scalar_escapes_quotes() {
        assert_eq!(yaml_scalar(r#"a"b"#), "\"a\\\"b\"");
        assert_eq!(yaml_flow_list(&["x/**".into(), "y".into()]), "[\"x/**\", \"y\"]");
        assert_eq!(yaml_flow_list(&[]), "[]");
    }
}
