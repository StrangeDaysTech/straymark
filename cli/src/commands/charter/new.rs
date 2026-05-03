//! `devtrail charter new` — scaffold a Charter from the framework template.
//!
//! Three origin paths:
//! - `--from-ailog AILOG-ID`: post-MVP / maintenance mode (the Sentinel case).
//! - `--from-spec specs/.../spec.md`: greenfield mode driven by SpecKit.
//! - neither: Charter scaffolded without an explicit origin (must be filled
//!   manually before status moves to `in-progress`).
//!
//! Mutual exclusion of `--from-ailog` and `--from-spec` is enforced by clap
//! at parse time.

use anyhow::{anyhow, bail, Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input};
use std::path::Path;

use crate::charter::next_charter_number;
use crate::config::DevTrailConfig;
use crate::utils;

/// Default effort when the user does not pass `--type`. M is the median bucket
/// observed across Sentinel PLAN-01..06 and a sensible neutral default.
const DEFAULT_EFFORT: &str = "M";

pub fn run(
    path: &str,
    effort_arg: Option<&str>,
    from_ailog: Option<&str>,
    from_spec: Option<&str>,
    title_arg: Option<&str>,
    slug_arg: Option<&str>,
) -> Result<()> {
    // clap enforces mutual exclusion via conflicts_with — keep this assertion
    // as a defense against direct programmatic invocation.
    if from_ailog.is_some() && from_spec.is_some() {
        bail!("--from-ailog and --from-spec are mutually exclusive");
    }

    let resolved = utils::resolve_project_root(path)
        .ok_or_else(|| anyhow!("DevTrail not installed. Run 'devtrail init' first."))?;
    let project_root = &resolved.path;
    let devtrail_dir = project_root.join(".devtrail");

    let resolved_language = DevTrailConfig::resolve_language(project_root);
    let lang = resolved_language.as_str();

    // Title (interactive fallback matches `devtrail new`'s UX).
    let title = match title_arg {
        Some(t) => t.to_string(),
        None => Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Charter title")
            .interact_text()?,
    };
    if title.trim().is_empty() {
        bail!("Title is required");
    }

    let effort = effort_arg.unwrap_or(DEFAULT_EFFORT);

    // Validate origin inputs (early rejection of obviously malformed values).
    if let Some(ailog_id) = from_ailog {
        validate_ailog_id(ailog_id)?;
    }
    if let Some(spec_path) = from_spec {
        validate_spec_path(project_root, spec_path)?;
    }

    // Resolve template (with i18n) and load.
    let templates_dir = devtrail_dir.join("templates");
    let template_path = utils::resolve_localized_path(&templates_dir, "charter-template.md", lang);
    let template = std::fs::read_to_string(&template_path).with_context(|| {
        format!(
            "Charter template not found at {}. Run `devtrail repair` to restore framework files.",
            template_path.display()
        )
    })?;

    // Build identifiers.
    // F1 (cli-3.7.2): `--slug` lets the operator override the title-derived
    // slug when the auto-derivation drops meaningful suffixes (e.g.
    // `…-plan-04-f3` → cut to `…-plan` because the limit hit). The override
    // is normalized through the same slugifier so it cannot smuggle in
    // characters that break the filename.
    let nn = next_charter_number(project_root);
    let slug = match slug_arg {
        Some(s) if !s.trim().is_empty() => slugify(s),
        _ => slugify(&title),
    };
    if slug.is_empty() {
        bail!(
            "{} produces an empty slug — must contain at least one alphanumeric character",
            if slug_arg.is_some() { "--slug" } else { "Title" }
        );
    }
    let charter_id = format!("CHARTER-{:02}-{}", nn, slug);
    let filename = format!("{:02}-{}.md", nn, slug);

    // Substitute placeholders. The template uses unique tokens for each
    // substitution so plain `String::replace` is safe and predictable.
    let content = apply_substitutions(
        &template,
        &charter_id,
        effort,
        &title,
        from_ailog,
        from_spec,
    );

    // Write to docs/charters/.
    let charters_dir = project_root.join("docs").join("charters");
    utils::ensure_dir(&charters_dir)?;
    let out_path = charters_dir.join(&filename);
    if out_path.exists() {
        bail!(
            "Charter file already exists: {} (next number computed as {:02} but a file with this slug exists)",
            out_path.display(),
            nn
        );
    }
    std::fs::write(&out_path, content)?;

    let rel_path = out_path
        .strip_prefix(project_root)
        .unwrap_or(&out_path)
        .display();

    println!();
    utils::success(&format!("Created: {}", rel_path));
    println!();
    println!("  {}", "Next steps:".bold());
    for line in next_steps(from_ailog, from_spec) {
        println!("    {}", line);
    }
    println!();

    Ok(())
}

/// Build the "Next steps" guidance shown after `charter new` succeeds. Pure
/// function — exposed for unit testing. Steps are numbered dynamically so
/// suppressing the conditional origin-step does not leave a numbering gap
/// (the bug fixed in cli-3.6.1, originally reported as F1 of
/// AILOG-2026-05-02-028 in Sentinel).
fn next_steps(from_ailog: Option<&str>, from_spec: Option<&str>) -> Vec<String> {
    let mut steps: Vec<&str> = vec![
        "Edit the Charter to fill in Context, Scope, Files to modify, Verification, Risks, Tasks.",
        "Set the trigger field in frontmatter to a concrete observable signal.",
    ];
    if from_ailog.is_none() && from_spec.is_none() {
        steps.push(
            "Set originating_ailogs or originating_spec in frontmatter (or leave both absent if standalone).",
        );
    }
    steps.push(
        "When you start executing: change frontmatter status from `declared` to `in-progress`.",
    );
    steps
        .into_iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {}", i + 1, s))
        .collect()
}

/// Apply all placeholder substitutions to the template body. Returns the
/// substituted content. Pure function — exposed for unit testing.
fn apply_substitutions(
    template: &str,
    charter_id: &str,
    effort: &str,
    title: &str,
    from_ailog: Option<&str>,
    from_spec: Option<&str>,
) -> String {
    let mut content = template.to_string();

    // charter_id placeholder (frontmatter).
    content = content.replace("charter_id: CHARTER-NN", &format!("charter_id: {}", charter_id));

    // effort_estimate (frontmatter — appears once).
    content = content.replace(
        "effort_estimate: M",
        &format!("effort_estimate: {}", effort),
    );

    // Title placeholders (EN and ES variants of the body H1).
    content = content.replace("# Charter: [BRIEF TITLE]", &format!("# Charter: {}", title));
    content = content.replace("# Charter: [TÍTULO BREVE]", &format!("# Charter: {}", title));

    // Prose effort mirror line (EN: "Effort:" / ES: "Esfuerzo:" with the same
    // bracketed enum). The "~[N] min" stays as a placeholder for the user to
    // fill in the actual time estimate.
    content = content.replace("[XS | S | M | L]", effort);

    // Origin: uncomment the chosen line and (optionally) tighten the prose summary.
    if let Some(ailog_id) = from_ailog {
        content = content.replace(
            "# originating_ailogs: [AILOG-YYYY-MM-DD-NNN]",
            &format!("originating_ailogs: [{}]", ailog_id),
        );
        // Replace the prose Origin placeholder with a concrete reference (EN).
        content = content.replace(
            "[human-readable summary; the machine-readable form is `originating_ailogs` or `originating_spec` in frontmatter]",
            &format!("Follow-up of {}. [Add 1-line context about why this Charter exists now.]", ailog_id),
        );
        // ES variant.
        content = content.replace(
            "[resumen humano; la forma machine-readable es `originating_ailogs` u `originating_spec` en el frontmatter]",
            &format!("Follow-up de {}. [Añadir 1 línea de contexto sobre por qué este Charter existe ahora.]", ailog_id),
        );
    } else if let Some(spec_path) = from_spec {
        content = content.replace(
            "# originating_spec: specs/001-feature/spec.md",
            &format!("originating_spec: {}", spec_path),
        );
        content = content.replace(
            "[human-readable summary; the machine-readable form is `originating_ailogs` or `originating_spec` in frontmatter]",
            &format!("Implementation derived from spec at {}. [Add 1-line context.]", spec_path),
        );
        content = content.replace(
            "[resumen humano; la forma machine-readable es `originating_ailogs` u `originating_spec` en el frontmatter]",
            &format!("Implementación derivada del spec en {}. [Añadir 1 línea de contexto.]", spec_path),
        );
    }
    // If neither: both `# originating_*` lines stay commented, the prose Origin
    // placeholder stays as-is for the user to fill in.

    content
}

/// Cheap syntactic check on the AILOG ID. Catches typos at scaffold time.
/// The schema's regex enforces the same shape on read-back.
fn validate_ailog_id(s: &str) -> Result<()> {
    if !s.starts_with("AILOG-") {
        bail!(
            "--from-ailog: expected an AILOG ID like AILOG-YYYY-MM-DD-NNN, got '{}'",
            s
        );
    }
    Ok(())
}

/// Verify the spec path exists relative to the project root. Catches typos and
/// the common confusion of passing a glob or a directory without spec.md.
fn validate_spec_path(project_root: &Path, spec_path: &str) -> Result<()> {
    let p = project_root.join(spec_path);
    if !p.exists() {
        bail!(
            "--from-spec: file does not exist at {} (relative to project root). \
             Pass the path to a SpecKit spec.md (e.g., specs/001-feature/spec.md).",
            p.display()
        );
    }
    Ok(())
}

/// Slugify a title for use in a Charter filename. Mirrors the implementation
/// in `commands::new::slugify` (kept private there — duplicated here to avoid
/// touching the existing command in this PR; consolidate to `utils` later).
///
/// F1 (cli-3.7.2): truncation now respects word boundaries. The previous
/// implementation cut at the 50-char limit and only trimmed a trailing `-`,
/// which produced mid-word slugs like `…-required-t` (truncating "true" to "t"
/// in Sentinel CHARTER-04). The fix is to back up to the last `-` boundary
/// at-or-before the limit, never producing a partial word fragment. Operators
/// who want a fully custom slug pass `--slug` to override this function entirely.
fn slugify(title: &str) -> String {
    let lower = title.to_lowercase();
    let parts: Vec<&str> = lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();
    let slug = parts.join("-");
    if slug.chars().count() > 50 {
        truncate_slug_at_word_boundary(&slug, 50)
    } else {
        slug
    }
}

/// Truncate `slug` at-or-before `max_chars`, never splitting a word.
///
/// The function works in two cases:
/// - If `slug[max_chars]` is `-` or end-of-string, then `slug[..max_chars]`
///   already ends on a complete word — we keep the full prefix (after
///   trimming any trailing `-`).
/// - Otherwise, `slug[..max_chars]` ends mid-word; we back up to the last
///   `-` boundary inside the truncated view and drop the partial token.
///
/// Falls back to a hard cut when the truncated view contains no hyphen at
/// all (single very long token).
fn truncate_slug_at_word_boundary(slug: &str, max_chars: usize) -> String {
    let truncated: String = slug.chars().take(max_chars).collect();

    let next_is_boundary = slug
        .chars()
        .nth(max_chars)
        .map(|c| c == '-')
        .unwrap_or(true);
    if next_is_boundary {
        return truncated.trim_end_matches('-').to_string();
    }

    let cut = match truncated.rfind('-') {
        Some(idx) => &truncated[..idx],
        None => truncated.as_str(),
    };
    cut.trim_end_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal template that covers all substitution points the runner touches.
    /// Mirrors the structure of dist/.devtrail/templates/charter-template.md
    /// without the full body.
    const TEMPLATE: &str = r#"---
charter_id: CHARTER-NN
status: declared
effort_estimate: M
trigger: "[1-line]"
# originating_ailogs: [AILOG-YYYY-MM-DD-NNN]
# originating_spec: specs/001-feature/spec.md
---

# Charter: [BRIEF TITLE]

> **Status (mirrored from frontmatter — source of truth is above):** declared. Effort: [XS | S | M | L] (~[N] min).
>
> **Origin:** [human-readable summary; the machine-readable form is `originating_ailogs` or `originating_spec` in frontmatter].

Body content.
"#;

    #[test]
    fn applies_all_basic_substitutions() {
        let out = apply_substitutions(
            TEMPLATE,
            "CHARTER-01-test-charter",
            "M",
            "Test Charter",
            None,
            None,
        );
        assert!(out.contains("charter_id: CHARTER-01-test-charter"));
        assert!(out.contains("# Charter: Test Charter"));
        assert!(out.contains("Effort: M (~[N] min)"));
        // Both origin lines remain commented out.
        assert!(out.contains("# originating_ailogs: [AILOG-YYYY-MM-DD-NNN]"));
        assert!(out.contains("# originating_spec: specs/001-feature/spec.md"));
    }

    #[test]
    fn from_ailog_uncomments_originating_ailogs() {
        let out = apply_substitutions(
            TEMPLATE,
            "CHARTER-01-x",
            "S",
            "X",
            Some("AILOG-2026-04-28-021"),
            None,
        );
        assert!(out.contains("originating_ailogs: [AILOG-2026-04-28-021]"));
        // The other origin stays commented as a placeholder.
        assert!(out.contains("# originating_spec: specs/001-feature/spec.md"));
        // Prose Origin line gets a concrete reference.
        assert!(out.contains("Follow-up of AILOG-2026-04-28-021"));
    }

    #[test]
    fn from_spec_uncomments_originating_spec() {
        let out = apply_substitutions(
            TEMPLATE,
            "CHARTER-02-x",
            "L",
            "X",
            None,
            Some("specs/001-test/spec.md"),
        );
        assert!(out.contains("originating_spec: specs/001-test/spec.md"));
        assert!(out.contains("# originating_ailogs: [AILOG-YYYY-MM-DD-NNN]"));
        assert!(out.contains("derived from spec at specs/001-test/spec.md"));
    }

    #[test]
    fn effort_substitution_handles_all_buckets() {
        for e in ["XS", "S", "M", "L"] {
            let out = apply_substitutions(TEMPLATE, "CHARTER-01-x", e, "X", None, None);
            assert!(out.contains(&format!("effort_estimate: {}", e)));
            assert!(out.contains(&format!("Effort: {} (~[N] min)", e)));
        }
    }

    #[test]
    fn validate_ailog_id_rejects_non_ailog_prefix() {
        assert!(validate_ailog_id("PLAN-05").is_err());
        assert!(validate_ailog_id("CHARTER-01").is_err());
        assert!(validate_ailog_id("").is_err());
    }

    #[test]
    fn validate_ailog_id_accepts_ailog_prefix() {
        assert!(validate_ailog_id("AILOG-2026-04-28-021").is_ok());
        // The CLI's syntactic check is intentionally loose — the schema
        // enforces the full pattern at validate time.
        assert!(validate_ailog_id("AILOG-anything").is_ok());
    }

    #[test]
    fn validate_spec_path_requires_existing_file() {
        let tmp = tempfile::TempDir::new().unwrap();
        let result = validate_spec_path(tmp.path(), "specs/001-missing/spec.md");
        assert!(result.is_err());

        let spec_dir = tmp.path().join("specs").join("001-test");
        std::fs::create_dir_all(&spec_dir).unwrap();
        std::fs::write(spec_dir.join("spec.md"), "# Spec").unwrap();
        assert!(validate_spec_path(tmp.path(), "specs/001-test/spec.md").is_ok());
    }

    #[test]
    fn slugify_matches_devtrail_new_pattern() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Per-service anomaly thresholds"), "per-service-anomaly-thresholds");
        assert_eq!(slugify("UPPER_case mixed!"), "upper-case-mixed");
    }

    #[test]
    fn slugify_truncates_long_titles_to_50_chars() {
        let long = "a".repeat(100);
        let s = slugify(&long);
        assert!(s.len() <= 50);
    }

    // ── F1 (cli-3.7.2): word-boundary truncation ─────────────────────────

    #[test]
    fn slugify_truncates_at_word_boundary_not_mid_word() {
        // CHARTER-04 reproduction case from issue #81: title that overflows
        // 50 chars by 1-2 chars used to produce a mid-word fragment like
        // "…required-t" (cutting "true" to "t"). Now the truncation backs up
        // to the last `-` boundary and drops the partial token entirely.
        let title = "Approve retroactivo bulk de docs review_required: true";
        let s = slugify(title);
        assert!(s.len() <= 50, "slug must fit limit, got {}: {s}", s.len());
        assert!(
            !s.ends_with("-t") && !s.ends_with("-tr") && !s.ends_with("-tru"),
            "slug must not end with a partial word fragment, got: {s}"
        );
        // Last completed word should be preserved.
        assert!(s.ends_with("required"), "got: {s}");
    }

    #[test]
    fn slugify_handles_no_hyphen_in_truncated_window() {
        // Single very long token: no hyphens to back up to. We hard-cut.
        let title = "supercalifragilisticexpialidocious".repeat(3);
        let s = slugify(&title);
        assert!(s.len() <= 50);
        assert!(!s.contains('-'));
    }

    #[test]
    fn slugify_strips_trailing_hyphens_after_word_boundary_cut() {
        // If the cut lands such that a trailing `-` survives, we trim it.
        let title = "abc-def-ghi-jkl-mno-pqr-stu-vwx-yz1-2345-6789-extra";
        let s = slugify(title);
        assert!(!s.ends_with('-'), "got: {s}");
    }

    #[test]
    fn truncate_slug_at_word_boundary_helper_is_pure() {
        // Direct unit test of the helper: cut at last `-` ≤ max_chars.
        assert_eq!(
            truncate_slug_at_word_boundary("foo-bar-baz-qux", 11),
            "foo-bar-baz"
        );
        assert_eq!(
            truncate_slug_at_word_boundary("foo-bar-baz-qux", 10),
            "foo-bar"
        );
        // No hyphen within window → hard cut.
        assert_eq!(truncate_slug_at_word_boundary("supercalifragilistic", 10), "supercalif");
    }

    #[test]
    fn next_steps_no_origin_has_4_sequential_numbered_lines() {
        let steps = next_steps(None, None);
        assert_eq!(steps.len(), 4);
        assert!(steps[0].starts_with("1. "));
        assert!(steps[1].starts_with("2. "));
        assert!(steps[2].starts_with("3. "));
        assert!(steps[3].starts_with("4. "));
    }

    #[test]
    fn next_steps_with_from_ailog_re_sequences_without_gap() {
        // Regression test for cli-3.6.0 F1: when --from-ailog is passed, the
        // origin-step is suppressed and the remaining steps must renumber to
        // 1/2/3, NOT skip from 2 to 4 leaving a gap.
        let steps = next_steps(Some("AILOG-2026-04-28-021"), None);
        assert_eq!(steps.len(), 3);
        assert!(steps[0].starts_with("1. "));
        assert!(steps[1].starts_with("2. "));
        assert!(steps[2].starts_with("3. "));
        // Verify step 2 is the trigger (not the suppressed origin step) and
        // step 3 is the in-progress one (not stuck at "4.").
        assert!(steps[1].contains("trigger"));
        assert!(steps[2].contains("in-progress"));
    }

    #[test]
    fn next_steps_with_from_spec_re_sequences_without_gap() {
        let steps = next_steps(None, Some("specs/001-test/spec.md"));
        assert_eq!(steps.len(), 3);
        assert!(steps[0].starts_with("1. "));
        assert!(steps[2].starts_with("3. "));
        assert!(steps[2].contains("in-progress"));
    }

    #[test]
    fn next_steps_no_step_starts_with_4_when_origin_is_set() {
        // Defensive: even if the steps grow, when an origin is set, no line
        // should ever emit a "4. " prefix (because the conditional step is
        // suppressed and renumbering applies). This guards against regressions
        // that re-introduce hardcoded numbers.
        for (ailog, spec) in [
            (Some("AILOG-2026-04-28-021"), None),
            (None, Some("specs/x/spec.md")),
        ] {
            let steps = next_steps(ailog, spec);
            assert!(
                !steps.iter().any(|s| s.starts_with("4. ")),
                "no step should be numbered 4 when origin is set; got {:?}",
                steps
            );
        }
    }
}
