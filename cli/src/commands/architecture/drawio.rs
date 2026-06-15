//! Pure-Rust mxGraph (DrawIO) emitter for `plan.drawio` (Spec 002 §3.2, T2.4).
//!
//! Emits one vertex per component, grouped into rows by layer (dagre-style
//! layered layout — no JS dependency). Each cell carries the custom attribute
//! `straymark_component_id` via DrawIO's `<object>` wrapper (how DrawIO persists
//! custom attrs), so Loom (A2) can key non-destructive status styles on it
//! without rewriting the human's geometry (NFR1). Geometry is a deterministic
//! first draft the human rearranges in DrawIO.

use straymark_core::architecture::ArchModel;

const MARGIN_X: i32 = 40;
const MARGIN_Y: i32 = 40;
const BOX_W: i32 = 200;
const BOX_H: i32 = 60;
const GAP_X: i32 = 40;
const GAP_Y: i32 = 80;

/// Render an `ArchModel` as an uncompressed DrawIO `.drawio` document.
pub fn render_drawio(model: &ArchModel) -> String {
    let mut cells = String::new();

    // Layers in declared order; components whose layer is unknown go in a
    // trailing row so nothing is silently dropped.
    let mut row = 0;
    let mut layer_ids: Vec<&str> = Vec::new();
    let mut ordered = model.layers.clone();
    ordered.sort_by_key(|l| l.order);
    for layer in &ordered {
        layer_ids.push(&layer.id);
        emit_row(
            &mut cells,
            row,
            model.components.iter().filter(|c| c.layer == layer.id),
        );
        row += 1;
    }
    // Orphan components (layer not declared).
    let orphans = model
        .components
        .iter()
        .filter(|c| !layer_ids.contains(&c.layer.as_str()));
    if orphans.clone().next().is_some() {
        emit_row(&mut cells, row, orphans);
    }

    format!(
        r#"<mxfile host="straymark" type="device">
  <diagram name="Architecture" id="straymark-architecture">
    <mxGraphModel dx="1024" dy="768" grid="1" gridSize="10" guides="1" tooltips="1" connect="1" arrows="1" fold="1" page="1" pageScale="1" pageWidth="1100" pageHeight="850" math="0" shadow="0">
      <root>
        <mxCell id="0" />
        <mxCell id="1" parent="0" />
{cells}      </root>
    </mxGraphModel>
  </diagram>
</mxfile>
"#
    )
}

/// Append one row of component cells (left-to-right) for a layer.
fn emit_row<'a>(out: &mut String, row: i32, components: impl Iterator<Item = &'a straymark_core::architecture::Component>) {
    let y = MARGIN_Y + row * (BOX_H + GAP_Y);
    for (col, comp) in components.enumerate() {
        let x = MARGIN_X + col as i32 * (BOX_W + GAP_X);
        out.push_str(&render_cell(comp, x, y));
    }
}

/// Render a single component vertex at `(x, y)`. The custom attribute
/// `straymark_component_id` is carried on the DrawIO `<object>` wrapper.
pub fn render_cell(comp: &straymark_core::architecture::Component, x: i32, y: i32) -> String {
    let id = xml_escape(&comp.id);
    let label = xml_escape(&comp.label);
    format!(
        r#"        <object label="{label}" straymark_component_id="{id}" id="comp-{id}">
          <mxCell style="rounded=0;whiteSpace=wrap;html=1;" vertex="1" parent="1">
            <mxGeometry x="{x}" y="{y}" width="{BOX_W}" height="{BOX_H}" as="geometry" />
          </mxCell>
        </object>
"#
    )
}

/// Append `components` to an existing DrawIO document as a new trailing row,
/// preserving every existing byte (cells are text-inserted right before
/// `</root>`). NFR1: `sync` never rewrites the human's geometry. Returns `None`
/// if the document has no `</root>` (not a model we wrote).
pub fn append_cells(
    existing_xml: &str,
    components: &[straymark_core::architecture::Component],
) -> Option<String> {
    if components.is_empty() {
        return Some(existing_xml.to_string());
    }
    let anchor = existing_xml.rfind("</root>")?;
    let y = match max_y(existing_xml) {
        Some(prev) => prev + BOX_H + GAP_Y,
        None => MARGIN_Y,
    };
    let mut cells = String::new();
    for (col, comp) in components.iter().enumerate() {
        let x = MARGIN_X + col as i32 * (BOX_W + GAP_X);
        cells.push_str(&render_cell(comp, x, y));
    }
    let mut out = String::with_capacity(existing_xml.len() + cells.len());
    out.push_str(&existing_xml[..anchor]);
    out.push_str(&cells);
    out.push_str(&existing_xml[anchor..]);
    Some(out)
}

/// Largest `y="…"` geometry value in the document (the bottom-most row).
pub fn max_y(xml: &str) -> Option<i32> {
    let mut best: Option<i32> = None;
    let mut rest = xml;
    while let Some(i) = rest.find("y=\"") {
        rest = &rest[i + 3..];
        if let Some(end) = rest.find('"') {
            if let Ok(v) = rest[..end].parse::<i32>() {
                best = Some(best.map_or(v, |b| b.max(v)));
            }
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }
    best
}

/// Extract every `straymark_component_id` attribute value (XML-unescaped).
pub fn parse_component_ids(xml: &str) -> Vec<String> {
    const ATTR: &str = "straymark_component_id=\"";
    let mut ids = Vec::new();
    let mut rest = xml;
    while let Some(i) = rest.find(ATTR) {
        rest = &rest[i + ATTR.len()..];
        if let Some(end) = rest.find('"') {
            ids.push(xml_unescape(&rest[..end]));
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }
    ids
}

/// Escape the five XML predefined entities for use in attribute values.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Reverse of [`xml_escape`] (`&amp;` last, so earlier replacements aren't
/// re-decoded).
fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;
    use straymark_core::architecture::{Component, Layer};

    fn comp(id: &str, layer: &str, label: &str) -> Component {
        Component {
            id: id.to_string(),
            label: label.to_string(),
            layer: layer.to_string(),
            globs: vec![format!("{id}/**")],
            links: Vec::new(),
            docs: Vec::new(),
            external: false,
        }
    }

    fn model() -> ArchModel {
        ArchModel {
            version: 0,
            layers: vec![
                Layer { id: "core".into(), label: "Core".into(), order: 1 },
                Layer { id: "frontend".into(), label: "Frontend".into(), order: 0 },
            ],
            components: vec![
                comp("cli", "core", "CLI"),
                comp("core", "core", "Core"),
                comp("web", "frontend", "Web"),
            ],
        }
    }

    #[test]
    fn one_cell_per_component_with_attr() {
        let xml = render_drawio(&model());
        assert!(xml.contains("<mxfile"));
        assert!(xml.contains("<mxGraphModel"));
        // one straymark_component_id per component
        for id in ["cli", "core", "web"] {
            assert_eq!(
                xml.matches(&format!("straymark_component_id=\"{id}\"")).count(),
                1,
                "expected exactly one cell for {id}"
            );
        }
        // base cells present
        assert!(xml.contains(r#"<mxCell id="0" />"#));
        assert!(xml.contains(r#"<mxCell id="1" parent="0" />"#));
    }

    #[test]
    fn orphan_layer_component_still_emitted() {
        let mut m = model();
        m.components.push(comp("ghost", "nonexistent", "Ghost"));
        let xml = render_drawio(&m);
        assert!(xml.contains("straymark_component_id=\"ghost\""));
    }

    #[test]
    fn labels_are_xml_escaped() {
        let mut m = model();
        m.components.push(comp("amp", "core", "A & B <x>"));
        let xml = render_drawio(&m);
        assert!(xml.contains("label=\"A &amp; B &lt;x&gt;\""));
    }

    #[test]
    fn parse_component_ids_roundtrips_and_unescapes() {
        let m = model();
        let xml = render_drawio(&m);
        let mut ids = parse_component_ids(&xml);
        ids.sort();
        assert_eq!(ids, vec!["cli", "core", "web"]);
        // unescape applied
        let raw = r#"<object straymark_component_id="a&amp;b" />"#;
        assert_eq!(parse_component_ids(raw), vec!["a&b"]);
    }

    #[test]
    fn append_cells_preserves_existing_bytes() {
        let xml = render_drawio(&model());
        let before_anchor = &xml[..xml.rfind("</root>").unwrap()];
        let new = comp("extra", "core", "Extra");
        let appended = append_cells(&xml, std::slice::from_ref(&new)).unwrap();
        // every pre-existing byte up to </root> is preserved verbatim
        assert!(appended.starts_with(before_anchor));
        assert!(appended.contains("straymark_component_id=\"extra\""));
        // the new cell sits below the previous max-y row
        let prev_max = max_y(&xml).unwrap();
        assert!(max_y(&appended).unwrap() > prev_max);
        // exactly one more cell than before
        assert_eq!(
            parse_component_ids(&appended).len(),
            parse_component_ids(&xml).len() + 1
        );
    }

    #[test]
    fn append_cells_none_without_root() {
        assert!(append_cells("<not-drawio/>", &[comp("x", "core", "X")]).is_none());
    }
}
