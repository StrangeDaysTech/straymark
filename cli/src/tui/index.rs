use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::document::DocFrontMatter;
use super::i18n_strings::t;
use crate::utils;

/// A group in the documentation hierarchy (e.g., "02-design")
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DocGroup {
    /// Directory name (e.g., "02-design")
    pub name: String,
    /// Display label (e.g., "Design")
    pub label: String,
    pub path: PathBuf,
    pub subgroups: Vec<DocSubgroup>,
    /// Files directly in this group (not in a subgroup)
    pub files: Vec<DocEntry>,
}

/// A subgroup within a group (e.g., "decisions" under "02-design")
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DocSubgroup {
    /// Directory name (e.g., "technical-debt")
    pub name: String,
    /// Display label (e.g., "Technical debt")
    pub label: String,
    pub path: PathBuf,
    /// Files directly in this subgroup
    pub files: Vec<DocEntry>,
    /// User-created subdirectories within this subgroup
    pub user_dirs: Vec<UserDir>,
}

/// A user-created subdirectory within a subgroup (e.g., "daemon" under "agent-logs")
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UserDir {
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<DocEntry>,
}

/// A documentation file entry
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DocEntry {
    pub filename: String,
    pub path: PathBuf,
    /// Display title (from frontmatter, H1, or humanized filename)
    pub title: String,
    pub id: String,
    /// Short type badge: "AI", "DC", "AD", "ET", "RQ", "TS", "IN", "TD"
    pub doc_type: String,
    pub tags: Vec<String>,
    pub created: String,
    pub has_frontmatter: bool,
}

/// Bidirectional relationship index
#[derive(Debug, Default)]
pub struct RelationIndex {
    /// doc_id -> list of doc_ids it references
    pub references: HashMap<String, Vec<String>>,
    /// doc_id -> list of doc_ids that reference it
    pub referenced_by: HashMap<String, Vec<String>>,
    /// doc_id -> file path
    pub id_to_path: HashMap<String, PathBuf>,
}

/// The full documentation index
pub struct DocIndex {
    pub groups: Vec<DocGroup>,
    pub relations: RelationIndex,
    pub total_docs: usize,
}

/// Subgroup definition: (dir_name, display_label)
type SubgroupDef = (&'static str, &'static str);

/// Known documentation group definitions: (dir_name, display_label, subgroups)
const GROUP_DEFS: &[(&str, &str, &[SubgroupDef])] = &[
    ("00-governance", "Governance", &[("exceptions", "Exceptions")]),
    ("01-requirements", "Requirements", &[]),
    ("02-design", "Design", &[("decisions", "Decisions")]),
    ("03-implementation", "Implementation", &[]),
    ("04-testing", "Testing", &[]),
    (
        "05-operations",
        "Operations",
        &[("incidents", "Incidents"), ("runbooks", "Runbooks")],
    ),
    (
        "06-evolution",
        "Evolution",
        &[("technical-debt", "Technical debt")],
    ),
    (
        "07-ai-audit",
        "AI Audit",
        &[
            ("agent-logs", "Agent logs"),
            ("decisions", "Decisions"),
            ("ethical-reviews", "Ethical reviews"),
        ],
    ),
    ("08-security", "Security", &[]),
    ("09-ai-models", "AI Models", &[]),
];

impl DocIndex {
    /// Build the index by scanning the .straymark directory.
    ///
    /// `language` selects the preferred locale (e.g. `"en"`, `"es"`, `"zh-CN"`).
    /// When non-`"en"`, framework files at group roots (e.g. governance docs)
    /// are transparently swapped for their `i18n/<lang>/<filename>` counterpart
    /// when one exists; otherwise the English original is used. User-authored
    /// content under subgroups (`decisions/`, `incidents/`, ...) is never
    /// localized.
    pub fn build(straymark_dir: &Path, language: &str) -> Self {
        let mut groups = Vec::new();
        let mut relations = RelationIndex::default();
        let mut total_docs = 0;

        for &(group_name, group_label, subgroup_defs) in GROUP_DEFS {
            let group_path = straymark_dir.join(group_name);
            let localized_group_label = t(group_label, language).to_string();
            if !group_path.exists() {
                groups.push(DocGroup {
                    name: group_name.to_string(),
                    label: localized_group_label,
                    path: group_path,
                    subgroups: Vec::new(),
                    files: Vec::new(),
                });
                continue;
            }

            // Scan files directly in the group dir. Group roots host framework
            // docs that ship with translations under `i18n/<lang>/`, so apply
            // localization here.
            let files = scan_md_files_flat(&group_path, Some(language), &mut relations);
            total_docs += files.len();

            // Scan subgroups and their user-created subdirectories
            let mut subgroups = Vec::new();
            for &(sg_name, sg_label) in subgroup_defs {
                let sg_path = group_path.join(sg_name);
                if sg_path.exists() {
                    // Subgroups hold adopter content, which has no localized
                    // sibling to swap to.
                    let sg_files = scan_md_files_flat(&sg_path, None, &mut relations);
                    total_docs += sg_files.len();

                    // Scan user-created subdirectories
                    let mut user_dirs = Vec::new();
                    if let Ok(entries) = std::fs::read_dir(&sg_path) {
                        let mut dirs: Vec<PathBuf> = entries
                            .flatten()
                            .map(|e| e.path())
                            .filter(|p| p.is_dir())
                            .collect();
                        dirs.sort();
                        for dir_path in dirs {
                            let dir_name = dir_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .to_string();
                            let dir_files = scan_md_files(&dir_path, &mut relations);
                            total_docs += dir_files.len();
                            user_dirs.push(UserDir {
                                name: dir_name,
                                path: dir_path,
                                files: dir_files,
                            });
                        }
                    }

                    subgroups.push(DocSubgroup {
                        name: sg_name.to_string(),
                        label: t(sg_label, language).to_string(),
                        path: sg_path,
                        files: sg_files,
                        user_dirs,
                    });
                } else {
                    subgroups.push(DocSubgroup {
                        name: sg_name.to_string(),
                        label: t(sg_label, language).to_string(),
                        path: sg_path,
                        files: Vec::new(),
                        user_dirs: Vec::new(),
                    });
                }
            }

            groups.push(DocGroup {
                name: group_name.to_string(),
                label: localized_group_label,
                path: group_path,
                subgroups,
                files,
            });
        }

        // Synthetic "Charters" group at the end. Charters live at
        // .straymark/charters/, alongside the rest of the framework state
        // (declarative .md and telemetry .yaml share the directory). We
        // append them as a 10th-style pseudo-group so the existing
        // NavSelection tree model handles them without modification. The
        // group is added only when at least one Charter exists — adopters
        // who don't use the pattern see no empty stub.
        let project_root = straymark_dir.parent().unwrap_or(straymark_dir);
        let charter_files = scan_charters(project_root, &mut relations);
        if !charter_files.is_empty() {
            total_docs += charter_files.len();
            groups.push(DocGroup {
                // Sentinel name with underscore prefix so it cannot collide
                // with a real GROUP_DEFS entry that always uses NN-name.
                name: "_charters".to_string(),
                label: t("Charters", language).to_string(),
                path: straymark_core::charter::charters_dir(project_root),
                subgroups: Vec::new(),
                files: charter_files,
            });
        }

        // Synthetic "Follow-ups" group (cli-3.19.0, ADR-2026-06-03-001).
        // The registry is one file, but its FU-NNN entries are the units the
        // operator navigates — so each entry becomes a DocEntry under a
        // per-bucket subgroup. Selecting any entry opens the registry file.
        // Added only when the registry exists; bucket names are canonical
        // English vocabulary (machine-parsed), so they are not localized.
        if let Some((registry_entry, bucket_subgroups, n_entries)) =
            scan_followups(project_root, &mut relations)
        {
            total_docs += 1 + n_entries;
            groups.push(DocGroup {
                name: "_followups".to_string(),
                label: t("Follow-ups", language).to_string(),
                path: crate::followups::registry_path(project_root),
                subgroups: bucket_subgroups,
                files: vec![registry_entry],
            });
        }

        Self {
            groups,
            relations,
            total_docs,
        }
    }

    /// Find the file path for a related link.
    /// Tries multiple resolution strategies:
    /// 1. Exact document ID match (e.g., "ADR-2025-06-15-001")
    /// 2. Filename match (e.g., "AGENT-RULES.md")
    /// 3. Path suffix match (e.g., "00-governance/AGENT-RULES.md")
    pub fn find_by_ref(&self, reference: &str) -> Option<PathBuf> {
        // 1. Try as document ID
        if let Some(path) = self.relations.id_to_path.get(reference) {
            return Some(path.clone());
        }

        // Normalize: strip leading ./ or ../ segments for matching
        let clean_ref = reference
            .trim_start_matches("../")
            .trim_start_matches("./");

        // Extract just the filename part
        let ref_filename = std::path::Path::new(clean_ref)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(clean_ref);

        // Search all entries across all groups
        let mut candidates: Vec<&PathBuf> = Vec::new();

        for group in &self.groups {
            for entry in &group.files {
                if entry_matches(&entry.filename, &entry.path, ref_filename, clean_ref) {
                    candidates.push(&entry.path);
                }
            }
            for sg in &group.subgroups {
                for entry in &sg.files {
                    if entry_matches(&entry.filename, &entry.path, ref_filename, clean_ref) {
                        candidates.push(&entry.path);
                    }
                }
                for ud in &sg.user_dirs {
                    for entry in &ud.files {
                        if entry_matches(&entry.filename, &entry.path, ref_filename, clean_ref) {
                            candidates.push(&entry.path);
                        }
                    }
                }
            }
        }

        // If exactly one match, return it. If multiple, prefer the one
        // whose path ends with the clean reference.
        match candidates.len() {
            0 => None,
            1 => Some(candidates[0].clone()),
            _ => {
                // Prefer path suffix match
                let suffix_match = candidates.iter().find(|p| {
                    p.to_str()
                        .map(|s| s.ends_with(clean_ref))
                        .unwrap_or(false)
                });
                Some(suffix_match.unwrap_or(&candidates[0]).to_path_buf())
            }
        }
    }
}

fn entry_matches(filename: &str, path: &Path, ref_filename: &str, clean_ref: &str) -> bool {
    // Exact filename match
    if filename == ref_filename {
        return true;
    }
    // Path suffix match (e.g., "00-governance/AGENT-RULES.md")
    if let Some(path_str) = path.to_str() {
        if path_str.ends_with(clean_ref) {
            return true;
        }
    }
    false
}

/// Scan only direct .md files in a directory (non-recursive, for group root dirs).
///
/// When `localize` is `Some(lang)` and a translation exists at
/// `dir/i18n/<lang>/<filename>`, the entry's path (and therefore the title /
/// frontmatter shown in the TUI) is taken from the translated file. Pass
/// `None` for directories whose contents are user-authored and have no
/// localized counterpart.
fn scan_md_files_flat(
    dir: &Path,
    localize: Option<&str>,
    relations: &mut RelationIndex,
) -> Vec<DocEntry> {
    let mut entries = Vec::new();

    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return entries;
    };

    let mut paths: Vec<PathBuf> = read_dir
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension().and_then(|e| e.to_str()) == Some("md")
                && !p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("TEMPLATE-") || n.starts_with('.'))
                    .unwrap_or(true)
        })
        .collect();

    paths.sort_by(|a, b| {
        let name_a = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let name_b = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
        name_a.cmp(name_b)
    });

    for path in paths {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Prefer a localized variant when this directory is a framework
        // root that ships with translations. Falls back to the English
        // original silently when no translation exists.
        let resolved_path = match localize {
            Some(lang) => utils::resolve_localized_path(dir, &filename, lang),
            None => path,
        };

        let meta = quick_scan_frontmatter(&resolved_path, relations);

        entries.push(DocEntry {
            filename,
            path: resolved_path,
            title: meta.title,
            id: meta.id,
            doc_type: meta.doc_type,
            tags: meta.tags,
            created: meta.created,
            has_frontmatter: meta.has_frontmatter,
        });
    }

    entries
}

/// Scan .md files recursively (for subgroups that may have nested subdirectories)
fn scan_md_files(dir: &Path, relations: &mut RelationIndex) -> Vec<DocEntry> {
    let mut entries = Vec::new();
    let mut paths = Vec::new();
    collect_md_files(dir, &mut paths);
    paths.sort_by(|a, b| {
        let name_a = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let name_b = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
        name_a.cmp(name_b)
    });

    fn collect_md_files(dir: &Path, paths: &mut Vec<PathBuf>) {
        let Ok(read_dir) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_md_files(&path, paths);
            } else if path.is_file()
                && path.extension().and_then(|e| e.to_str()) == Some("md")
                && !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("TEMPLATE-") || n.starts_with('.'))
                    .unwrap_or(true)
            {
                paths.push(path);
            }
        }
    }

    for path in paths {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        // Quick frontmatter scan (just read enough to get id/title/tags/created/related)
        let meta = quick_scan_frontmatter(&path, relations);

        entries.push(DocEntry {
            filename,
            path,
            title: meta.title,
            id: meta.id,
            doc_type: meta.doc_type,
            tags: meta.tags,
            created: meta.created,
            has_frontmatter: meta.has_frontmatter,
        });
    }

    entries
}

struct ScannedMeta {
    title: String,
    id: String,
    doc_type: String,
    tags: Vec<String>,
    created: String,
    has_frontmatter: bool,
}

/// Extract doc type badge from filename prefix
fn doc_type_badge(filename: &str) -> String {
    let badges: &[(&str, &str)] = &[
        ("AILOG-", "AI"),
        ("AIDEC-", "DC"),
        ("ADR-", "AD"),
        ("ETH-", "ET"),
        ("REQ-", "RQ"),
        ("TES-", "TS"),
        ("INC-", "IN"),
        ("TDE-", "TD"),
        ("SEC-", "SC"),
        ("MCARD-", "MC"),
        ("SBOM-", "SB"),
        ("DPIA-", "DP"),
    ];
    for &(prefix, badge) in badges {
        if filename.starts_with(prefix) {
            return badge.to_string();
        }
    }
    String::new()
}

/// Try to find the first H1 title (# Title) in markdown content
fn find_h1_title(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(title) = trimmed.strip_prefix("# ") {
            let title = title.trim();
            if !title.is_empty() {
                return Some(title.to_string());
            }
        }
    }
    None
}

/// Convert a filename stem to a human-readable title
fn humanize_filename(stem: &str) -> String {
    stem.replace('-', " ").replace('_', " ")
}

fn fallback_meta(path: &Path, content: Option<&str>) -> ScannedMeta {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown");
    let stem = path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown");

    // Try H1 from content, then humanize filename
    let title = content
        .and_then(find_h1_title)
        .unwrap_or_else(|| humanize_filename(stem));

    ScannedMeta {
        title,
        id: String::new(),
        doc_type: doc_type_badge(filename),
        tags: Vec::new(),
        created: String::new(),
        has_frontmatter: false,
    }
}

/// Build TUI-compatible `DocEntry` records for all Charters in a project.
/// Reuses `straymark_core::charter` for discovery/parsing so the schema and the TUI
/// stay aligned on what "a Charter" means. Each entry carries the
/// `charter_id` as its `id` so hyperlinks via `find_by_ref` resolve, and a
/// "CH" badge so the nav tree distinguishes Charters from governance docs.
fn scan_charters(project_root: &Path, relations: &mut RelationIndex) -> Vec<DocEntry> {
    let paths = straymark_core::charter::discover_charters(project_root);
    let mut entries = Vec::with_capacity(paths.len());
    for path in paths {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let entry = match straymark_core::charter::parse_charter(&path) {
            Ok(charter) => {
                let title = straymark_core::charter::display_title(&charter);
                let id = charter.frontmatter.charter_id.clone();
                if !id.is_empty() {
                    relations
                        .id_to_path
                        .insert(id.clone(), path.to_path_buf());
                }
                DocEntry {
                    filename,
                    path: path.clone(),
                    title,
                    id,
                    doc_type: "CH".to_string(),
                    tags: Vec::new(),
                    created: String::new(),
                    has_frontmatter: true,
                }
            }
            Err(_) => {
                // Charter has malformed frontmatter — show it anyway as a
                // degraded entry so the user can find and fix it from the TUI.
                let stem = path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                DocEntry {
                    filename,
                    path: path.clone(),
                    title: humanize_filename(&stem),
                    id: String::new(),
                    doc_type: "CH".to_string(),
                    tags: Vec::new(),
                    created: String::new(),
                    has_frontmatter: false,
                }
            }
        };
        entries.push(entry);
    }
    entries
}

/// Build the synthetic Follow-ups group content: the registry file itself as
/// a root entry plus one `DocSubgroup` per non-empty bucket, each holding one
/// `DocEntry` per FU. Every entry's path points at the registry file (entries
/// are blocks, not files). Returns `None` when the registry does not exist or
/// cannot be parsed — the TUI shows no empty stub, mirroring `_charters`.
fn scan_followups(
    project_root: &Path,
    relations: &mut RelationIndex,
) -> Option<(DocEntry, Vec<DocSubgroup>, usize)> {
    let registry_path = crate::followups::registry_path(project_root);
    if !registry_path.exists() {
        return None;
    }
    let registry = crate::followups::parse_registry(&registry_path).ok()?;

    let registry_entry = DocEntry {
        filename: "follow-ups-backlog.md".to_string(),
        path: registry_path.clone(),
        title: "Follow-ups Backlog (registry)".to_string(),
        id: String::new(),
        doc_type: "FU".to_string(),
        tags: Vec::new(),
        created: registry.frontmatter.last_scan.clone().unwrap_or_default(),
        has_frontmatter: true,
    };

    let mut subgroups = Vec::new();
    let mut n_entries = 0usize;
    for section in registry.sections.iter().filter(|s| s.is_bucket) {
        if section.entries.is_empty() {
            continue;
        }
        let files: Vec<DocEntry> = section
            .entries
            .iter()
            .map(|e| {
                relations
                    .id_to_path
                    .insert(e.fu_id.clone(), registry_path.clone());
                DocEntry {
                    filename: "follow-ups-backlog.md".to_string(),
                    path: registry_path.clone(),
                    title: format!("{} — {}", e.fu_id, e.description),
                    id: e.fu_id.clone(),
                    doc_type: "FU".to_string(),
                    tags: e.labels.clone(),
                    created: String::new(),
                    has_frontmatter: true,
                }
            })
            .collect();
        n_entries += files.len();
        subgroups.push(DocSubgroup {
            name: section.name.clone(),
            label: section.name.clone(),
            path: registry_path.clone(),
            files,
            user_dirs: Vec::new(),
        });
    }

    Some((registry_entry, subgroups, n_entries))
}

fn quick_scan_frontmatter(path: &Path, relations: &mut RelationIndex) -> ScannedMeta {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return fallback_meta(path, None),
    };

    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return fallback_meta(path, Some(&content));
    }

    let after = &trimmed[3..];
    let Some(end) = after.find("\n---") else {
        return fallback_meta(path, Some(&content));
    };

    let yaml_str = &after[..end];
    let body = &after[end + 4..]; // content after closing ---
    let fm: Option<DocFrontMatter> = serde_yaml::from_str(yaml_str).ok();

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    match fm {
        Some(fm) => {
            let id = fm.id.clone();
            let title = if !fm.title.is_empty() {
                fm.title.clone()
            } else {
                // Try H1 from body, then humanize filename
                find_h1_title(body).unwrap_or_else(|| {
                    humanize_filename(
                        path.file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown"),
                    )
                })
            };
            let tags = fm.tags.clone();
            let created = fm.created.clone().unwrap_or_default();

            // Index relationships
            if !id.is_empty() {
                relations.id_to_path.insert(id.clone(), path.to_path_buf());

                if !fm.related.is_empty() {
                    for related_id in &fm.related {
                        relations
                            .referenced_by
                            .entry(related_id.clone())
                            .or_default()
                            .push(id.clone());
                    }
                    relations
                        .references
                        .entry(id.clone())
                        .or_default()
                        .extend(fm.related.iter().cloned());
                }
            }

            ScannedMeta {
                title,
                id,
                doc_type: doc_type_badge(filename),
                tags,
                created,
                has_frontmatter: true,
            }
        }
        None => fallback_meta(path, Some(body)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a fixture .straymark tree with a translated and an English-only
    /// governance doc, and verify DocIndex prefers the translation when
    /// language=zh-CN, falling back to English silently when no translation
    /// exists.
    #[test]
    fn build_zh_cn_swaps_governance_path_when_translation_present() {
        let tmp = tempfile::TempDir::new().unwrap();
        let straymark_dir = tmp.path().join(".straymark");
        let governance = straymark_dir.join("00-governance");
        let zh = governance.join("i18n").join("zh-CN");
        std::fs::create_dir_all(&zh).unwrap();

        // Translated doc (governance).
        std::fs::write(
            governance.join("QUICK-REFERENCE.md"),
            "# Quick Reference\n\nEnglish body.\n",
        )
        .unwrap();
        std::fs::write(
            zh.join("QUICK-REFERENCE.md"),
            "# 快速参考\n\n中文正文。\n",
        )
        .unwrap();

        // English-only doc (no zh-CN sibling).
        std::fs::write(
            governance.join("ISO-25010-2023-REFERENCE.md"),
            "# ISO 25010\n\nEnglish only.\n",
        )
        .unwrap();

        let index = DocIndex::build(&straymark_dir, "zh-CN");

        let governance_group = index
            .groups
            .iter()
            .find(|g| g.name == "00-governance")
            .expect("governance group present");

        let quick_ref = governance_group
            .files
            .iter()
            .find(|e| e.filename == "QUICK-REFERENCE.md")
            .expect("QUICK-REFERENCE indexed");
        assert_eq!(
            quick_ref.path,
            zh.join("QUICK-REFERENCE.md"),
            "zh-CN translation should be preferred"
        );
        assert_eq!(quick_ref.title, "快速参考");

        let iso = governance_group
            .files
            .iter()
            .find(|e| e.filename == "ISO-25010-2023-REFERENCE.md")
            .expect("ISO doc indexed");
        assert_eq!(
            iso.path,
            governance.join("ISO-25010-2023-REFERENCE.md"),
            "missing translation must fall back to English silently"
        );
        assert_eq!(iso.title, "ISO 25010");
    }

    #[test]
    fn build_en_never_descends_into_i18n_subdirs() {
        let tmp = tempfile::TempDir::new().unwrap();
        let straymark_dir = tmp.path().join(".straymark");
        let governance = straymark_dir.join("00-governance");
        let zh = governance.join("i18n").join("zh-CN");
        std::fs::create_dir_all(&zh).unwrap();

        std::fs::write(governance.join("AGENT-RULES.md"), "# Rules").unwrap();
        std::fs::write(zh.join("AGENT-RULES.md"), "# 规则").unwrap();

        let index = DocIndex::build(&straymark_dir, "en");
        let governance_group = index
            .groups
            .iter()
            .find(|g| g.name == "00-governance")
            .unwrap();

        // Exactly one entry: the English root file. The translation must
        // never appear as a separate doc (no duplication of total_docs).
        assert_eq!(governance_group.files.len(), 1);
        assert_eq!(
            governance_group.files[0].path,
            governance.join("AGENT-RULES.md")
        );
    }

    #[test]
    fn build_appends_charters_synthetic_group_when_present() {
        let tmp = tempfile::TempDir::new().unwrap();
        let project_root = tmp.path();
        let straymark_dir = project_root.join(".straymark");
        std::fs::create_dir_all(&straymark_dir).unwrap();
        let charters_dir = project_root.join(".straymark").join("charters");
        std::fs::create_dir_all(&charters_dir).unwrap();
        std::fs::write(
            charters_dir.join("01-real.md"),
            "---\n\
             charter_id: CHARTER-01-real\n\
             status: declared\n\
             effort_estimate: M\n\
             trigger: \"x\"\n\
             ---\n\n\
             # Charter: Real Title\n\nbody\n",
        )
        .unwrap();
        std::fs::write(
            charters_dir.join("02-broken.md"),
            "no frontmatter at all\n",
        )
        .unwrap();

        let index = DocIndex::build(&straymark_dir, "en");
        // The synthetic group is appended after the GROUP_DEFS entries.
        let charters_group = index
            .groups
            .iter()
            .find(|g| g.name == "_charters")
            .expect("synthetic charters group present");
        assert_eq!(charters_group.label, "Charters");
        assert_eq!(charters_group.files.len(), 2);

        let real = charters_group
            .files
            .iter()
            .find(|e| e.filename == "01-real.md")
            .unwrap();
        assert_eq!(real.id, "CHARTER-01-real");
        assert_eq!(real.title, "Real Title");
        assert_eq!(real.doc_type, "CH");

        // Charters with broken frontmatter still appear (degraded entry) so
        // the user can find and fix them from the TUI.
        let broken = charters_group
            .files
            .iter()
            .find(|e| e.filename == "02-broken.md")
            .unwrap();
        assert_eq!(broken.doc_type, "CH");
        assert!(!broken.has_frontmatter);

        // Charter id is indexed in relations so find_by_ref resolves it.
        assert_eq!(
            index.relations.id_to_path.get("CHARTER-01-real"),
            Some(&charters_dir.join("01-real.md"))
        );
    }

    #[test]
    fn build_skips_charters_group_when_no_charters_exist() {
        let tmp = tempfile::TempDir::new().unwrap();
        let straymark_dir = tmp.path().join(".straymark");
        std::fs::create_dir_all(&straymark_dir).unwrap();
        // No .straymark/charters/ directory.

        let index = DocIndex::build(&straymark_dir, "en");
        assert!(
            index.groups.iter().all(|g| g.name != "_charters"),
            "no synthetic Charters group should be present when none exist"
        );
    }

    #[test]
    fn build_charters_label_localizes_to_zh_cn() {
        let tmp = tempfile::TempDir::new().unwrap();
        let project_root = tmp.path();
        let straymark_dir = project_root.join(".straymark");
        std::fs::create_dir_all(&straymark_dir).unwrap();
        let charters_dir = project_root.join(".straymark").join("charters");
        std::fs::create_dir_all(&charters_dir).unwrap();
        std::fs::write(
            charters_dir.join("01-x.md"),
            "---\ncharter_id: CHARTER-01-x\nstatus: declared\neffort_estimate: M\ntrigger: x\n---\n\n# Charter: X\n",
        )
        .unwrap();

        let index = DocIndex::build(&straymark_dir, "zh-CN");
        let charters_group = index
            .groups
            .iter()
            .find(|g| g.name == "_charters")
            .expect("synthetic charters group present");
        assert_eq!(charters_group.label, "章程");
    }

    #[test]
    fn build_does_not_localize_user_subgroups() {
        let tmp = tempfile::TempDir::new().unwrap();
        let straymark_dir = tmp.path().join(".straymark");
        let decisions = straymark_dir.join("02-design").join("decisions");
        let stray_zh = decisions.join("i18n").join("zh-CN");
        std::fs::create_dir_all(&stray_zh).unwrap();

        std::fs::write(
            decisions.join("ADR-2026-01-01-001-foo.md"),
            "# English ADR",
        )
        .unwrap();
        // A translation file under a user subgroup must be ignored — adopter
        // content has no canonical English<->zh mapping.
        std::fs::write(stray_zh.join("ADR-2026-01-01-001-foo.md"), "# 中文 ADR")
            .unwrap();

        let index = DocIndex::build(&straymark_dir, "zh-CN");
        let design = index
            .groups
            .iter()
            .find(|g| g.name == "02-design")
            .unwrap();
        let decisions_sg = design
            .subgroups
            .iter()
            .find(|s| s.name == "decisions")
            .unwrap();

        let adr = decisions_sg
            .files
            .iter()
            .find(|e| e.filename == "ADR-2026-01-01-001-foo.md")
            .expect("ADR indexed");
        assert_eq!(
            adr.path,
            decisions.join("ADR-2026-01-01-001-foo.md"),
            "user-authored docs must not be swapped for any i18n sibling"
        );
    }
}
