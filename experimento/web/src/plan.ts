//! Architecture Plan view (Loom A2.3, Spec 002 §6): render `plan.drawio` with
//! maxGraph, **preserving the human geometry**, and apply the §4 status as
//! non-destructive cell colors keyed on the `straymark_component_id` each DrawIO
//! `<object>` carries (NFR1).
//!
//! We parse the DrawIO XML ourselves (the geometry — x/y/w/h per cell, the
//! source/target of each edge) and recreate the vertices with maxGraph's stable
//! `insertVertex`/`insertEdge`, instead of the experimental `ModelXmlSerializer`
//! codec (which does not round-trip DrawIO `<object>` user-cells in 0.23). The
//! geometry comes straight from the file, so a human's hand-arranged plan is
//! reproduced verbatim; status is only ever a color, never a layout change.
//!
//! The model status comes from `/api/architecture`; the geometry from
//! `/api/architecture/plan.drawio`. Layer toggles, a component detail panel, and
//! the "where are we" panel are A2.4.

import { Graph, type Cell, type CellStyle } from '@maxgraph/core';
import { t } from './i18n';

interface ArchComponent {
  id: string;
  states: string[];
}
interface ArchResponse {
  model_present: boolean;
  components: ArchComponent[];
}

/** Fill + stroke per projected state (matches the CLI `status --where` palette). */
const STATE_STYLE: Record<string, { fill: string; stroke: string }> = {
  active: { fill: '#2e7d46', stroke: '#5ad18a' },
  'in-progress': { fill: '#8a6d1f', stroke: '#e8c34a' },
  implemented: { fill: '#24506e', stroke: '#5b9fc8' },
  'has-debt': { fill: '#6e2440', stroke: '#c97aa0' },
  'wiring-gap': { fill: '#6e2a24', stroke: '#c97a72' },
  uncharted: { fill: '#23262f', stroke: '#3a3f4d' },
};
/** When a component holds several states, the box fill shows the most salient. */
const PRIORITY = ['active', 'in-progress', 'implemented', 'has-debt', 'wiring-gap', 'uncharted'];

/** The legend's state order (localized labels rendered by the caller). */
export const LEGEND_STATES = PRIORITY;

let graph: Graph | null = null;

/** (Re)render the plan into `container`. Idempotent — safe to call on every
 * `architecture` WS event. */
export async function renderPlan(container: HTMLElement): Promise<void> {
  let arch: ArchResponse;
  let planXml: string | null;
  try {
    const [a, p] = await Promise.all([
      fetch('/api/architecture').then((r) => r.json() as Promise<ArchResponse>),
      fetch('/api/architecture/plan.drawio').then((r) => (r.ok ? r.text() : null)),
    ]);
    arch = a;
    planXml = p;
  } catch {
    showMessage(container, t('plan.error'));
    return;
  }

  if (!arch.model_present || !planXml) {
    showMessage(container, t('plan.empty'));
    return;
  }

  const doc = new DOMParser().parseFromString(planXml, 'application/xml');
  if (doc.querySelector('parsererror') || !doc.querySelector('object[straymark_component_id]')) {
    showMessage(container, t('plan.error'));
    return;
  }
  const stateById = new Map(arch.components.map((c) => [c.id, c.states]));

  container.textContent = '';
  graph = new Graph(container);
  graph.setPanning(true);
  graph.setEnabled(false); // read-only display: never edit the human's geometry
  const parent = graph.getDefaultParent();
  const cellByDomId = new Map<string, Cell>();

  graph.batchUpdate(() => {
    // Vertices — one per DrawIO `<object straymark_component_id>` at its authored
    // geometry, filled by projected state.
    for (const obj of Array.from(doc.querySelectorAll('object[straymark_component_id]'))) {
      const compId = obj.getAttribute('straymark_component_id') ?? '';
      const geom = obj.querySelector('mxGeometry');
      if (!geom) continue;
      const state = pickState(stateById.get(compId) ?? []);
      const palette = STATE_STYLE[state] ?? STATE_STYLE.uncharted;
      const style: CellStyle = {
        fillColor: palette.fill,
        strokeColor: palette.stroke,
        fontColor: '#e8eaf0',
        rounded: false,
      };
      const cell = graph!.insertVertex({
        parent,
        value: obj.getAttribute('label') ?? compId,
        position: [num(geom, 'x'), num(geom, 'y')],
        size: [num(geom, 'width', 200), num(geom, 'height', 60)],
        style,
      });
      const domId = obj.getAttribute('id');
      if (domId) cellByDomId.set(domId, cell);
    }
    // Edges — the dependency arrows, by the DrawIO cell ids they connect.
    for (const edge of Array.from(doc.querySelectorAll('mxCell[edge="1"]'))) {
      const source = cellByDomId.get(edge.getAttribute('source') ?? '');
      const target = cellByDomId.get(edge.getAttribute('target') ?? '');
      if (source && target) {
        graph!.insertEdge({
          parent,
          source,
          target,
          style: { strokeColor: '#6b7280', endArrow: 'block', rounded: true },
        });
      }
    }
  });
  graph.center(true, true);
}

/** Parse a numeric attribute off an mxGeometry, with a fallback. */
function num(el: Element, attr: string, fallback = 0): number {
  const v = Number(el.getAttribute(attr));
  return Number.isFinite(v) ? v : fallback;
}

/** The most salient state of a component (drives its box fill). */
function pickState(states: string[]): string {
  for (const p of PRIORITY) {
    if (states.includes(p)) return p;
  }
  return 'uncharted';
}

/** The fill/stroke for a state name (for the legend swatches). */
export function stateColor(state: string): { fill: string; stroke: string } {
  return STATE_STYLE[state] ?? STATE_STYLE.uncharted;
}

function showMessage(container: HTMLElement, msg: string): void {
  graph = null;
  container.innerHTML = `<div class="plan-empty">${msg}</div>`;
}
