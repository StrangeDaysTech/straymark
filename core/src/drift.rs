//! Pure drift matcher primitives: the wildcard set-difference at the heart of
//! `straymark charter drift`.
//!
//! Moved into `straymark-core` in Loom A1.0 (`002-architecture-plan`) so the
//! architecture "you are here" projection and the Loom server compute
//! declared-vs-modified the same way the CLI's `charter drift` command does
//! (one matcher, structurally no drift). These functions are pure (no IO, no
//! git): the CLI's `drift` command keeps its git-range diff, AILOG suppression,
//! and Batch-Ledger orchestration and calls [`compute_drift`] here.
//!
//! Glob semantics mirror the original bash `check-charter-drift.sh`
//! (`sed 's/\./\\./g; s/\*/.*/g'` + `^…$`): `*` matches any (possibly empty)
//! run of characters and spans `/`; every other character is literal.

/// Glob match where `*` matches any (possibly empty) run of characters and
/// every other character is literal, anchored over the whole string. Mirrors
/// the script's `sed 's/\./\\./g; s/\*/.*/g'` + `^…$` for path-like inputs.
pub fn glob_match(pattern: &str, text: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == text; // no wildcard → literal equality
    }
    if !text.starts_with(parts[0]) {
        return false;
    }
    let mut pos = parts[0].len();
    for (i, part) in parts.iter().enumerate().skip(1) {
        if i == parts.len() - 1 {
            // Last segment must end the string (and not overlap consumed prefix).
            return text.len() >= pos && text[pos..].ends_with(part);
        }
        if part.is_empty() {
            continue;
        }
        match text[pos..].find(part) {
            Some(idx) => pos += idx + part.len(),
            None => return false,
        }
    }
    true
}

/// True when a declared path's wildcard form is satisfied by `target`.
/// Ellipsis `prefix...suffix` → any path with that prefix; glob `prefix*suffix`
/// → [`glob_match`]. Returns `None` when `decl` is a literal path.
pub fn wildcard_satisfied_by(decl: &str, target: &str) -> Option<bool> {
    if let Some(idx) = decl.rfind("...") {
        Some(target.starts_with(&decl[..idx]))
    } else if decl.contains('*') {
        Some(glob_match(decl, target))
    } else {
        None
    }
}

/// Set-difference at Charter close: `(declared_omitted, modified_extra)`.
///
/// - **Omitted**: declared but not modified. A wildcard declaration is
///   satisfied by any modified path it matches; a literal needs an exact match.
/// - **Extra** (scope expansion): modified but not declared. Charter-doc and
///   AILOG paths are always in scope; a modified path also matched by any
///   declared wildcard is not extra.
pub fn compute_drift(declared: &[String], modified: &[String]) -> (Vec<String>, Vec<String>) {
    let omitted: Vec<String> = declared
        .iter()
        .filter(|decl| {
            !modified
                .iter()
                .any(|m| wildcard_satisfied_by(decl, m).unwrap_or_else(|| m == *decl))
        })
        .cloned()
        .collect();

    let extra: Vec<String> = modified
        .iter()
        .filter(|m| {
            if m.starts_with(".straymark/charters/") || m.starts_with(".straymark/07-ai-audit/") {
                return false;
            }
            if declared.iter().any(|d| d == *m) {
                return false;
            }
            // Allowed when a declared wildcard prefix/glob matches it.
            !declared
                .iter()
                .any(|decl| wildcard_satisfied_by(decl, m).unwrap_or(false))
        })
        .cloned()
        .collect();

    (omitted, extra)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(items: &[&str]) -> Vec<String> {
        items.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn glob_match_mirrors_script_semantics() {
        assert!(glob_match("AILOG-*.md", "AILOG-2026-06-12-001.md"));
        assert!(glob_match("src/*.rs", "src/main.rs"));
        assert!(glob_match("a*b*c", "axxbyyc"));
        assert!(glob_match("*.md", "README.md"));
        assert!(glob_match("dir/*", "dir/anything/deep.rs")); // `*` spans `/`, like `.*`
        assert!(!glob_match("AILOG-*.md", "AILOG-2026.txt"));
        assert!(!glob_match("src/*.rs", "src/main.go"));
        // `.` is literal, not a regex metachar.
        assert!(!glob_match("a.b", "axb"));
        assert!(glob_match("a.b", "a.b"));
        // No wildcard → exact equality.
        assert!(glob_match("src/main.rs", "src/main.rs"));
        assert!(!glob_match("src/main.rs", "src/main.rs.bak"));
    }

    #[test]
    fn compute_drift_literal_omission_and_scope_expansion() {
        let declared = s(&["src/a.rs", "src/b.rs", "src/c.rs"]);
        let modified = s(&["src/a.rs", "src/d.rs"]);
        let (omitted, extra) = compute_drift(&declared, &modified);
        assert_eq!(omitted, s(&["src/b.rs", "src/c.rs"])); // declared, not modified
        assert_eq!(extra, s(&["src/d.rs"])); // modified, not declared
    }

    #[test]
    fn compute_drift_wildcards_and_in_scope_paths() {
        // Ellipsis + glob declarations satisfied by a matching modified path;
        // charter-doc and AILOG paths are always in scope (never "extra").
        let declared = s(&[
            ".straymark/07-ai-audit/agent-logs/AILOG-...md",
            "src/gen/*.rs",
            "src/lit.rs",
        ]);
        let modified = s(&[
            ".straymark/07-ai-audit/agent-logs/AILOG-2026-06-12-001.md",
            ".straymark/charters/05-x.md",
            "src/gen/wire.rs",
            "src/lit.rs",
        ]);
        let (omitted, extra) = compute_drift(&declared, &modified);
        assert!(omitted.is_empty(), "all declared satisfied (2 wildcards + 1 literal)");
        assert!(extra.is_empty(), "charter/AILOG paths in scope; gen/wire matches glob");
    }
}
