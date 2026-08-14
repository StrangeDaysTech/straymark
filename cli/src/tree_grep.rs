//! Shared grep/parse-tier tree walking — the mechanical layer behind
//! `analyze declared-vs-wired` and `followups verify --claims`.
//!
//! Deliberately not an AST: claim re-derivation stays at the grep tier so it
//! works cross-stack (design constraint 2 of #419). Precision comes from the
//! checks that consume these primitives, not from the walk itself.

use anyhow::{Context, Result};
use regex::Regex;
use std::collections::BTreeMap;
use std::path::Path;

/// Glob files relative to `base`, scan each for `re`, and return a map of
/// symbol name → first relative file path it appeared in. A `BTreeMap` keeps
/// the output deterministic (sorted) without an explicit sort.
pub fn collect_symbols(base: &Path, glob_pat: &str, re: &Regex) -> Result<BTreeMap<String, String>> {
    let pattern = format!("{}/{}", base.display(), glob_pat);
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    let entries =
        glob::glob(&pattern).with_context(|| format!("invalid glob pattern: `{glob_pat}`"))?;
    for entry in entries {
        let path = match entry {
            Ok(p) => p,
            Err(_) => continue,
        };
        if !path.is_file() {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue, // skip binary / unreadable files
        };
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .display()
            .to_string();
        for caps in re.captures_iter(&content) {
            if let Some(m) = caps.get(1) {
                out.entry(m.as_str().to_string()).or_insert_with(|| rel.clone());
            }
        }
    }
    Ok(out)
}

/// Directory names never worth scanning for code claims: VCS internals,
/// build output, vendored deps, and the governance tree itself (a symbol
/// mentioned in an AILOG is not a caller).
const EXCLUDED_DIRS: &[&str] = &[".git", "target", "node_modules", ".straymark"];

/// One readable text file in the tree: path relative to the walked base, plus
/// its content.
pub struct TextFile {
    pub rel_path: String,
    pub content: String,
}

/// Walk `base` for readable text files, skipping [`EXCLUDED_DIRS`] and
/// symlinks. Best-effort: unreadable or non-UTF-8 files are skipped.
pub fn read_text_tree(base: &Path) -> Vec<TextFile> {
    let mut out = Vec::new();
    walk(base, base, &mut out);
    out
}

fn walk(base: &Path, dir: &Path, out: &mut Vec<TextFile>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let file_type = match entry.file_type() {
            Ok(f) => f,
            Err(_) => continue,
        };
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            let name = entry.file_name();
            if EXCLUDED_DIRS.contains(&name.to_string_lossy().as_ref()) {
                continue;
            }
            walk(base, &path, out);
        } else if file_type.is_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let rel_path = path
                    .strip_prefix(base)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                out.push(TextFile { rel_path, content });
            }
        }
    }
}

/// Word-boundary occurrences of `symbol` across a walked tree: how many files
/// mention it at least once, and the total mention count. `foo` does not
/// match `foobar`; `foo_bar` matches `crate::foo_bar(...)` because `:` and
/// `(` are non-word chars.
pub fn symbol_occurrences(tree: &[TextFile], symbol: &str) -> (usize, usize) {
    let re = match Regex::new(&format!(r"\b{}\b", regex::escape(symbol))) {
        Ok(r) => r,
        Err(_) => return (0, 0),
    };
    let mut files = 0;
    let mut total = 0;
    for f in tree {
        let n = re.find_iter(&f.content).count();
        if n > 0 {
            files += 1;
            total += n;
        }
    }
    (files, total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_symbols_extracts_capture_group_1() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("a.rs"), "fn foo() {}\nfn bar() {}\n").unwrap();
        let re = Regex::new(r"fn (\w+)").unwrap();
        let syms = collect_symbols(tmp.path(), "*.rs", &re).unwrap();
        assert!(syms.contains_key("foo"));
        assert!(syms.contains_key("bar"));
        assert_eq!(syms["foo"], "a.rs");
    }

    #[test]
    fn collect_symbols_handles_nested_glob() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("client/src")).unwrap();
        std::fs::write(tmp.path().join("client/src/proxy.rs"), "fn complete_auth() {}\n").unwrap();
        let re = Regex::new(r"fn (\w+)").unwrap();
        let syms = collect_symbols(tmp.path(), "client/**/*.rs", &re).unwrap();
        assert!(syms.contains_key("complete_auth"));
    }

    #[test]
    fn read_text_tree_skips_excluded_dirs() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(tmp.path().join("main.rs"), "fn main() {}\n").unwrap();
        std::fs::create_dir_all(tmp.path().join("target")).unwrap();
        std::fs::write(tmp.path().join("target/out.rs"), "fn generated() {}\n").unwrap();
        std::fs::create_dir_all(tmp.path().join(".straymark")).unwrap();
        std::fs::write(tmp.path().join(".straymark/registry.md"), "FU-001\n").unwrap();
        let tree = read_text_tree(tmp.path());
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].rel_path, "main.rs");
    }

    #[test]
    fn symbol_occurrences_respects_word_boundaries() {
        let tmp = tempfile::TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("a.rs"),
            "fn foo() {}\nfn foobar() { foo(); }\n",
        )
        .unwrap();
        std::fs::write(tmp.path().join("b.rs"), "use a::foo;\n").unwrap();
        let tree = read_text_tree(tmp.path());
        let (files, total) = symbol_occurrences(&tree, "foo");
        assert_eq!(files, 2);
        assert_eq!(total, 3);
        let (files, _) = symbol_occurrences(&tree, "foobar");
        assert_eq!(files, 1);
    }
}
