//! Heuristic contract-shape extraction from code (Go producers, TS consumers).
//!
//! Not a real parser — a tolerant, line-oriented heuristic good enough to key a
//! contract's *shape* (field names, enum variants) to a `ContractId` derived
//! from the nearest `/api/...` endpoint anchor. Conservative by design (R3/R6):
//! a file with no endpoint anchor yields nothing rather than a guess.
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
    let mut out = Vec::new();
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
        out.extend(extract_file(&rel, &content, lang));
    }
    out.sort_by(|a, b| (&a.contract, &a.source.file).cmp(&(&b.contract, &b.source.file)));
    out
}

fn extract_file(rel: &str, content: &str, lang: Lang) -> Vec<ContractShape> {
    let anchors = anchors(content);
    if anchors.is_empty() {
        return Vec::new();
    }
    let decls = match lang {
        Lang::Go => parse_go(content),
        Lang::Ts => parse_ts(content),
        Lang::Spec => return Vec::new(),
    };
    let role = match lang {
        Lang::Go => ShapeRole::Producer,
        _ => ShapeRole::Consumer,
    };

    let mut by_contract: BTreeMap<String, Agg> = BTreeMap::new();
    for d in decls {
        let Some(cid) = contract_for_line(&anchors, d.line) else {
            continue;
        };
        let agg = by_contract.entry(cid).or_default();
        if agg.symbol.is_none() {
            agg.symbol = Some(d.name.clone());
        }
        for f in d.fields {
            if !agg.fields.iter().any(|x| x.name == f.name) {
                agg.fields.push(f);
            }
        }
        agg.enums.extend(d.enums);
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
            } else if matches!(ext(&p), Some("go") | Some("ts") | Some("tsx")) {
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
        let shapes = extract_file("h.go", src, Lang::Go);
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
        let shapes = extract_file("t.ts", src, Lang::Ts);
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
        assert!(extract_file("x.go", src, Lang::Go).is_empty());
    }
}
