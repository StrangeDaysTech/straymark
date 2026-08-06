// Generator: transforms dist/.claude/skills/*/SKILL.md into every channel that
// wants the minimal frontmatter (`name` + `description` only):
//
//   .codex/skills/   Codex CLI (OpenAI)
//   .agent/skills/   Antigravity CLI (`agy`) — a workspace customization root
//
// Usage:
//   cargo run --bin gen_minimal_skills [--check] [<dist-root>]
//
// Default <dist-root> is `../dist` relative to CWD (i.e. run from `cli/`).
// With --check, exits 1 if any destination tree differs from what would be
// generated. Used in CI to keep both channels in sync with `.claude/skills/`.
//
// Generating rather than hand-mirroring is the point: `.gemini/skills/`, the
// channel `.agent/skills/` replaces, had drifted behind `.claude/` in 7 of 15
// skills precisely because nothing regenerated or gated it.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    let mut check_only = false;
    let mut dist_arg: Option<String> = None;
    for arg in env::args().skip(1) {
        if arg == "--check" {
            check_only = true;
        } else if !arg.starts_with("--") && dist_arg.is_none() {
            dist_arg = Some(arg);
        } else {
            eprintln!("unknown argument: {arg}");
            process::exit(2);
        }
    }
    let dist = PathBuf::from(dist_arg.unwrap_or_else(|| "../dist".to_string()));
    if let Err(e) = run(&dist, check_only) {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

/// Channels generated from `.claude/skills/`: (label, root directory).
const TARGETS: &[(&str, &str)] = &[("Codex", ".codex"), ("Antigravity", ".agent")];

fn run(dist: &Path, check_only: bool) -> Result<(), String> {
    for (label, root) in TARGETS {
        run_target(dist, check_only, label, root)?;
    }
    Ok(())
}

fn run_target(dist: &Path, check_only: bool, label: &str, root: &str) -> Result<(), String> {
    let src = dist.join(".claude").join("skills");
    let dst = dist.join(root).join("skills");

    if !src.is_dir() {
        return Err(format!("source not found: {}", src.display()));
    }

    let mut produced: Vec<(PathBuf, String)> = Vec::new();
    let mut names: Vec<String> = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(&src)
        .map_err(|e| format!("read_dir {}: {e}", src.display()))?
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let content = fs::read_to_string(&skill_md)
            .map_err(|e| format!("read {}: {e}", skill_md.display()))?;
        let transformed = transform(&content);
        let out = dst.join(&name).join("SKILL.md");
        produced.push((out, transformed));
        names.push(name);
    }

    if check_only {
        let mut drift = Vec::new();
        for (out, expected) in &produced {
            let actual = fs::read_to_string(out).unwrap_or_default();
            if &actual != expected {
                drift.push(out.clone());
            }
        }
        // Also flag extra files in dst that aren't in produced.
        if dst.is_dir() {
            for entry in fs::read_dir(&dst).map_err(|e| e.to_string())?.flatten() {
                let p = entry.path();
                if !p.is_dir() {
                    continue;
                }
                let n = entry.file_name().to_string_lossy().into_owned();
                if !names.contains(&n) {
                    drift.push(p.join("SKILL.md"));
                }
            }
        }
        if drift.is_empty() {
            println!("{label} skills are in sync ({} skills).", produced.len());
            return Ok(());
        }
        eprintln!("{label} skills out of sync. Run: cargo run --bin gen_minimal_skills");
        for p in drift {
            eprintln!("  - {}", p.display());
        }
        process::exit(1);
    }

    // Clean dst and write fresh.
    if dst.exists() {
        fs::remove_dir_all(&dst)
            .map_err(|e| format!("remove_dir_all {}: {e}", dst.display()))?;
    }
    fs::create_dir_all(&dst).map_err(|e| format!("create_dir_all {}: {e}", dst.display()))?;
    for (out, content) in &produced {
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        }
        fs::write(out, content).map_err(|e| format!("write {}: {e}", out.display()))?;
    }
    println!("Generated {} {label} skills in {}", produced.len(), dst.display());
    for n in &names {
        println!("  - {n}");
    }
    Ok(())
}

/// Strip Claude-specific frontmatter keys, keeping only `name` and `description`.
fn transform(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return content.to_string();
    }
    let close_idx = match lines.iter().enumerate().skip(1).find(|(_, l)| l.trim() == "---") {
        Some((i, _)) => i,
        None => return content.to_string(),
    };
    let fm = &lines[1..close_idx];
    let body = &lines[close_idx + 1..];

    let mut out = String::new();
    out.push_str("---\n");
    let mut in_kept_value = false;
    for line in fm {
        let trimmed = line.trim_start();
        // Top-level key line (no leading whitespace, contains colon before any space).
        let is_top_level_key = line.len() == trimmed.len() && line.contains(':');
        if is_top_level_key {
            let key = trimmed.split(':').next().unwrap_or("");
            if key == "name" || key == "description" {
                out.push_str(line);
                out.push('\n');
                in_kept_value = true;
            } else {
                in_kept_value = false;
            }
        } else if in_kept_value {
            // Continuation of a kept multiline value (indented line).
            out.push_str(line);
            out.push('\n');
        }
        // Otherwise: continuation of a dropped key — skip.
    }
    out.push_str("---\n");
    for line in body {
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::transform;

    #[test]
    fn keeps_name_and_description_only() {
        let input = "---\nname: foo\ndescription: bar\nallowed-tools: Read, Write\nargument-hint: \"X\"\n---\n\n# Body\n";
        let out = transform(input);
        assert!(out.contains("name: foo"));
        assert!(out.contains("description: bar"));
        assert!(!out.contains("allowed-tools"));
        assert!(!out.contains("argument-hint"));
        assert!(out.contains("# Body"));
    }

    #[test]
    fn no_frontmatter_passthrough() {
        let input = "# No frontmatter\nbody\n";
        assert_eq!(transform(input), input);
    }

    #[test]
    fn drops_unknown_keys() {
        let input = "---\nname: a\nmodel: claude-opus\ndescription: b\n---\nbody\n";
        let out = transform(input);
        assert!(!out.contains("model:"));
        assert!(out.contains("name: a"));
        assert!(out.contains("description: b"));
    }
}
