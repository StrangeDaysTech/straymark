//! Low-level, dependency-free text scanners shared across the adapter, the
//! code-shape extractor, and provenance inference. No regex (matching the
//! `core` philosophy); char-boundary-safe over accented prose.

use std::collections::HashSet;

/// Scan `text` for all identifiers shaped `<prefix><body>` where `body` is made
/// of ASCII alphanumerics / hyphens and contains at least one digit. Returns
/// them de-duplicated, in first-seen order.
///
/// Matches both `FR-010` (Sentinel) and `FR1` (Loom) shapes, plus `PM-002`,
/// `AILOG-2026-04-24-006`, `ADR-…`, `AIDEC-…`.
pub(crate) fn scan_ids(text: &str, prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut idx = 0;
    while let Some(rel) = text[idx..].find(prefix) {
        let start = idx + rel;
        let after = start + prefix.len();
        let mut end = after;
        for c in text[after..].chars() {
            if c.is_ascii_alphanumeric() || c == '-' {
                end += c.len_utf8();
            } else {
                break;
            }
        }
        let id = &text[start..end];
        let body = &text[after..end];
        if !body.is_empty() && body.chars().any(|c| c.is_ascii_digit()) && seen.insert(id.to_string())
        {
            out.push(id.to_string());
        }
        idx = end.max(start + 1);
    }
    out
}

/// Scan `/api/...` endpoint references; trims trailing punctuation. Skips a
/// `/api/` glued to an identifier/alias/relative path (preceded by an
/// alphanumeric, `_`, `@`, or `.`) — those are module specifiers like
/// `import … from '@/api/types.gen'`, not HTTP endpoints.
pub(crate) fn scan_endpoints(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let mut idx = 0;
    while let Some(rel) = text[idx..].find("/api/") {
        let start = idx + rel;
        let is_module_path = text[..start]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '@' | '.'));
        if is_module_path {
            idx = start + 1;
            continue;
        }
        let mut end = start;
        for c in text[start..].chars() {
            if c.is_whitespace() || matches!(c, '`' | '"' | '\'' | '(' | ')' | '<' | '>' | '|') {
                break;
            }
            end += c.len_utf8();
        }
        let ep = text[start..end]
            .trim_end_matches(['.', ',', ';', ':'])
            .to_string();
        if seen.insert(ep.clone()) {
            out.push(ep);
        }
        idx = end.max(start + 1);
    }
    out
}

/// Normalize an endpoint path into a stable, cross-language `ContractId`.
///
/// Drops a leading HTTP method, any query string, the `/api/` prefix, a `vN`
/// version segment, and path parameters (`{id}` / `:id` / template `${id}`);
/// joins the rest with `.`. `GET /api/v1/services/{id}/health` → `services.health`.
/// Tolerant of call-site forms too: `` `/services/${id}/health?x=1` `` → `services.health`.
pub(crate) fn normalize_endpoint(ep: &str) -> String {
    // Last whitespace token drops a leading HTTP method; then strip a query string.
    let token = ep.split_whitespace().next_back().unwrap_or(ep).trim();
    let path = token.split('?').next().unwrap_or(token);
    let mut segs: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segs.first() == Some(&"api") {
        segs.remove(0);
    }
    if let Some(first) = segs.first() {
        if first.len() > 1 && first.starts_with('v') && first[1..].chars().all(|c| c.is_ascii_digit())
        {
            segs.remove(0);
        }
    }
    segs.into_iter()
        .filter(|s| !(s.starts_with('{') || s.starts_with(':') || s.starts_with('$')))
        .collect::<Vec<_>>()
        .join(".")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_ids_matches_both_fr_shapes() {
        assert_eq!(scan_ids("- **FR-010** estado", "FR"), vec!["FR-010"]);
        assert_eq!(scan_ids("- **FR1** — node", "FR"), vec!["FR1"]);
    }

    #[test]
    fn scan_ids_dedups_in_order() {
        let t = "FR-010 then FR-002 then FR-010 again";
        assert_eq!(scan_ids(t, "FR"), vec!["FR-010", "FR-002"]);
    }

    #[test]
    fn scan_ids_ignores_prefix_without_digits() {
        assert!(scan_ids("Frontend FRONT matters", "FR").is_empty());
    }

    #[test]
    fn scan_ids_is_char_boundary_safe() {
        assert_eq!(
            scan_ids("según AILOG-2026-04-24-006 está", "AILOG-"),
            vec!["AILOG-2026-04-24-006"]
        );
    }

    #[test]
    fn endpoints_are_scanned_and_trimmed() {
        let eps = scan_endpoints("call `GET /api/v1/services/{id}/health`. also /api/v1/services,");
        assert!(eps.contains(&"/api/v1/services/{id}/health".to_string()));
        assert!(eps.contains(&"/api/v1/services".to_string()));
    }

    #[test]
    fn normalize_strips_method_version_and_params() {
        assert_eq!(normalize_endpoint("GET /api/v1/services/{id}/health"), "services.health");
        assert_eq!(normalize_endpoint("/api/v2/users/:uid/profile"), "users.profile");
        assert_eq!(normalize_endpoint("/api/v1/services"), "services");
    }

    #[test]
    fn endpoints_skip_module_specifiers() {
        // `@/api/...` and relative `./api/...` imports are not HTTP endpoints.
        assert!(scan_endpoints("import type { HealthSnapshot } from '@/api/types.gen';").is_empty());
        assert!(scan_endpoints("import { api } from './api/client';").is_empty());
        // A real reference in prose/comments still scans.
        assert_eq!(scan_endpoints("// serves GET /api/v1/services"), vec!["/api/v1/services"]);
    }

    #[test]
    fn normalize_handles_call_site_forms() {
        // Relative path (the typed client adds the `/api/v1` base) + template param.
        assert_eq!(normalize_endpoint("/services/${serviceId}/health"), "services.health");
        // Query string is dropped.
        assert_eq!(normalize_endpoint("/audit/records?action_type=AI_AUTO"), "audit.records");
        assert_eq!(normalize_endpoint("/services"), "services");
    }
}
