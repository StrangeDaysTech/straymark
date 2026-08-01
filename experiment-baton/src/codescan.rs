//! Heuristic contract-shape extraction from code (Go producers, TS consumers).
//!
//! Not a real parser — a tolerant, line-oriented heuristic good enough to key a
//! contract's *shape* (field names, enum variants) to a `ContractId`. Two keying
//! sources, in priority order:
//!
//! 1. **Call-site binding** (#313): a usage-grounded `api.get<HealthSnapshot>(
//!    `/services/${id}/health`)` binds the *type* to the *route*. This is what
//!    keys a generated types file (`types.gen.ts`) whose declarations carry no
//!    endpoint anchor of their own — they'd otherwise all collapse onto one
//!    coarse contract (or be dropped entirely).
//! 2. **Producer-side route keying** (#319): a huma route-registration call
//!    (`huma.Get(api, "/path", h.handlerMethod)`) binds the handler symbol to
//!    the route; the handler's output struct (`<handler>Output`) then keys to
//!    that route instead of the nearest anchor. Fixes the huma pattern where
//!    all registrations sit in one block and the struct is defined ~75 lines
//!    below, so nearest-anchor binds it to the wrong route.
//! 3. **Nearest endpoint anchor**: the `/api/...` reference at or above a
//!    declaration. Used when no binding keys the type.
//!
//! Conservative by design (R3/R6): a declaration with neither a binding nor an
//! anchor yields nothing rather than a guess; a type bound to two different
//! routes is dropped as ambiguous.
//!
//! Cross-language keying (spec Q4): the join key is the **normalized endpoint**,
//! the one anchor a Go handler and a TS type provably share.

use std::collections::BTreeMap;
use std::path::Path;

use crate::intent::{ContractShape, EnumDef, Field, Lang, ShapeRole, SourceRef};
use crate::scan::{normalize_endpoint, scan_endpoints};

const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "dist",
    "build",
    ".specify",
    "specs",
    ".docusaurus",
];

/// Scan all Go/TS source under `root` for contract shapes.
pub fn scan(root: &Path) -> Vec<ContractShape> {
    // Read every code file once; keep (rel, lang, content).
    let mut files: Vec<(String, Lang, String)> = Vec::new();
    for path in walk_code(root) {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let lang = match ext(&path) {
            Some("go") => Lang::Go,
            Some("ts") | Some("tsx") => Lang::Ts,
            _ => continue,
        };
        files.push((rel, lang, content));
    }

    // Pass 1: repo-wide type→endpoint bindings mined from TS call sites — what
    // keys an anchorless generated types file (#313).
    let bindings = collect_bindings(&files);

    // Pass 1b: Go producer-side route registrations → handler→route map (#319).
    let go_bindings = collect_go_bindings(&files);

    // Pass 2: extract shapes, binding-first keying.
    let mut out = Vec::new();
    for (rel, lang, content) in &files {
        out.extend(extract_file(rel, content, *lang, &bindings, &go_bindings));
    }
    out.sort_by(|a, b| (&a.contract, &a.source.file).cmp(&(&b.contract, &b.source.file)));
    out
}

/// Mine `IDENT<Type>('/route')` call sites across TS files into a
/// `Type → ContractId` map (the binding keying source for #313). Conservative
/// (R6): a type bound to two *different* contracts is dropped as ambiguous.
fn collect_bindings(files: &[(String, Lang, String)]) -> BTreeMap<String, String> {
    let mut seen: BTreeMap<String, Option<String>> = BTreeMap::new();
    for (_, lang, content) in files {
        if *lang != Lang::Ts {
            continue;
        }
        for line in content.lines() {
            for (ty, cid) in scan_type_route_bindings(line) {
                match seen.get(&ty) {
                    None => {
                        seen.insert(ty, Some(cid));
                    }
                    // already bound to a different route → ambiguous, drop it
                    Some(Some(prev)) if *prev != cid => {
                        seen.insert(ty, None);
                    }
                    _ => {}
                }
            }
        }
    }
    seen.into_iter().filter_map(|(k, v)| v.map(|c| (k, c))).collect()
}

/// Mine huma route-registration calls across Go files into a
/// `TypeName → ContractId` map (#319). Parses `huma.Get(api, "/path", h.handlerMethod)`
/// (and Post/Put/Patch/Delete) to extract `(handlerMethod, route)`, then keys
/// `<handlerMethod>Output` → route via the huma naming convention.
/// Conservative (R6): only binds when the convention resolves unambiguously.
fn collect_go_bindings(files: &[(String, Lang, String)]) -> BTreeMap<String, String> {
    let mut seen: BTreeMap<String, Option<String>> = BTreeMap::new();
    for (_, lang, content) in files {
        if *lang != Lang::Go {
            continue;
        }
        for line in content.lines() {
            if let Some((handler, cid)) = parse_huma_registration(line) {
                // Key both the handler itself and the `<handler>Output` struct.
                for key in [handler.clone(), format!("{handler}Output")] {
                    match seen.get(&key) {
                        None => {
                            seen.insert(key, Some(cid.clone()));
                        }
                        Some(Some(prev)) if *prev != cid => {
                            seen.insert(key, None);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    seen.into_iter().filter_map(|(k, v)| v.map(|c| (k, c))).collect()
}

/// Parse a huma route registration line into `(handler_name, ContractId)`.
/// Recognizes: `huma.Get(api, "/path", h.handlerMethod)` and the other HTTP
/// method variants (Post, Put, Patch, Delete, Head, Options).
fn parse_huma_registration(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    // Must contain a huma method call with a route literal.
    let methods = ["huma.Get(", "huma.Post(", "huma.Put(", "huma.Patch(", "huma.Delete(", "huma.Head(", "huma.Options("];
    let rest = methods.iter().find_map(|m| trimmed.find(m).map(|i| &trimmed[i + m.len()..]))?;
    // First arg after the method is the API handle (skip it), then the route literal.
    let route = first_path_literal_after_comma(rest)?;
    let cid = normalize_endpoint(&route);
    if cid.is_empty() {
        return None;
    }
    // Handler: the last argument — `h.handlerMethod` or `h.handlerMethod,` (with options).
    let handler = extract_huma_handler(trimmed)?;
    Some((handler, cid))
}

/// Extract the route literal (second argument) from a huma registration call.
fn first_path_literal_after_comma(args: &str) -> Option<String> {
    // Skip the first argument (api handle), find the comma.
    let comma = args.find(',')?;
    let after = args[comma + 1..].trim_start();
    first_path_literal(after)
}

/// Extract the handler method name from a huma registration line.
/// Looks for `h.handlerMethod` pattern — the last `h.<identifier>` before `)` or `,`.
fn extract_huma_handler(line: &str) -> Option<String> {
    // Find `h.<identifier>` patterns.
    let mut last_handler = None;
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'h' && i + 1 < bytes.len() && bytes[i + 1] == b'.' {
            let start = i + 2;
            let mut end = start;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            if end > start {
                last_handler = Some(line[start..end].to_string());
            }
            i = end;
        } else {
            i += 1;
        }
    }
    last_handler
}

/// Extract `(TypeName, ContractId)` from a single line's `…<Type>('/path')` call
/// sites. Recognizes the typed-client shape generated API clients use; ignores
/// JSX (`<Table<Row>`) and generics with no leading-`/` route literal argument.
fn scan_type_route_bindings(line: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (i, c) in line.char_indices() {
        if c != '>' {
            continue;
        }
        let Some(lt) = line[..i].rfind('<') else {
            continue;
        };
        let Some(ty) = valid_type_name(&line[lt + 1..i]) else {
            continue;
        };
        // require `(` after the closing `>` (optional whitespace)
        let rest = line[i + 1..].trim_start();
        let Some(after_paren) = rest.strip_prefix('(') else {
            continue;
        };
        // the first argument must be a quoted `/`-path literal
        let Some(path) = first_path_literal(after_paren) else {
            continue;
        };
        let cid = normalize_endpoint(&path);
        if !cid.is_empty() {
            out.push((ty, cid));
        }
    }
    out
}

/// A single PascalCase/`_`-leading type identifier (optionally `[]`-suffixed),
/// else `None`. Rejects multi-token generics and comparison noise.
fn valid_type_name(s: &str) -> Option<String> {
    let s = s.trim().strip_suffix("[]").unwrap_or_else(|| s.trim()).trim();
    let mut chars = s.chars();
    let first = chars.next()?;
    if !(first.is_ascii_uppercase() || first == '_') {
        return None;
    }
    s.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
        .then(|| s.to_string())
}

/// The first argument, if it is a quoted (or backtick) literal whose value
/// starts with `/`. Returns the literal's content (interpolations intact).
fn first_path_literal(arg: &str) -> Option<String> {
    let s = arg.trim_start();
    let q = s.chars().next()?;
    if q != '\'' && q != '"' && q != '`' {
        return None;
    }
    let from = q.len_utf8();
    let rel = s[from..].find(q)?;
    let lit = &s[from..from + rel];
    lit.starts_with('/').then(|| lit.to_string())
}

/// The leading type identifier of a type hint: `HealthState[]` → `HealthState`,
/// `Record<string, number>` → `Record`.
fn base_type(hint: &str) -> &str {
    let h = hint.trim();
    let end = h
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(h.len());
    &h[..end]
}

fn extract_file(
    rel: &str,
    content: &str,
    lang: Lang,
    bindings: &BTreeMap<String, String>,
    go_bindings: &BTreeMap<String, String>,
) -> Vec<ContractShape> {
    let anchors = anchors(content);
    let decls = match lang {
        Lang::Go => parse_go(content),
        Lang::Ts => parse_ts(content),
        Lang::Spec => return Vec::new(),
    };
    let role = match lang {
        Lang::Go => ShapeRole::Producer,
        _ => ShapeRole::Consumer,
    };

    // Select the binding map for this language: Go uses huma route registrations
    // (#319), TS uses typed-client call sites (#313).
    let lang_bindings = match lang {
        Lang::Go => go_bindings,
        _ => bindings,
    };

    // Index named enum decls so an interface bound to a route can pull in the
    // `type X = 'A'|'B'` its fields reference — that union has no anchor/binding
    // of its own in a generated types file (needed for C3 enum mismatches).
    let union_by_name: BTreeMap<&str, &EnumDef> = decls
        .iter()
        .filter_map(|d| {
            d.enums
                .iter()
                .find_map(|e| e.name.as_deref().map(|n| (n, e)))
        })
        .collect();

    let mut by_contract: BTreeMap<String, Agg> = BTreeMap::new();
    for d in &decls {
        // Binding-first (a usage-grounded type→route link), then nearest anchor.
        let Some(cid) = lang_bindings
            .get(&d.name)
            .cloned()
            .or_else(|| contract_for_line(&anchors, d.line))
        else {
            continue;
        };
        let agg = by_contract.entry(cid).or_default();
        if agg.symbol.is_none() {
            agg.symbol = Some(d.name.clone());
        }
        for f in &d.fields {
            if !agg.fields.iter().any(|x| x.name == f.name) {
                agg.fields.push(f.clone());
            }
            // Attribute a referenced union's variants to this contract.
            if let Some(hint) = &f.type_hint {
                if let Some(e) = union_by_name.get(base_type(hint)) {
                    if !agg.enums.iter().any(|x| x.name == e.name) {
                        agg.enums.push((*e).clone());
                    }
                }
            }
        }
        for e in &d.enums {
            if !agg.enums.iter().any(|x| x.name == e.name) {
                agg.enums.push(e.clone());
            }
        }
    }

    by_contract
        .into_iter()
        .map(|(contract, agg)| ContractShape {
            contract,
            role,
            language: lang,
            source: SourceRef {
                file: rel.to_string(),
                symbol: agg.symbol,
            },
            fields: agg.fields,
            enums: agg.enums,
        })
        .collect()
}

#[derive(Default)]
struct Agg {
    symbol: Option<String>,
    fields: Vec<Field>,
    enums: Vec<EnumDef>,
}

struct Decl {
    line: usize,
    name: String,
    fields: Vec<Field>,
    enums: Vec<EnumDef>,
}

/// `(line_no, ContractId)` for every endpoint reference in the file.
fn anchors(content: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (i, line) in content.lines().enumerate() {
        for ep in scan_endpoints(line) {
            let id = normalize_endpoint(&ep);
            if !id.is_empty() {
                out.push((i, id));
            }
        }
    }
    out
}

/// Nearest endpoint anchor at or above `line`; falls back to the file's first
/// anchor so a declaration above a sole anchor still binds.
fn contract_for_line(anchors: &[(usize, String)], line: usize) -> Option<String> {
    let preceding = anchors
        .iter()
        .filter(|(l, _)| *l <= line)
        .max_by_key(|(l, _)| *l);
    preceding
        .or_else(|| anchors.first())
        .map(|(_, c)| c.clone())
}

// ---- Go -------------------------------------------------------------------

fn parse_go(content: &str) -> Vec<Decl> {
    let mut out = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let mut last_enum_type: Option<(usize, String)> = None;
    while i < lines.len() {
        let line = lines[i].trim();
        // struct: `type Name struct {`
        if let Some(name) = line
            .strip_prefix("type ")
            .and_then(|r| r.strip_suffix("struct {"))
            .map(|r| r.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            let mut fields = Vec::new();
            i += 1;
            while i < lines.len() && lines[i].trim() != "}" {
                if let Some(f) = go_field(lines[i]) {
                    fields.push(f);
                }
                i += 1;
            }
            out.push(Decl {
                line: i,
                name,
                fields,
                enums: Vec::new(),
            });
        } else if let Some(name) = line
            .strip_prefix("type ")
            .and_then(|r| r.strip_suffix(" string"))
            .map(|r| r.trim().to_string())
        {
            // `type HealthState string` — remember for the following const block.
            last_enum_type = Some((i, name));
        } else if line.starts_with("const (") {
            let mut variants = Vec::new();
            i += 1;
            while i < lines.len() && lines[i].trim() != ")" {
                variants.extend(extract_quoted(lines[i], '"'));
                i += 1;
            }
            if !variants.is_empty() {
                let (line_no, name) = last_enum_type
                    .clone()
                    .map(|(l, n)| (l, Some(n)))
                    .unwrap_or((i, None));
                out.push(Decl {
                    line: line_no,
                    name: name.clone().unwrap_or_default(),
                    fields: Vec::new(),
                    enums: vec![EnumDef { name, variants }],
                });
            }
        }
        i += 1;
    }
    out
}

/// A Go struct field with a json tag → `(json name, go type)`.
fn go_field(line: &str) -> Option<Field> {
    let name = extract_json_tag(line)?;
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let type_hint = tokens.get(1).map(|s| s.trim().to_string());
    Some(Field { name, type_hint })
}

fn extract_json_tag(line: &str) -> Option<String> {
    let start = line.find("json:\"")? + "json:\"".len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    let raw = &rest[..end];
    let name = raw.split(',').next()?.trim();
    (!name.is_empty() && name != "-").then(|| name.to_string())
}

// ---- TypeScript -----------------------------------------------------------

fn parse_ts(content: &str) -> Vec<Decl> {
    let mut out = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim().trim_start_matches("export ").trim();
        // interface: `interface Name {`
        if let Some(name) = line
            .strip_prefix("interface ")
            .and_then(|r| r.strip_suffix('{'))
            .map(|r| r.trim().to_string())
            .filter(|s| !s.is_empty())
        {
            let mut fields = Vec::new();
            i += 1;
            while i < lines.len() && lines[i].trim() != "}" {
                if let Some(f) = ts_field(lines[i]) {
                    fields.push(f);
                }
                i += 1;
            }
            out.push(Decl {
                line: i,
                name,
                fields,
                enums: Vec::new(),
            });
        } else if line.starts_with("type ") && line.contains('=') && line.contains('\'') {
            // string-union enum: `type Name = 'A' | 'B';`
            let name = line["type ".len()..]
                .split(['=', ' '])
                .next()
                .unwrap_or_default()
                .trim()
                .to_string();
            let variants = extract_quoted(line, '\'');
            if !variants.is_empty() {
                out.push(Decl {
                    line: i,
                    name: name.clone(),
                    fields: Vec::new(),
                    enums: vec![EnumDef {
                        name: Some(name),
                        variants,
                    }],
                });
            }
        }
        i += 1;
    }
    out
}

/// A TS interface member `name?: type;` → `Field`.
fn ts_field(line: &str) -> Option<Field> {
    let t = line.trim();
    if t.starts_with("//") || t.starts_with('*') || t.starts_with("/*") {
        return None;
    }
    let (lhs, rhs) = t.split_once(':')?;
    let name = lhs.trim().trim_end_matches('?').trim();
    if name.is_empty() || name.contains(char::is_whitespace) {
        return None;
    }
    let type_hint = rhs.trim().trim_end_matches([';', ',']).trim();
    Some(Field {
        name: name.to_string(),
        type_hint: (!type_hint.is_empty()).then(|| type_hint.to_string()),
    })
}

// ---- shared ----------------------------------------------------------------

/// Extract substrings between matching `quote` characters on a line.
fn extract_quoted(line: &str, quote: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = line.char_indices();
    while let Some((start, c)) = chars.next() {
        if c == quote {
            let from = start + c.len_utf8();
            let mut to = from;
            for (j, d) in line[from..].char_indices() {
                if d == quote {
                    to = from + j;
                    break;
                }
            }
            if to > from {
                out.push(line[from..to].to_string());
                // advance past the closing quote
                for (k, _) in chars.by_ref() {
                    if k >= to {
                        break;
                    }
                }
            }
        }
    }
    out
}

fn ext(path: &Path) -> Option<&str> {
    path.extension().and_then(|e| e.to_str())
}

/// Tests describe assumed contracts, not produced ones — never a contract source.
fn is_test_file(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    name.ends_with("_test.go")
        || name.contains(".test.")
        || name.contains(".spec.")
        || name.ends_with(".d.ts")
}

fn walk_code(root: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let mut entries: Vec<_> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
                if !SKIP_DIRS.contains(&name) {
                    stack.push(p);
                }
            } else if matches!(ext(&p), Some("go") | Some("ts") | Some("tsx"))
                && !is_test_file(&p)
            {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_bindings() -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    fn no_go_bindings() -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    #[test]
    fn go_struct_and_enum_extracted() {
        let src = "// GET /api/v1/services/{id}/health\n\
                   type componentResponse struct {\n\
                   \tName   string `json:\"name\"`\n\
                   \tState  string `json:\"state\"`\n\
                   \tDetail string `json:\"detail,omitempty\"`\n\
                   }\n\
                   type HealthState string\n\
                   const (\n\
                   \tA HealthState = \"OPERATIONAL\"\n\
                   \tB HealthState = \"IDLE\"\n\
                   )\n";
        let shapes = extract_file("h.go", src, Lang::Go, &no_bindings(), &no_go_bindings());
        assert_eq!(shapes.len(), 1);
        let s = &shapes[0];
        assert_eq!(s.contract, "services.health");
        assert_eq!(s.role, ShapeRole::Producer);
        let names: Vec<&str> = s.fields.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["name", "state", "detail"]);
        let variants = &s.enums.iter().flat_map(|e| e.variants.clone()).collect::<Vec<_>>();
        assert!(variants.contains(&"OPERATIONAL".to_string()));
        assert!(variants.contains(&"IDLE".to_string()));
    }

    #[test]
    fn ts_interface_and_union_extracted() {
        let src = "// consumes GET /api/v1/services/{id}/health\n\
                   export type HealthStatus = 'GREEN' | 'YELLOW' | 'RED';\n\
                   export interface ComponentHealth {\n\
                   \tname: string;\n\
                   \tstatus: HealthStatus;\n\
                   \tcpu?: number;\n\
                   }\n";
        let shapes = extract_file("t.ts", src, Lang::Ts, &no_bindings(), &no_go_bindings());
        assert_eq!(shapes.len(), 1);
        let s = &shapes[0];
        assert_eq!(s.contract, "services.health");
        assert_eq!(s.role, ShapeRole::Consumer);
        let names: Vec<&str> = s.fields.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["name", "status", "cpu"]);
        let variants = &s.enums.iter().flat_map(|e| e.variants.clone()).collect::<Vec<_>>();
        assert_eq!(variants, &vec!["GREEN", "YELLOW", "RED"]);
    }

    #[test]
    fn no_endpoint_anchor_yields_nothing() {
        let src = "type Foo struct {\n\tBar string `json:\"bar\"`\n}\n";
        assert!(extract_file("x.go", src, Lang::Go, &no_bindings(), &no_go_bindings()).is_empty());
    }

    // ---- call-site binding (#313) ----------------------------------------

    #[test]
    fn scan_bindings_parses_typed_client_call_sites() {
        let b = scan_type_route_bindings("  queryFn: () => api.get<HealthSnapshot>(`/services/${id}/health`),");
        assert_eq!(b, vec![("HealthSnapshot".to_string(), "services.health".to_string())]);

        let b = scan_type_route_bindings("api.post<Incident>('/incidents', input)");
        assert_eq!(b, vec![("Incident".to_string(), "incidents".to_string())]);

        // query string stripped
        let b = scan_type_route_bindings("api.get<SearchRecordsResponse>('/audit/records?action_type=AI_AUTO')");
        assert_eq!(b, vec![("SearchRecordsResponse".to_string(), "audit.records".to_string())]);
    }

    #[test]
    fn scan_bindings_ignores_jsx_and_argless_generics() {
        // JSX generic component, no route → not a binding.
        assert!(scan_type_route_bindings("        <Table<ComponentHealth>").is_empty());
        // generic call with a non-path first argument → not a binding.
        assert!(scan_type_route_bindings("const [x] = useState<HealthSnapshot>(null);").is_empty());
        assert!(scan_type_route_bindings("useQuery<HealthSnapshot>({ queryKey })").is_empty());
    }

    #[test]
    fn binding_keys_anchorless_type_and_resolves_its_enum() {
        // A generated types file: no endpoint anchor anywhere.
        let src = "export type Semaphore = 'GREEN' | 'RED';\n\
                   export interface DashboardComponent {\n\
                   \tname: string;\n\
                   \tstatus: Semaphore;\n\
                   \tcpu?: number;\n\
                   }\n";
        // Without a binding the anchorless file yields nothing (conservative).
        assert!(extract_file("types.gen.ts", src, Lang::Ts, &no_bindings(), &no_go_bindings()).is_empty());

        // With a call-site binding the interface keys to its route, and the
        // referenced union's variants ride along (for C3).
        let bindings = BTreeMap::from([("DashboardComponent".to_string(), "services.health".to_string())]);
        let shapes = extract_file("types.gen.ts", src, Lang::Ts, &bindings, &no_go_bindings());
        assert_eq!(shapes.len(), 1);
        let s = &shapes[0];
        assert_eq!(s.contract, "services.health");
        let names: Vec<&str> = s.fields.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["name", "status", "cpu"]);
        let variants: Vec<String> = s.enums.iter().flat_map(|e| e.variants.clone()).collect();
        assert_eq!(variants, vec!["GREEN", "RED"]);
    }

    // ---- huma route keying (#319) ----------------------------------------

    #[test]
    fn huma_registration_parsed() {
        let (handler, cid) = parse_huma_registration(
            "huma.Get(api, \"/api/v1/services/{service_id}/health\", h.getServiceHealth)"
        ).unwrap();
        assert_eq!(handler, "getServiceHealth");
        assert_eq!(cid, "services.health");

        let (handler, cid) = parse_huma_registration(
            "huma.Post(api, \"/api/v1/incidents\", h.createIncident)"
        ).unwrap();
        assert_eq!(handler, "createIncident");
        assert_eq!(cid, "incidents");
    }

    #[test]
    fn huma_output_struct_keys_to_route_not_nearest_anchor() {
        // Huma pattern: registrations in one block, struct defined far below.
        // Without go_bindings, nearest-anchor binds to the wrong route.
        let src = "huma.Get(api, \"/api/v1/services/{id}/health\", h.getServiceHealth)\n\
                   huma.Get(api, \"/api/v1/services/{id}/metrics\", h.getServiceMetrics)\n\
                   \n\
                   type getServiceHealthOutput struct {\n\
                   \tName   string `json:\"name\"`\n\
                   \tState  string `json:\"state\"`\n\
                   }\n";
        let files = vec![("handlers.go".to_string(), Lang::Go, src.to_string())];
        let go_bindings = collect_go_bindings(&files);
        // getServiceHealthOutput should bind to services.health, not services.metrics.
        assert_eq!(
            go_bindings.get("getServiceHealthOutput").map(|s| s.as_str()),
            Some("services.health")
        );
        let shapes = extract_file("handlers.go", src, Lang::Go, &no_bindings(), &go_bindings);
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].contract, "services.health");
        let names: Vec<&str> = shapes[0].fields.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["name", "state"]);
    }
}
