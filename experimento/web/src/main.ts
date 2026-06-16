// Loom M3 — rich UI: incremental deltas, cycle reporting, centrality sizing,
// search / pin / open-in-editor, and project-driven i18n.

import Graph, { UndirectedGraph } from 'graphology';
import louvain from 'graphology-communities-louvain';
import { circular } from 'graphology-layout';
import forceAtlas2 from 'graphology-layout-forceatlas2';
import betweennessCentrality from 'graphology-metrics/centrality/betweenness';
import pagerank from 'graphology-metrics/centrality/pagerank';
import Sigma from 'sigma';

import { setLocale, t } from './i18n';
import { renderPlan, stateColor, LEGEND_STATES, setPlanHooks } from './plan';
import { renderAxon, dispose as disposeAxon, setExplode } from './axon';

interface ApiNode {
  id: string;
  has_explicit_id: boolean;
  doc_type: string;
  title: string;
  status: string;
  risk_level: string;
  created: string | null;
  agent: string | null;
  tags: string[];
  path: string;
  degree_in: number;
  degree_out: number;
}

interface ApiEdge {
  id: number;
  source: string;
  target: string;
  edge_type: string;
  resolved: boolean;
}

interface Cycle {
  node_ids: string[];
  edge_types: string[];
}

interface ApiStats {
  total_docs: number;
  total_edges: number;
  by_type: Record<string, number>;
  by_status: Record<string, number>;
  by_risk: Record<string, number>;
  orphans: string[];
  broken_links: ApiEdge[];
  file_references: number;
  external_links: number;
  cycles: Cycle[];
}

interface ApiGraph {
  nodes: ApiNode[];
  edges: ApiEdge[];
  stats: ApiStats;
}

interface GraphDelta {
  event: 'delta';
  added: ApiNode[];
  removed: string[];
  changed: ApiNode[];
  edges: ApiEdge[];
  stats: ApiStats;
}

interface Thread {
  node_ids: string[];
  edge_ids: number[];
}

const COMMUNITY_COLORS = [
  '#e8833a', '#5d9cec', '#4caf78', '#9b7fd4', '#d4a93c', '#3cbfb4',
  '#d4607f', '#8aa455', '#c75050', '#7f9ad4', '#b06a4f', '#74b89a',
];
const TYPE_COLORS: Record<string, string> = {
  AILOG: '#e8833a', AIDEC: '#d4a93c', ADR: '#5d9cec', ETH: '#9b7fd4',
  REQ: '#4caf78', TES: '#3cbfb4', INC: '#c75050', TDE: '#b06a4f',
  SEC: '#d4607f', MCARD: '#7f9ad4', SBOM: '#8aa455', DPIA: '#b07fd4',
  PIPIA: '#c08fb0', CACFILE: '#a0788f', TC260RA: '#90a0b8', AILABEL: '#74b89a',
  // R2 — non-document entities.
  CHARTER: '#e6a817', PLAN: '#7e8fd4', AUDIT: '#bf6f8f',
};
const FALLBACK_COLOR = '#8b91a0';
const DIM_NODE = '#2c303b';
const DIM_EDGE = '#1b1e25';
const THREAD_EDGE = '#e8833a';
const SHORT_LABEL_CHARS = 42;
const MAX_LEGEND_COMMUNITIES = 8;
const MIN_SIZE = 4;
const MAX_SIZE = 14;

type SizingMode = 'degree' | 'betweenness' | 'pagerank';

const colorForCommunity = (community: number) =>
  COMMUNITY_COLORS[community % COMMUNITY_COLORS.length];
const escapeHtml = (s: string) =>
  s.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);

function shortLabel(title: string): string {
  if (title.length <= SHORT_LABEL_CHARS) return title;
  const cut = title.slice(0, SHORT_LABEL_CHARS);
  const lastSpace = cut.lastIndexOf(' ');
  return cut.slice(0, lastSpace > 24 ? lastSpace : SHORT_LABEL_CHARS).trimEnd() + '…';
}

const graph = new Graph({ multi: true, type: 'directed' });
let thread: { nodes: Set<string>; edges: Set<string> } | null = null;
let selected: string | null = null;
let hovered: string | null = null;
let focusedCommunity: number | null = null;
let pinned: Set<string> | null = null;
let sizingMode: SizingMode = 'betweenness';
let showLabels = true;
let hideIsolated = false;
let isolated = new Set<string>();
const openStatsSections = new Set<string>();

const container = document.getElementById('graph')!;
const countsEl = document.getElementById('counts')!;
const statusEl = document.getElementById('status')!;
const selectionEl = document.getElementById('selection')!;
const legendEl = document.getElementById('legend')!;
const statsEl = document.getElementById('stats')!;
const filterForm = document.getElementById('filters') as HTMLFormElement;
const searchEl = document.getElementById('search') as HTMLInputElement;
const searchResultsEl = document.getElementById('search-results')!;
const sizingEl = document.getElementById('sizing') as HTMLSelectElement;
const labelsToggle = document.getElementById('labels-toggle') as HTMLInputElement;
const isolatedToggle = document.getElementById('isolated-toggle') as HTMLInputElement;
const labelsLabelEl = document.getElementById('labels-label')!;
const isolatedLabelEl = document.getElementById('isolated-label')!;
const pinbarEl = document.getElementById('pinbar')!;
const pinLabelEl = document.getElementById('pin-label')!;
const zoomInEl = document.getElementById('zoom-in')!;
const zoomOutEl = document.getElementById('zoom-out')!;
const zoomResetEl = document.getElementById('zoom-reset')!;

function drawDarkNodeHover(
  context: CanvasRenderingContext2D,
  data: { x: number; y: number; size: number; color?: string; label?: string | null },
  settings: { labelSize: number; labelFont: string; labelWeight?: string },
): void {
  const size = settings.labelSize;
  context.font = `${settings.labelWeight ?? 'normal'} ${size}px ${settings.labelFont}`;
  const label = data.label ?? '';
  const width = context.measureText(label).width;
  const x = data.x + data.size + 5;
  const y = data.y + size / 3;

  context.beginPath();
  context.roundRect(x - 4, y - size - 2, width + 12, size + 9, 5);
  context.fillStyle = '#1c1f27';
  context.fill();
  context.strokeStyle = '#3a3f4d';
  context.stroke();
  context.fillStyle = '#e8e9ee';
  context.fillText(label, x + 2, y);
  context.beginPath();
  context.arc(data.x, data.y, data.size, 0, Math.PI * 2);
  context.fillStyle = data.color ?? FALLBACK_COLOR;
  context.fill();
}

const renderer = new Sigma(graph, container, {
  defaultEdgeColor: '#3a3f4d',
  // Gentler wheel/trackpad-pinch zoom (Sigma default is 1.7, which feels
  // over-accelerated on a multitouch trackpad). The on-screen buttons use their
  // own explicit step, so they stay snappy regardless.
  zoomingRatio: 1.4,
  labelColor: { color: '#d6d9e0' },
  labelFont: 'system-ui',
  labelSize: 12,
  // R3 — keep labels legible at 100+ nodes: only the most prominent node per
  // grid cell is labeled, and nodes are sized by centrality, so the labels that
  // survive are the important ones. Zooming in reveals more.
  labelRenderedSizeThreshold: 10,
  labelDensity: 0.6,
  labelGridCellSize: 180,
  defaultDrawNodeHover: drawDarkNodeHover,
  nodeReducer(node, data) {
    if (hideIsolated && isolated.has(node)) {
      return { ...data, hidden: true };
    }
    const expanded = node === hovered || node === selected;
    const label = expanded ? (data.fullLabel as string | undefined) ?? data.label : data.label;
    if (thread) {
      if (thread.nodes.has(node)) {
        return { ...data, label, forceLabel: expanded, zIndex: 1, highlighted: node === selected };
      }
      return { ...data, color: DIM_NODE, label: null, zIndex: 0 };
    }
    if (pinned && !pinned.has(node)) {
      return { ...data, color: DIM_NODE, label: null, zIndex: 0 };
    }
    if (focusedCommunity !== null && data.community !== focusedCommunity) {
      return { ...data, color: DIM_NODE, label: null, zIndex: 0 };
    }
    const lifted = focusedCommunity !== null || pinned !== null;
    // Selected/hovered node always shows its label, even under density culling.
    return { ...data, label, forceLabel: expanded, zIndex: lifted ? 1 : 0 };
  },
  edgeReducer(edge, data) {
    if (thread) {
      if (thread.edges.has(edge)) {
        return { ...data, color: THREAD_EDGE, size: (data.size ?? 1) * 2, zIndex: 1 };
      }
      return { ...data, color: DIM_EDGE, zIndex: 0 };
    }
    const [source, target] = graph.extremities(edge);
    if (pinned) {
      const internal = pinned.has(source) && pinned.has(target);
      return internal ? { ...data, zIndex: 1 } : { ...data, color: DIM_EDGE, zIndex: 0 };
    }
    if (focusedCommunity !== null) {
      const internal =
        graph.getNodeAttribute(source, 'community') === focusedCommunity
        && graph.getNodeAttribute(target, 'community') === focusedCommunity;
      return internal ? { ...data, zIndex: 1 } : { ...data, color: DIM_EDGE, zIndex: 0 };
    }
    return data;
  },
});

function fa2Settings() {
  return { ...forceAtlas2.inferSettings(graph), strongGravityMode: true, gravity: 0.5 };
}

/** Recompute Louvain communities over the live graph and recolor nodes.
 * A seeded RNG keeps cluster ids/colors stable across unchanged rebuilds. */
function recolorCommunities(): void {
  const projection = new UndirectedGraph();
  graph.forEachNode((id) => projection.addNode(id));
  graph.forEachEdge((_e, _a, source, target) => {
    if (projection.hasNode(source) && projection.hasNode(target)) {
      projection.mergeEdge(source, target);
    }
  });

  let assignment: Record<string, number>;
  if (projection.size === 0) {
    assignment = {};
    let i = 0;
    graph.forEachNode((id) => { assignment[id] = i++; });
  } else {
    let seed = 0x5eed1234;
    const rng = () => {
      seed ^= seed << 13;
      seed ^= seed >>> 17;
      seed ^= seed << 5;
      return (seed >>> 0) / 0x100000000;
    };
    assignment = louvain(projection, { rng });
  }

  graph.forEachNode((id) => {
    const community = Number(assignment[id] ?? 0);
    graph.setNodeAttribute(id, 'community', community);
    graph.setNodeAttribute(id, 'color', colorForCommunity(community));
  });
}

/** Size nodes by the selected centrality measure, normalized to [MIN,MAX]. */
function applySizing(): void {
  if (graph.order === 0) return;
  let scores: Record<string, number>;
  if (sizingMode === 'betweenness') {
    scores = betweennessCentrality(graph) as Record<string, number>;
  } else if (sizingMode === 'pagerank') {
    scores = pagerank(graph) as Record<string, number>;
  } else {
    scores = {};
    graph.forEachNode((id) => { scores[id] = graph.degree(id); });
  }
  let max = 0;
  for (const v of Object.values(scores)) max = Math.max(max, v);
  graph.forEachNode((id) => {
    const norm = max > 0 ? (scores[id] ?? 0) / max : 0;
    // sqrt keeps low-centrality nodes legible instead of vanishing.
    graph.setNodeAttribute(id, 'size', MIN_SIZE + Math.sqrt(norm) * (MAX_SIZE - MIN_SIZE));
  });
}

/** Recompute the set of isolated nodes (no resolved/rendered edge) so the node
 * reducer can hide them without an O(n) scan per frame (R3). */
function recomputeIsolated(): void {
  isolated = new Set<string>();
  graph.forEachNode((id) => {
    if (graph.degree(id) === 0) isolated.add(id);
  });
}

/** Refresh the "hide isolated" control with the current isolated count. */
function updateViewControls(): void {
  isolatedLabelEl.textContent = `${t('view.hideIsolated')} (${isolated.size})`;
}

function addNodeFromApi(n: ApiNode, x: number, y: number): void {
  graph.addNode(n.id, {
    label: shortLabel(n.title),
    fullLabel: n.title,
    color: FALLBACK_COLOR,
    size: MIN_SIZE,
    x,
    y,
    docType: n.doc_type,
    community: 0,
  });
}

function addEdgeFromApi(e: ApiEdge): void {
  if (e.resolved && graph.hasNode(e.source) && graph.hasNode(e.target)) {
    graph.addEdgeWithKey(String(e.id), e.source, e.target, {
      size: e.edge_type === 'RELATED_TO' ? 1 : 1.6,
      type: 'arrow',
    });
  }
}

function applyGraph(api: ApiGraph): void {
  const positions = new Map<string, { x: number; y: number }>();
  graph.forEachNode((id, attrs) =>
    positions.set(id, { x: attrs.x as number, y: attrs.y as number }),
  );
  graph.clear();

  for (const n of api.nodes) {
    const prev = positions.get(n.id);
    addNodeFromApi(n, prev?.x ?? Math.random(), prev?.y ?? Math.random());
  }
  for (const e of api.edges) addEdgeFromApi(e);

  recolorCommunities();
  if (graph.order > 0) {
    if (positions.size === 0) {
      circular.assign(graph);
      forceAtlas2.assign(graph, { iterations: 300, settings: fa2Settings() });
    } else {
      forceAtlas2.assign(graph, { iterations: 30, settings: fa2Settings() });
    }
  }
  applySizing();

  if (selected && !graph.hasNode(selected)) clearSelection();
  if (pinned) reconcilePin();
  recomputeIsolated();
  renderCounts(api.stats);
  renderLegend();
  renderStats(api.stats);
  updateViewControls();
  renderer.refresh();
}

/** Patch the live graph from an incremental delta (T3.1), preserving the
 * layout of unchanged nodes. */
function applyDelta(delta: GraphDelta): void {
  for (const id of delta.removed) {
    if (graph.hasNode(id)) graph.dropNode(id);
  }

  const fresh: string[] = [];
  for (const n of delta.added) {
    if (graph.hasNode(n.id)) graph.dropNode(n.id);
    addNodeFromApi(n, Math.random(), Math.random());
    fresh.push(n.id);
  }
  for (const n of delta.changed) {
    if (graph.hasNode(n.id)) {
      graph.mergeNodeAttributes(n.id, {
        label: shortLabel(n.title),
        fullLabel: n.title,
        docType: n.doc_type,
      });
    } else {
      addNodeFromApi(n, Math.random(), Math.random());
      fresh.push(n.id);
    }
  }

  // Edges are positional ids → replace wholesale (cheap, no relayout cost).
  graph.clearEdges();
  for (const e of delta.edges) addEdgeFromApi(e);

  recolorCommunities();
  for (const id of fresh) positionNearNeighbors(id);
  if (delta.added.length || delta.removed.length) {
    forceAtlas2.assign(graph, { iterations: 30, settings: fa2Settings() });
  }
  applySizing();

  if (selected && !graph.hasNode(selected)) clearSelection();
  if (pinned) reconcilePin();
  recomputeIsolated();
  renderCounts(delta.stats);
  renderLegend();
  renderStats(delta.stats);
  updateViewControls();
  renderer.refresh();
}

function positionNearNeighbors(id: string): void {
  let x = 0;
  let y = 0;
  let count = 0;
  graph.forEachNeighbor(id, (neighbor) => {
    x += graph.getNodeAttribute(neighbor, 'x') as number;
    y += graph.getNodeAttribute(neighbor, 'y') as number;
    count += 1;
  });
  if (count > 0) {
    graph.setNodeAttribute(id, 'x', x / count + (Math.random() - 0.5) * 0.1);
    graph.setNodeAttribute(id, 'y', y / count + (Math.random() - 0.5) * 0.1);
  }
}

function renderCounts(stats: ApiStats): void {
  countsEl.textContent = t('counts', { docs: stats.total_docs, links: stats.total_edges });
}

function renderLegend(): void {
  const communities = new Map<number, Array<{ title: string; degree: number }>>();
  graph.forEachNode((id, attrs) => {
    const community = attrs.community as number;
    const members = communities.get(community) ?? [];
    members.push({ title: attrs.fullLabel as string, degree: graph.degree(id) });
    communities.set(community, members);
  });
  const ranked = [...communities.entries()].sort((a, b) => b[1].length - a[1].length);
  const singletons = ranked.filter(([, members]) => members.length === 1).length;
  const main = ranked
    .filter(([, members]) => members.length >= 2)
    .slice(0, MAX_LEGEND_COMMUNITIES);
  const hidden = ranked.length - singletons - main.length;

  const buttons = main.map(([community, members]) => {
    const representative = [...members].sort((a, b) => b.degree - a.degree)[0].title;
    const active = community === focusedCommunity ? ' active' : '';
    return `<button type="button" class="community${active}" data-community="${community}"
      title="${escapeHtml(t('legend.focus', { n: members.length }))}">
      <i style="background:${colorForCommunity(community)}"></i>
      <span>${escapeHtml(shortLabel(representative))}</span><b>${members.length}</b>
    </button>`;
  }).join('');

  legendEl.innerHTML = `
    <div class="legend-summary">
      <b>${communities.size}</b> ${escapeHtml(t('legend.communities'))} · <b>${singletons}</b> ${escapeHtml(t('legend.isolated'))}
      ${hidden > 0 ? ` · ${escapeHtml(t('legend.hidden', { n: hidden }))}` : ''}
    </div>
    <div class="community-list">${buttons}</div>`;
  // Click handling is delegated on the stable #legend container (see below),
  // so it survives the innerHTML rewrite this function performs.
}

function focusCommunity(community: number): void {
  const nextCommunity = focusedCommunity === community ? null : community;
  clearSelection();
  focusedCommunity = nextCommunity;
  legendEl.querySelectorAll<HTMLElement>('[data-community]').forEach((button) => {
    button.classList.toggle(
      'active',
      nextCommunity !== null && Number(button.dataset.community) === nextCommunity,
    );
  });
  renderer.refresh();
}

function countRows(values: Record<string, number>): string {
  return Object.entries(values)
    .sort((a, b) => b[1] - a[1])
    .map(([label, count]) => `<span>${escapeHtml(label)} <b>${count}</b></span>`)
    .join('');
}

function nodeButton(id: string): string {
  return `<button type="button" class="node-link" data-node="${escapeHtml(id)}">${escapeHtml(id)}</button>`;
}

function bindNodeLinks(root: HTMLElement): void {
  root.querySelectorAll<HTMLElement>('[data-node]').forEach((button) => {
    button.addEventListener('click', (event) => {
      event.preventDefault();
      event.stopPropagation();
      void focusNode(button.dataset.node!);
    });
  });
}

function renderStats(stats: ApiStats): void {
  const broken = stats.broken_links
    .map((edge) => `<li>${nodeButton(edge.source)} → <span class="dangling">${escapeHtml(edge.target)}</span></li>`)
    .join('');
  const cycles = stats.cycles
    .map((cycle) => `<li class="cycle">${cycle.node_ids.map(nodeButton).join(' ⟲ ')}</li>`)
    .join('');
  statsEl.innerHTML = `
    <h2>${escapeHtml(t('stats.title'))}</h2>
    <div class="stat-total"><b>${stats.total_docs}</b> ${escapeHtml(t('stats.docs'))} <b>${stats.total_edges}</b> ${escapeHtml(t('stats.links'))}</div>
    <h3>${escapeHtml(t('stats.types'))}</h3><div class="chips">${countRows(stats.by_type)}</div>
    <h3>${escapeHtml(t('stats.status'))}</h3><div class="chips">${countRows(stats.by_status)}</div>
    <h3>${escapeHtml(t('stats.risk'))}</h3><div class="chips">${countRows(stats.by_risk)}</div>
    ${stats.cycles.length ? `<details data-section="cycles"${openStatsSections.has('cycles') ? ' open' : ''}>
      <summary class="cycles">${escapeHtml(t('stats.cycles'))} (${stats.cycles.length})</summary>
      <ul>${cycles}</ul>
    </details>` : ''}
    <details data-section="orphans"${openStatsSections.has('orphans') ? ' open' : ''}>
      <summary>${escapeHtml(t('stats.orphans'))} (${stats.orphans.length})</summary>
      <ul>${stats.orphans.map((id) => `<li>${nodeButton(id)}</li>`).join('')}</ul>
    </details>
    <details data-section="dangling"${openStatsSections.has('dangling') ? ' open' : ''}>
      <summary>${escapeHtml(t('stats.broken'))} (${stats.broken_links.length})</summary>
      <ul>${broken}</ul>
    </details>
    <div class="stat-refs">${escapeHtml(t('stats.fileRefs'))}: ${stats.file_references} · ${escapeHtml(t('stats.externalRefs'))}: ${stats.external_links}</div>`;
  // Click handling is delegated on the stable #stats container (see below), so
  // it survives the innerHTML rewrite this function performs on every rebuild.
}

async function select(nodeId: string): Promise<void> {
  selected = nodeId;
  const [threadRes, nodeRes] = await Promise.all([
    fetch(`/api/node/${encodeURIComponent(nodeId)}/thread`),
    fetch(`/api/node/${encodeURIComponent(nodeId)}`),
  ]);
  if (!threadRes.ok) return;
  const result: Thread = await threadRes.json();
  thread = { nodes: new Set(result.node_ids), edges: new Set(result.edge_ids.map(String)) };
  if (nodeRes.ok) showDetail(await nodeRes.json());
  renderer.refresh();
}

function clearSelection(): void {
  selected = null;
  thread = null;
  selectionEl.classList.remove('open');
  renderer.refresh();
}

function edgeLinks(edges: ApiEdge[], direction: 'in' | 'out'): string {
  if (edges.length === 0) return `<span class="muted">${escapeHtml(t('detail.none'))}</span>`;
  return edges.map((edge) => {
    const endpoint = direction === 'in' ? edge.source : edge.target;
    return edge.resolved
      ? nodeButton(endpoint)
      : `<span class="dangling">${escapeHtml(endpoint)} (${escapeHtml(t('detail.dangling'))})</span>`;
  }).join('');
}

function showDetail(detail: {
  node: ApiNode;
  excerpt: string | null;
  excerpt_truncated: boolean;
  in_edges: ApiEdge[];
  out_edges: ApiEdge[];
}): void {
  const n = detail.node;
  const fileUri = encodeURI(n.path.startsWith('/') ? n.path : `/${n.path}`);
  selectionEl.innerHTML = `
    <button type="button" class="close" aria-label="Close">×</button>
    <div class="doc-type" style="color:${TYPE_COLORS[n.doc_type] ?? FALLBACK_COLOR}">${n.doc_type}</div>
    <h2>${escapeHtml(n.title)}</h2>
    <div class="meta">${escapeHtml(n.id)}<br>
      ${escapeHtml(t('detail.status'))}: ${escapeHtml(n.status)} · ${escapeHtml(t('detail.risk'))}: ${escapeHtml(n.risk_level)}
      ${n.created ? ` · ${escapeHtml(n.created)}` : ''}
      ${n.tags.length ? `<br>${escapeHtml(t('detail.tags'))}: ${n.tags.map(escapeHtml).join(', ')}` : ''}
      <br><span class="path">${escapeHtml(n.path)}</span>
    </div>
    <div class="actions">
      <button type="button" class="pin">${escapeHtml(t('detail.pin'))}</button>
      <a class="open" href="vscode://file${fileUri}">${escapeHtml(t('detail.openVscode'))}</a>
      <a class="open" href="cursor://file${fileUri}">${escapeHtml(t('detail.openCursor'))}</a>
      <button type="button" class="copy" data-path="${escapeHtml(n.path)}">${escapeHtml(t('detail.copyPath'))}</button>
    </div>
    <h3>${escapeHtml(t('detail.outgoing'))} (${detail.out_edges.length})</h3><div class="links">${edgeLinks(detail.out_edges, 'out')}</div>
    <h3>${escapeHtml(t('detail.incoming'))} (${detail.in_edges.length})</h3><div class="links">${edgeLinks(detail.in_edges, 'in')}</div>
    <h3>${escapeHtml(t('detail.excerpt'))}</h3>
    <p class="excerpt">${escapeHtml(detail.excerpt ?? '')}</p>
    ${detail.excerpt_truncated
      ? `<div class="excerpt-note">${escapeHtml(t('detail.excerptNote'))}</div>`
      : ''}`;
  selectionEl.classList.add('open');
  bindNodeLinks(selectionEl);
  selectionEl.querySelector<HTMLElement>('.close')?.addEventListener('click', (event) => {
    event.preventDefault();
    event.stopPropagation();
    clearSelection();
  });
  selectionEl.querySelector<HTMLElement>('.pin')?.addEventListener('click', (event) => {
    event.preventDefault();
    event.stopPropagation();
    pinThread();
  });
  selectionEl.querySelector<HTMLElement>('.copy')?.addEventListener('click', (event) => {
    event.preventDefault();
    event.stopPropagation();
    const button = event.currentTarget as HTMLButtonElement;
    void navigator.clipboard?.writeText(button.dataset.path ?? '').then(() => {
      button.textContent = t('detail.copied');
      setTimeout(() => { button.textContent = t('detail.copyPath'); }, 1200);
    });
  });
}

/** Pin the current thread as an isolated working set (T3.4). */
function pinThread(): void {
  const members = thread ? [...thread.nodes] : selected ? [selected] : [];
  if (members.length === 0) return;
  pinned = new Set(members);
  clearSelection();
  updatePinbar();
  renderer.refresh();
}

/** Drop nodes that no longer exist after a rebuild; clear the pin if empty. */
function reconcilePin(): void {
  if (!pinned) return;
  for (const id of [...pinned]) if (!graph.hasNode(id)) pinned.delete(id);
  if (pinned.size === 0) pinned = null;
  updatePinbar();
}

function updatePinbar(): void {
  if (pinned) {
    pinLabelEl.textContent = t('pin.label', { n: pinned.size });
    pinbarEl.classList.add('open');
  } else {
    pinbarEl.classList.remove('open');
  }
}

function filterQuery(): string {
  const params = new URLSearchParams();
  const data = new FormData(filterForm);
  for (const [key, value] of data) {
    const trimmed = String(value).trim();
    if (trimmed) params.set(key, trimmed);
  }
  return params.toString();
}

async function loadFilteredGraph(): Promise<void> {
  const query = filterQuery();
  const response = await fetch(`/api/graph${query ? `?${query}` : ''}`);
  if (response.ok) {
    focusedCommunity = null;
    applyGraph(await response.json());
  }
}

async function focusNode(id: string): Promise<void> {
  if (!graph.hasNode(id)) {
    filterForm.reset();
    await loadFilteredGraph();
  }
  if (graph.hasNode(id)) {
    await select(id);
    focusCamera(id);
  }
}

function focusCamera(id: string): void {
  if (!graph.hasNode(id)) return;
  const display = renderer.getNodeDisplayData(id);
  if (display) {
    renderer.getCamera().animate({ x: display.x, y: display.y, ratio: 0.4 }, { duration: 450 });
  }
}

function runSearch(): void {
  const query = searchEl.value.trim().toLowerCase();
  if (!query) {
    searchResultsEl.classList.remove('open');
    searchResultsEl.innerHTML = '';
    return;
  }
  const matches: Array<{ id: string; title: string }> = [];
  graph.forEachNode((id, attrs) => {
    if (matches.length >= 8) return;
    const title = (attrs.fullLabel as string) ?? id;
    if (id.toLowerCase().includes(query) || title.toLowerCase().includes(query)) {
      matches.push({ id, title });
    }
  });
  searchResultsEl.innerHTML = matches.length
    ? matches.map((m) => `<button type="button" data-node="${escapeHtml(m.id)}">${escapeHtml(m.title)}</button>`).join('')
    : `<div class="empty">${escapeHtml(t('search.none'))}</div>`;
  searchResultsEl.classList.add('open');
  searchResultsEl.querySelectorAll<HTMLElement>('[data-node]').forEach((button) => {
    button.addEventListener('click', () => {
      searchResultsEl.classList.remove('open');
      searchEl.value = '';
      void focusNode(button.dataset.node!);
    });
  });
}

function applyStaticI18n(): void {
  document.getElementById('subtitle')!.textContent =
    currentView === 'plan' ? t('plan.subtitle') : t('app.subtitle');
  document.getElementById('badge')!.textContent = t('app.experimental');
  document.getElementById('tab-kg')!.textContent = t('tab.kg');
  document.getElementById('tab-plan')!.textContent = t('tab.plan');
  document.getElementById('axon-explode-label')!.textContent = t('axon.explode');
  buildPlanLegend();
  searchEl.placeholder = t('search.placeholder');
  document.getElementById('sizing-label')!.textContent = t('size.label');
  sizingEl.options[0].text = t('size.betweenness');
  sizingEl.options[1].text = t('size.pagerank');
  sizingEl.options[2].text = t('size.degree');
  const placeholder = (name: string, key: string) => {
    const input = filterForm.querySelector<HTMLInputElement>(`[name="${name}"]`);
    if (input) input.placeholder = t(key);
  };
  placeholder('type', 'filter.type');
  placeholder('status', 'filter.status');
  placeholder('risk', 'filter.risk');
  placeholder('tag', 'filter.tag');
  document.getElementById('filter-submit')!.textContent = t('filter.submit');
  document.getElementById('filter-clear')!.textContent = t('filter.clear');
  document.getElementById('pin-clear')!.textContent = t('pin.unpin');
  labelsLabelEl.textContent = t('view.labels');
  for (const [el, key] of [[zoomInEl, 'zoom.in'], [zoomOutEl, 'zoom.out'], [zoomResetEl, 'zoom.reset']] as const) {
    el.title = t(key);
    el.setAttribute('aria-label', t(key));
  }
  updateViewControls();
  statusEl.textContent = t('status.connecting');
}

renderer.on('clickNode', ({ node }) => void select(node));
renderer.on('clickStage', clearSelection);
renderer.on('enterNode', ({ node }) => { hovered = node; renderer.refresh(); });
renderer.on('leaveNode', () => { hovered = null; renderer.refresh(); });

// Delegated click handling on the stable panel containers. renderStats /
// renderLegend rewrite their innerHTML on every rebuild, so per-button listeners
// would be destroyed mid-interaction (swallowing clicks under frequent updates).
// The container elements persist, so one listener each stays valid.
statsEl.addEventListener('click', (event) => {
  const el = event.target as HTMLElement;
  const summary = el.closest('summary');
  if (summary && statsEl.contains(summary)) {
    const details = summary.parentElement as HTMLDetailsElement;
    const section = details.dataset.section;
    if (section) {
      // The native <details> toggle runs after this handler; mirror its result
      // into the persisted open-section set so a later re-render restores it.
      if (details.open) openStatsSections.delete(section);
      else openStatsSections.add(section);
    }
    return; // let the browser toggle the <details> natively
  }
  const link = el.closest<HTMLElement>('[data-node]');
  if (link) {
    event.preventDefault();
    void focusNode(link.dataset.node!);
  }
});

legendEl.addEventListener('click', (event) => {
  const button = (event.target as HTMLElement).closest<HTMLElement>('[data-community]');
  if (button) {
    event.preventDefault();
    focusCommunity(Number(button.dataset.community));
  }
});

filterForm.addEventListener('submit', (event) => {
  event.preventDefault();
  void loadFilteredGraph();
});
filterForm.addEventListener('reset', () => setTimeout(() => void loadFilteredGraph(), 0));

searchEl.addEventListener('input', runSearch);
searchEl.addEventListener('keydown', (event) => {
  if (event.key === 'Enter') {
    event.preventDefault();
    searchResultsEl.querySelector<HTMLElement>('[data-node]')?.click();
  }
});

sizingEl.addEventListener('change', () => {
  sizingMode = sizingEl.value as SizingMode;
  applySizing();
  renderer.refresh();
});

labelsToggle.addEventListener('change', () => {
  showLabels = labelsToggle.checked;
  renderer.setSetting('renderLabels', showLabels);
});

isolatedToggle.addEventListener('change', () => {
  hideIsolated = isolatedToggle.checked;
  renderer.refresh();
});

document.getElementById('pin-clear')!.addEventListener('click', () => {
  pinned = null;
  updatePinbar();
  renderer.refresh();
});

// On-screen zoom / center controls — precise alternative to trackpad pinch.
const ZOOM_STEP = 1.6;
zoomInEl.addEventListener('click', () => {
  void renderer.getCamera().animatedZoom({ duration: 250, factor: ZOOM_STEP });
});
zoomOutEl.addEventListener('click', () => {
  void renderer.getCamera().animatedUnzoom({ duration: 250, factor: ZOOM_STEP });
});
zoomResetEl.addEventListener('click', () => {
  void renderer.getCamera().animatedReset({ duration: 350 });
});

// ── View switch: Knowledge Graph ⇄ Architecture Plan (A2.3) ──
type View = 'kg' | 'plan';
type PlanMode = '2d' | '3d';
let currentView: View = 'kg';
let planMode: PlanMode = '2d';
const planEl = document.getElementById('plan')!;
const axonEl = document.getElementById('axon')!;
const planLegendEl = document.getElementById('plan-legend')!;
const viewTabs = Array.from(document.querySelectorAll<HTMLButtonElement>('#view-tabs button'));
const planModeBtns = Array.from(document.querySelectorAll<HTMLButtonElement>('#plan-mode button'));

/** Render the Architecture view in the active projection (2D plan / 3D axon). */
function renderArchitecture(): void {
  if (planMode === '3d') void renderAxon(axonEl);
  else void renderPlan(planEl);
}

function setView(view: View): void {
  if (view === currentView) return;
  currentView = view;
  document.body.classList.toggle('view-plan', view === 'plan');
  for (const b of viewTabs) b.classList.toggle('active', b.dataset.view === view);
  document.getElementById('subtitle')!.textContent =
    view === 'plan' ? t('plan.subtitle') : t('app.subtitle');
  // Deep-link the active view so the plan is shareable / reloadable.
  const hash = view === 'plan' ? '#architecture' : '';
  if (location.hash !== hash) {
    history.replaceState(null, '', hash || location.pathname + location.search);
  }
  if (view === 'plan') {
    renderArchitecture();
  } else {
    disposeAxon(); // free the WebGL context when leaving the 3D view
    // The #graph canvas was display:none while in the plan view, so Sigma never
    // saw the resize back to full size — it renders blank until a zoom nudges
    // it. Force a resize + refresh on the next frame (after the reflow).
    requestAnimationFrame(() => {
      renderer.resize(true);
      renderer.refresh();
    });
  }
}

/** Switch the Architecture view between the 2D plan and the 3D axonometric. */
function setPlanMode(mode: PlanMode): void {
  if (mode === planMode) return;
  planMode = mode;
  document.body.classList.toggle('plan-3d', mode === '3d');
  for (const b of planModeBtns) b.classList.toggle('active', b.dataset.mode === mode);
  if (mode === '2d') disposeAxon(); // leaving 3D frees the GL context
  if (currentView === 'plan') renderArchitecture();
}

for (const b of planModeBtns) {
  b.addEventListener('click', () => setPlanMode((b.dataset.mode as PlanMode) ?? '2d'));
}

const axonExplode = document.getElementById('axon-explode-range') as HTMLInputElement | null;
axonExplode?.addEventListener('input', () => setExplode(Number(axonExplode.value) / 100));

/** The color legend for the plan's status overlay (localized). */
function buildPlanLegend(): void {
  planLegendEl.innerHTML =
    `<div class="pl-title">${t('plan.legend')}</div>` +
    LEGEND_STATES.map((s) => {
      const c = stateColor(s);
      return `<div class="pl-row"><span class="pl-sw" style="background:${c.fill};border-color:${c.stroke}"></span>${t('plan.state.' + s)}</div>`;
    }).join('');
}

for (const b of viewTabs) {
  b.addEventListener('click', () => setView((b.dataset.view as View) ?? 'kg'));
}

// Cross-view bridge (FR8/§8): clicking a Charter in the plan's component panel
// switches to the knowledge graph and selects that node (when present).
setPlanHooks({
  openNode: (nodeId: string) => {
    setView('kg');
    if (graph.hasNode(nodeId)) void select(nodeId);
  },
});

// Deep-link the active view synchronously (independent of the async boot, so a
// `#architecture` reload opens the plan immediately). `setView` falls back to
// the default locale until `boot()` resolves the project locale.
if (location.hash === '#architecture') setView('plan');
window.addEventListener('hashchange', () => {
  setView(location.hash === '#architecture' ? 'plan' : 'kg');
});

function connect(): void {
  const proto = location.protocol === 'https:' ? 'wss' : 'ws';
  const ws = new WebSocket(`${proto}://${location.host}/api/stream`);
  ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    statusEl.textContent = t('status.live');
    statusEl.className = 'live';
    // Architecture overlay deltas (A2.2) are independent of the KG filter.
    if (msg.event === 'architecture') {
      if (currentView === 'plan') renderArchitecture();
      return;
    }
    if (filterQuery()) {
      // Under an active filter the unfiltered delta/rebuild is ambiguous;
      // refetch the filtered view.
      void loadFilteredGraph();
    } else if (msg.event === 'rebuild') {
      applyGraph(msg.graph as ApiGraph);
    } else if (msg.event === 'delta') {
      applyDelta(msg as GraphDelta);
    }
  };
  ws.onclose = () => {
    statusEl.textContent = t('status.reconnecting');
    statusEl.className = 'down';
    setTimeout(connect, 1500);
  };
}

async function boot(): Promise<void> {
  try {
    const meta = await fetch('/api/meta');
    if (meta.ok) setLocale((await meta.json()).locale ?? 'en');
  } catch {
    setLocale('en');
  }
  sizingEl.value = sizingMode;
  applyStaticI18n();
  connect();
}

void boot();
