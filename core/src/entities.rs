//! Non-document graph entities (R2): charters, plans, and audit reviews.
//!
//! StrayMark documents (`TYPE-YYYY-MM-DD-NNN-*.md`) are not the only things the
//! corpus cross-links to. Charters (`.straymark/charters/NN-slug.md`), plan
//! telemetry (`.straymark/plans/PLAN-NN.telemetry.yaml`) and audit reviews
//! (`.straymark/audits/<charter>/review.md`) are referenced from frontmatter
//! but are not `DocType`s and are not picked up by `discover_documents`. This
//! module discovers them as lightweight [`Entity`] values so the graph builder
//! can add them as nodes — turning references to them from dangling into real
//! edges (Loom *and* `straymark audit`, via the shared builder).

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::graph::EdgeType;

/// A discovered non-document node: a charter, plan, or audit review.
pub struct Entity {
    /// Stable node id (a `charter_id`, a `plan_id`, or an audit's relative path).
    pub id: String,
    /// Synthetic node type: `"CHARTER"`, `"PLAN"`, or `"AUDIT"`.
    pub doc_type: String,
    pub title: String,
    pub status: String,
    pub path: PathBuf,
    /// Outgoing edges this entity declares (e.g. a charter → its originating
    /// AILOGs, an audit → its charter). Targets are resolved like any edge.
    pub links: Vec<(EdgeType, String)>,
}

/// Discover all charter, plan and audit entities under a `.straymark/` dir.
/// Missing subdirectories are simply skipped (other adopters may have none).
pub fn discover_entities(straymark_dir: &Path) -> Vec<Entity> {
    let mut entities = Vec::new();
    discover_charters(straymark_dir, &mut entities);
    discover_audits(straymark_dir, &mut entities);
    discover_plans(straymark_dir, &mut entities);
    entities
}

/// Markdown files directly inside `<dir>`, sorted for determinism.
fn markdown_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("md"))
            .collect(),
        Err(_) => Vec::new(),
    };
    files.sort();
    files
}

#[derive(Deserialize)]
struct CharterFm {
    charter_id: Option<String>,
    status: Option<String>,
    originating_ailogs: Option<Vec<String>>,
    #[serde(default)]
    execution_ailogs: Option<Vec<String>>,
}

fn discover_charters(straymark_dir: &Path, out: &mut Vec<Entity>) {
    for path in markdown_files(&straymark_dir.join("charters")) {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let (fm, body) = split_frontmatter(&content);
        let Some(fm) = fm.and_then(|f| serde_yaml::from_str::<CharterFm>(f).ok()) else {
            continue;
        };
        // A charter without a charter_id can't be referenced by one — skip.
        let Some(id) = fm.charter_id.filter(|s| !s.is_empty()) else {
            continue;
        };
        let mut links = Vec::new();
        // Origin AILOGs and close-time execution AILOGs (#215 Gap 2) both connect
        // the Charter to its AILOGs in the knowledge graph.
        for ailog in fm
            .originating_ailogs
            .into_iter()
            .flatten()
            .chain(fm.execution_ailogs.into_iter().flatten())
        {
            links.push((EdgeType::OriginatesFrom, ailog));
        }
        out.push(Entity {
            title: first_heading(body).unwrap_or_else(|| id.clone()),
            id,
            doc_type: "CHARTER".into(),
            status: fm.status.unwrap_or_else(|| "unknown".into()),
            path,
            links,
        });
    }
}

#[derive(Deserialize)]
struct AuditFm {
    charter_id: Option<String>,
}

fn discover_audits(straymark_dir: &Path, out: &mut Vec<Entity>) {
    let audits_dir = straymark_dir.join("audits");
    let mut charter_dirs: Vec<PathBuf> = match std::fs::read_dir(&audits_dir) {
        Ok(rd) => rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect(),
        Err(_) => Vec::new(),
    };
    charter_dirs.sort();
    for dir in charter_dirs {
        let review = dir.join("review.md");
        let Ok(content) = std::fs::read_to_string(&review) else {
            continue;
        };
        let (fm, _) = split_frontmatter(&content);
        let charter_id = fm
            .and_then(|f| serde_yaml::from_str::<AuditFm>(f).ok())
            .and_then(|f| f.charter_id);
        // Stable id = path relative to the .straymark dir (e.g.
        // `audits/CHARTER-13/review.md`); `review.md` alone is ambiguous.
        let id = review
            .strip_prefix(straymark_dir)
            .unwrap_or(&review)
            .to_string_lossy()
            .replace('\\', "/");
        let mut links = Vec::new();
        if let Some(cid) = charter_id {
            links.push((EdgeType::RelatedTo, cid));
        }
        out.push(Entity {
            title: dir
                .file_name()
                .and_then(|s| s.to_str())
                .map(|c| format!("Audit — {c}"))
                .unwrap_or_else(|| id.clone()),
            id,
            doc_type: "AUDIT".into(),
            status: "reviewed".into(),
            path: review,
            links,
        });
    }
}

#[derive(Deserialize)]
struct PlanTelemetry {
    plan_telemetry: PlanFields,
}

#[derive(Deserialize)]
struct PlanFields {
    plan_id: Option<String>,
    plan_title: Option<String>,
    closed_at: Option<String>,
    /// PLAN telemetry nests its relations (vs. AILOG's flat
    /// `originating_ailogs: [ID]`): each entry is `{ ailog_id, … }`. Without
    /// descending into this, a telemetry file is an edge-less orphan node
    /// (issue #262 follow-up).
    #[serde(default)]
    originating_ailogs: Vec<OriginatingAilog>,
}

#[derive(Deserialize)]
struct OriginatingAilog {
    ailog_id: Option<String>,
}

fn discover_plans(straymark_dir: &Path, out: &mut Vec<Entity>) {
    let plans_dir = straymark_dir.join("plans");
    let mut files: Vec<PathBuf> = match std::fs::read_dir(&plans_dir) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|s| s.to_str())
                    .is_some_and(|n| n.ends_with(".telemetry.yaml"))
            })
            .collect(),
        Err(_) => Vec::new(),
    };
    files.sort();
    for path in files {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Some(t) = serde_yaml::from_str::<PlanTelemetry>(&content).ok() else {
            continue;
        };
        let fields = t.plan_telemetry;
        let Some(id) = fields.plan_id.filter(|s| !s.is_empty()) else {
            continue;
        };
        // Nested telemetry relations → outgoing edges (PLAN → its AILOGs).
        let mut links = Vec::new();
        for oa in fields.originating_ailogs {
            if let Some(aid) = oa.ailog_id.filter(|s| !s.is_empty()) {
                links.push((EdgeType::OriginatesFrom, aid));
            }
        }
        out.push(Entity {
            title: fields.plan_title.unwrap_or_else(|| id.clone()),
            status: if fields.closed_at.is_some() {
                "closed".into()
            } else {
                "open".into()
            },
            id,
            doc_type: "PLAN".into(),
            path,
            links,
        });
    }
}

/// Split a `---\n…\n---` YAML frontmatter block from the body. Returns
/// `(Some(frontmatter), body)` when a block is present, else `(None, content)`.
fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    let trimmed = content.trim_start_matches(['\u{feff}', '\n', '\r', ' ']);
    let after = match trimmed.strip_prefix("---\n").or_else(|| trimmed.strip_prefix("---\r\n")) {
        Some(a) => a,
        None => return (None, content),
    };
    // Closing delimiter at the start of a line.
    if let Some(idx) = after.find("\n---") {
        let fm = &after[..idx];
        let body = after[idx + 4..].trim_start_matches(['-', '\n', '\r']);
        (Some(fm), body)
    } else {
        (None, content)
    }
}

/// First Markdown `# ` heading in `body`, trimmed.
fn first_heading(body: &str) -> Option<String> {
    body.lines()
        .find_map(|line| line.strip_prefix("# ").map(|h| h.trim().to_string()))
        .filter(|h| !h.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn test_discover_charter_plan_audit() {
        let dir = tempfile::tempdir().unwrap();
        let sm = dir.path();
        write(
            &sm.join("charters/15-auth.md"),
            "---\ncharter_id: CHARTER-15-auth\nstatus: closed\noriginating_ailogs: [AILOG-2026-05-11-043]\n---\n# Charter: Auth hardening\nbody\n",
        );
        write(
            &sm.join("audits/CHARTER-13/review.md"),
            "---\ncharter_id: CHARTER-13-channels\n---\n# Review\n",
        );
        write(
            &sm.join("plans/PLAN-01.telemetry.yaml"),
            "plan_telemetry:\n  plan_id: \"PLAN-01\"\n  plan_title: \"Deploy & governance\"\n  closed_at: \"2026-04-27\"\n  originating_ailogs:\n    - ailog_id: \"AILOG-2026-04-25-015\"\n      still_relevant_at_execution: true\n",
        );

        let mut entities = discover_entities(sm);
        entities.sort_by(|a, b| a.id.cmp(&b.id));
        assert_eq!(entities.len(), 3);

        let charter = entities.iter().find(|e| e.doc_type == "CHARTER").unwrap();
        assert_eq!(charter.id, "CHARTER-15-auth");
        assert_eq!(charter.title, "Charter: Auth hardening");
        assert_eq!(charter.links, vec![(EdgeType::OriginatesFrom, "AILOG-2026-05-11-043".to_string())]);

        let audit = entities.iter().find(|e| e.doc_type == "AUDIT").unwrap();
        assert_eq!(audit.id, "audits/CHARTER-13/review.md");
        assert_eq!(audit.links, vec![(EdgeType::RelatedTo, "CHARTER-13-channels".to_string())]);

        let plan = entities.iter().find(|e| e.doc_type == "PLAN").unwrap();
        assert_eq!(plan.id, "PLAN-01");
        assert_eq!(plan.title, "Deploy & governance");
        assert_eq!(plan.status, "closed");
        // Issue #262 follow-up: the nested `originating_ailogs[].ailog_id`
        // becomes an outgoing edge, so the PLAN node isn't an orphan.
        assert_eq!(
            plan.links,
            vec![(EdgeType::OriginatesFrom, "AILOG-2026-04-25-015".to_string())]
        );
    }

    #[test]
    fn test_discover_entities_missing_dirs_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        assert!(discover_entities(dir.path()).is_empty());
    }
}
