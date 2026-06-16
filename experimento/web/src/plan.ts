//! Architecture Plan view (Loom A2.3 + A2.4, Spec 002 §5/§6/§8): render
//! `plan.drawio` with maxGraph (preserving the human geometry, NFR1), overlay
//! the §4 status as non-destructive colors, and wire the interactions —
//! component detail panel (S2), layer toggle (FR4), the "where are we" panel
//! (§8), and cross-view links into the knowledge graph (FR8).
//!
//! We parse the DrawIO XML ourselves and recreate cells with maxGraph's stable
//! `insertVertex`/`insertEdge` (the experimental `ModelXmlSerializer` codec
//! doesn't round-trip DrawIO `<object>` user-cells in 0.23). Each vertex's cell
//! id is its `straymark_component_id`, so a click maps straight to the model.

import {
  Graph,
  InternalEvent,
  FitPlugin,
  PanningHandler,
  type Cell,
  type CellStyle,
  type EventObject,
} from '@maxgraph/core';
import { t } from './i18n';

interface ArchComponent {
  id: string;
  label: string;
  layer: string;
  states: string[];
}
interface ArchLayer {
  id: string;
  label: string;
  order: number;
  counts: Record<string, number>;
}
interface ArchResponse {
  model_present: boolean;
  layers: ArchLayer[];
  components: ArchComponent[];
}
interface ComponentDetail {
  id: string;
  label: string;
  layer: string;
  states: string[];
  owned_files: string[];
  docs: string[];
  charters: { charter_id: string; title: string; status: string }[];
}
interface WhereResponse {
  active_charters: { charter_id: string; title: string }[];
  declared_files: number;
  touched_files: number;
  recent_ailogs: string[];
  open_debt_files: number;
}

/** Cross-view bridge into the knowledge graph (implemented by `main.ts`). */
export interface PlanHooks {
  openNode(nodeId: string): void;
}

const STATE_STYLE: Record<string, { fill: string; stroke: string }> = {
  active: { fill: '#2e7d46', stroke: '#5ad18a' },
  'in-progress': { fill: '#8a6d1f', stroke: '#e8c34a' },
  implemented: { fill: '#24506e', stroke: '#5b9fc8' },
  'has-debt': { fill: '#6e2440', stroke: '#c97aa0' },
  'wiring-gap': { fill: '#6e2a24', stroke: '#c97a72' },
  uncharted: { fill: '#23262f', stroke: '#3a3f4d' },
};
const PRIORITY = ['active', 'in-progress', 'implemented', 'has-debt', 'wiring-gap', 'uncharted'];
export const LEGEND_STATES = PRIORITY;

let graph: Graph | null = null;
let hooks: PlanHooks = { openNode: () => {} };
let cellsByLayer = new Map<string, Cell[]>();

const detailEl = () => document.getElementById('plan-detail')!;
const whereEl = () => document.getElementById('plan-where')!;

/** Install the cross-view bridge (called once at boot). */
export function setPlanHooks(h: PlanHooks): void {
  hooks = h;
}

/** (Re)render the plan + its panels. Idempotent — safe on every `architecture`
 * WS event. */
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
    closeDetail();
    whereEl().innerHTML = '';
    return;
  }

  const doc = new DOMParser().parseFromString(planXml, 'application/xml');
  if (doc.querySelector('parsererror') || !doc.querySelector('object[straymark_component_id]')) {
    showMessage(container, t('plan.error'));
    return;
  }
  const compById = new Map(arch.components.map((c) => [c.id, c]));

  container.textContent = '';
  graph = new Graph(container);
  graph.setPanning(true);
  graph.setCellsMovable(false);
  graph.setCellsEditable(false);
  graph.setCellsResizable(false);
  graph.setConnectable(false);
  // Drag-to-pan with the left button anywhere (cells are locked, so dragging
  // a box pans rather than moves it; a plain click still opens its detail).
  const panning = graph.getPlugin<PanningHandler>(PanningHandler.pluginId);
  if (panning) {
    panning.useLeftButtonForPanning = true;
    panning.ignoreCell = true;
  }
  const parent = graph.getDefaultParent();
  const cellByDomId = new Map<string, Cell>();
  cellsByLayer = new Map();

  graph.batchUpdate(() => {
    for (const obj of Array.from(doc.querySelectorAll('object[straymark_component_id]'))) {
      const compId = obj.getAttribute('straymark_component_id') ?? '';
      const geom = obj.querySelector('mxGeometry');
      if (!geom) continue;
      const state = pickState(compById.get(compId)?.states ?? []);
      const palette = STATE_STYLE[state] ?? STATE_STYLE.uncharted;
      const style: CellStyle = {
        fillColor: palette.fill,
        strokeColor: palette.stroke,
        fontColor: '#e8eaf0',
        rounded: false,
      };
      const cell = graph!.insertVertex({
        parent,
        id: compId,
        value: obj.getAttribute('label') ?? compId,
        position: [num(geom, 'x'), num(geom, 'y')],
        size: [num(geom, 'width', 200), num(geom, 'height', 60)],
        style,
      });
      const domId = obj.getAttribute('id');
      if (domId) cellByDomId.set(domId, cell);
      const layer = compById.get(compId)?.layer ?? '';
      const list = cellsByLayer.get(layer) ?? [];
      list.push(cell);
      cellsByLayer.set(layer, list);
    }
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

  graph.addListener(InternalEvent.CLICK, (_sender: unknown, evt: EventObject) => {
    const cell = evt.getProperty('cell') as Cell | null;
    const id = cell?.isVertex() ? cell.getId() : null;
    if (id) void showDetail(id);
    else closeDetail();
  });

  wireZoom(container);
  fitView();
  renderWherePanel(arch.layers);
}

// ── Zoom / pan (so a big plan fits a small window) ───────────────────────────

/** Scale + center the whole plan to fit the viewport (initial + Fit button). */
function fitView(): void {
  if (!graph) return;
  graph.getPlugin<FitPlugin>('fit')?.fitCenter({ margin: 24 });
}

/** Wheel-zoom on the canvas + wire the zoom buttons (idempotent: `onclick`). */
function wireZoom(container: HTMLElement): void {
  container.onwheel = (e: WheelEvent) => {
    if (!graph) return;
    e.preventDefault();
    if (e.deltaY < 0) graph.zoomIn();
    else graph.zoomOut();
  };
  const onClick = (id: string, fn: () => void) => {
    const el = document.getElementById(id);
    if (el) el.onclick = fn;
  };
  onClick('plan-zoom-in', () => graph?.zoomIn());
  onClick('plan-zoom-out', () => graph?.zoomOut());
  onClick('plan-zoom-fit', () => fitView());
}

// ── Component detail panel (S2) ──────────────────────────────────────────────

async function showDetail(compId: string): Promise<void> {
  const detail = await fetch(`/api/architecture/component/${encodeURIComponent(compId)}`)
    .then((r) => (r.ok ? (r.json() as Promise<ComponentDetail>) : null))
    .catch(() => null);
  if (!detail) return;

  const badges = detail.states
    .map((s) => {
      const c = STATE_STYLE[s] ?? STATE_STYLE.uncharted;
      return `<span class="pd-badge" style="background:${c.fill};border-color:${c.stroke};color:#e8eaf0">${t('plan.state.' + s)}</span>`;
    })
    .join('');
  const charters = detail.charters.length
    ? `<h3>${t('plan.detail.charters')}</h3><ul>${detail.charters
        .map(
          (c) =>
            `<li><button class="pd-link" data-node="${esc(c.charter_id)}">${esc(c.title)}</button> <span class="pw-k">${esc(c.status)}</span></li>`,
        )
        .join('')}</ul>`
    : '';
  const files = detail.owned_files.length
    ? `<h3>${t('plan.detail.files')} (${detail.owned_files.length})</h3><ul>${detail.owned_files
        .slice(0, 50)
        .map((f) => `<li>${esc(f)}</li>`)
        .join('')}</ul>`
    : '';

  const el = detailEl();
  el.innerHTML =
    `<button class="close" aria-label="Close">×</button>` +
    `<h2>${esc(detail.label)}</h2>` +
    `<div class="pd-layer">${esc(detail.layer)}</div>` +
    `<div class="pd-badges">${badges || `<span class="pw-k">${t('plan.state.uncharted')}</span>`}</div>` +
    charters +
    files;
  el.classList.add('open');
  el.querySelector('.close')?.addEventListener('click', closeDetail);
  for (const b of Array.from(el.querySelectorAll<HTMLElement>('.pd-link[data-node]'))) {
    b.addEventListener('click', () => hooks.openNode(b.dataset.node!));
  }
}

function closeDetail(): void {
  detailEl().classList.remove('open');
}

// ── "Where are we" panel (§8) + layer toggle (FR4) ───────────────────────────

async function renderWherePanel(layers: ArchLayer[]): Promise<void> {
  const where = await fetch('/api/where')
    .then((r) => (r.ok ? (r.json() as Promise<WhereResponse>) : null))
    .catch(() => null);

  const el = whereEl();
  const pct = where && where.declared_files > 0
    ? Math.round((where.touched_files / where.declared_files) * 100)
    : 0;
  const active = where?.active_charters.length
    ? where.active_charters.map((c) => `<div class="pw-charter">${esc(c.title)}</div>`).join('')
    : `<div class="pw-k">${t('plan.where.noCharter')}</div>`;
  const progress = where && where.declared_files > 0
    ? `<div class="pw-row"><span class="pw-k">${t('plan.where.progress')}:</span> ${where.touched_files}/${where.declared_files} (${pct}%)<div class="pw-bar"><i style="width:${pct}%"></i></div></div>`
    : '';
  const ailogs = where?.recent_ailogs.length
    ? `<div class="pw-row"><span class="pw-k">${t('plan.where.recent')}:</span> ${where.recent_ailogs.slice(0, 3).map(esc).join(', ')}</div>`
    : '';
  const debt = where
    ? `<div class="pw-row"><span class="pw-k">${t('plan.where.debt')}:</span> ${where.open_debt_files}</div>`
    : '';

  const layerRows = layers
    .filter((l) => (cellsByLayer.get(l.id) ?? []).length > 0)
    .map((l) => {
      const n = (cellsByLayer.get(l.id) ?? []).length;
      return `<label class="pw-layer"><input type="checkbox" data-layer="${esc(l.id)}" checked />${esc(l.label)}<span class="pw-count">${n}</span></label>`;
    })
    .join('');

  el.innerHTML =
    `<h2>${t('plan.where.title')}</h2>` +
    `<div class="pw-row">${active}</div>` +
    progress +
    ailogs +
    debt +
    (layerRows ? `<div class="pw-layers"><h3>${t('plan.where.layers')}</h3>${layerRows}</div>` : '');

  for (const cb of Array.from(el.querySelectorAll<HTMLInputElement>('.pw-layer input[data-layer]'))) {
    cb.addEventListener('change', () => setLayerVisible(cb.dataset.layer!, cb.checked));
  }
}

/** Show/hide a layer's component cells (FR4 — the 2D "peel floors" analog). */
function setLayerVisible(layerId: string, visible: boolean): void {
  if (!graph) return;
  const cells = cellsByLayer.get(layerId) ?? [];
  const model = graph.getDataModel();
  graph.batchUpdate(() => {
    for (const cell of cells) model.setVisible(cell, visible);
  });
}

// ── helpers ──────────────────────────────────────────────────────────────────

function num(el: Element, attr: string, fallback = 0): number {
  const v = Number(el.getAttribute(attr));
  return Number.isFinite(v) ? v : fallback;
}

function pickState(states: string[]): string {
  for (const p of PRIORITY) {
    if (states.includes(p)) return p;
  }
  return 'uncharted';
}

export function stateColor(state: string): { fill: string; stroke: string } {
  return STATE_STYLE[state] ?? STATE_STYLE.uncharted;
}

function esc(s: string): string {
  return s.replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]!);
}

function showMessage(container: HTMLElement, msg: string): void {
  graph = null;
  container.innerHTML = `<div class="plan-empty">${esc(msg)}</div>`;
}
