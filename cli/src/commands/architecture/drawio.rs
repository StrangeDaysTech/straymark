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
        let id = xml_escape(&comp.id);
        let label = xml_escape(&comp.label);
        out.push_str(&format!(
            r#"        <object label="{label}" straymark_component_id="{id}" id="comp-{id}">
          <mxCell style="rounded=0;whiteSpace=wrap;html=1;" vertex="1" parent="1">
            <mxGeometry x="{x}" y="{y}" width="{BOX_W}" height="{BOX_H}" as="geometry" />
          </mxCell>
        </object>
"#
        ));
    }
}

/// Escape the five XML predefined entities for use in attribute values.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
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
}
