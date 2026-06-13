use anyhow::{bail, Context, Result};
use chrono::Local;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::path::PathBuf;

use crate::config::StrayMarkConfig;
use straymark_core::document::DocType;
use crate::utils;

pub fn run(path: &str, doc_type_arg: Option<&str>, title_arg: Option<&str>) -> Result<()> {
    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow::anyhow!("StrayMark not installed. Run 'straymark init' first."))?;
    let target = resolved.path;
    let straymark_dir = target.join(".straymark");

    let config = StrayMarkConfig::load(&target).unwrap_or_default();
    let resolved_language = StrayMarkConfig::resolve_language(&target);
    let lang = resolved_language.as_str();
    let china = config.has_region("china");

    // Select document type
    let doc_type = match doc_type_arg {
        Some(t) => {
            let dt = DocType::from_str_loose(t).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown document type '{}'. Valid types: {}",
                    t,
                    available_doc_types(china)
                        .iter()
                        .map(|d| d.prefix().to_lowercase())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
            if dt.is_china_only() && !china {
                bail!(
                    "Document type '{}' requires `regional_scope: china` in .straymark/config.yml",
                    dt.prefix().to_lowercase()
                );
            }
            dt
        }
        None => select_type_interactive(china)?,
    };

    // Get title
    let title = match title_arg {
        Some(t) => t.to_string(),
        None => Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Title")
            .interact_text()?,
    };
    if title.trim().is_empty() {
        bail!("Title is required");
    }

    // Generate slug, date, sequence
    let slug = slugify(&title);
    let today = Local::now().format("%Y-%m-%d").to_string();
    let doc_dir = straymark_dir.join(doc_type.directory());
    let seq = next_sequence_number(&doc_dir, doc_type, &today);

    // Load and fill template
    let template_path = resolve_template_path(&straymark_dir, doc_type, lang);
    let template = std::fs::read_to_string(&template_path)
        .with_context(|| format!("Template not found: {}", template_path.display()))?;

    let id = format!("{}-{}-{}", doc_type.prefix(), today, seq);
    let content = template
        .replace("YYYY-MM-DD-NNN", &format!("{}-{}", today, seq))
        .replace("YYYY-MM-DD", &today)
        .replace("[Descriptive title of the action]", &title)
        .replace("[Título descriptivo de la acción]", &title)
        .replace("[Decision title]", &title)
        .replace("[Título de la decisión]", &title)
        .replace("[Architectural decision title]", &title)
        .replace("[Título de la decisión arquitectónica]", &title)
        .replace("[Assessment title]", &title)
        .replace("[Título de la evaluación]", &title)
        .replace("[Model/System name]", &title)
        .replace("[Nombre del modelo/sistema]", &title)
        .replace("[System name]", &title)
        .replace("[Nombre del sistema]", &title)
        .replace("[Assessment scope]", &title)
        .replace("[Alcance de la evaluación]", &title)
        .replace("[Title]", &title)
        .replace("[Título]", &title)
        .replace("[agent-name-v1.0]", "manual-user")
        .replace("[nombre-agente-v1.0]", "manual-user")
        .replace("id: AILOG-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: AIDEC-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: ADR-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: ETH-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: REQ-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: TES-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: INC-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: TDE-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: SEC-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: MCARD-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: SBOM-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: DPIA-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: PIPIA-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: CACFILE-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: TC260RA-YYYY-MM-DD-NNN", &format!("id: {}", id))
        .replace("id: AILABEL-YYYY-MM-DD-NNN", &format!("id: {}", id));

    // Write file
    let filename = format!("{}-{}-{}-{}.md", doc_type.prefix(), today, seq, slug);
    utils::ensure_dir(&doc_dir)?;
    let filepath = doc_dir.join(&filename);
    std::fs::write(&filepath, content)?;

    // Print result
    let rel_path = filepath
        .strip_prefix(&target)
        .unwrap_or(&filepath)
        .display();
    println!();
    utils::success(&format!("Created: {}", rel_path));
    println!();
    println!("  {}", "Next steps:".bold());
    println!("    1. Edit the document to fill in details");
    println!(
        "    2. Commit: {}",
        format!("git add {}", rel_path).dimmed()
    );
    println!();

    Ok(())
}

/// DocType variants exposed to the user, filtered by `regional_scope`.
fn available_doc_types(china: bool) -> Vec<DocType> {
    DocType::ALL
        .iter()
        .copied()
        .filter(|t| !t.is_china_only() || china)
        .collect()
}

fn select_type_interactive(china: bool) -> Result<DocType> {
    let types = available_doc_types(china);
    let items: Vec<String> = types
        .iter()
        .map(|t| format!("{:<8} — {}", t.prefix().to_lowercase(), t.display_name()))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Document type")
        .items(&items)
        .default(0)
        .interact()?;

    Ok(types[selection])
}

fn slugify(title: &str) -> String {
    let lower = title.to_lowercase();
    let parts: Vec<&str> = lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();
    let slug = parts.join("-");
    // `slug` is built exclusively from ASCII alphanumerics joined by '-',
    // so every char is 1 byte and byte-slicing the first 50 is safe. The
    // `chars().take(50)` form keeps us robust if the filter ever changes.
    if slug.chars().count() > 50 {
        let truncated: String = slug.chars().take(50).collect();
        truncated.trim_end_matches('-').to_string()
    } else {
        slug
    }
}

fn next_sequence_number(doc_dir: &std::path::Path, doc_type: DocType, today: &str) -> String {
    let prefix_pattern = format!("{}-{}-", doc_type.prefix(), today);
    let mut max_seq = 0u32;

    if doc_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(doc_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_str().unwrap_or("");
                if let Some(rest) = name.strip_prefix(&prefix_pattern) {
                    // Take the first 3 chars safely; they must all be ASCII
                    // digits for the sequence to be valid.
                    let head: String = rest.chars().take(3).collect();
                    if head.chars().count() == 3 {
                        if let Ok(n) = head.parse::<u32>() {
                            max_seq = max_seq.max(n);
                        }
                    }
                }
            }
        }
    }

    format!("{:03}", max_seq + 1)
}

fn resolve_template_path(
    straymark_dir: &std::path::Path,
    doc_type: DocType,
    lang: &str,
) -> PathBuf {
    let template_name = format!("TEMPLATE-{}.md", doc_type.prefix());
    let templates_dir = straymark_dir.join("templates");
    utils::resolve_localized_path(&templates_dir, &template_name, lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Fix: auth bug #123"), "fix-auth-bug-123");
        assert_eq!(slugify("  spaces  everywhere  "), "spaces-everywhere");
        assert_eq!(slugify("UPPER-case_mixed"), "upper-case-mixed");
    }

    #[test]
    fn test_slugify_truncates() {
        let long_title = "a".repeat(60);
        assert!(slugify(&long_title).len() <= 50);
    }

    #[test]
    fn test_next_sequence_empty_dir() {
        let dir = tempfile::TempDir::new().unwrap();
        assert_eq!(
            next_sequence_number(dir.path(), DocType::Ailog, "2026-04-01"),
            "001"
        );
    }

    #[test]
    fn test_next_sequence_increments() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::write(
            dir.path().join("AILOG-2026-04-01-001-first.md"),
            "test",
        )
        .unwrap();
        assert_eq!(
            next_sequence_number(dir.path(), DocType::Ailog, "2026-04-01"),
            "002"
        );
    }
}
