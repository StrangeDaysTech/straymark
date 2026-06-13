// Loom M2 — analytics + panels (Spec 001 FR8–FR10).

import Graph, { UndirectedGraph } from 'graphology';
import louvain from 'graphology-communities-louvain';
import { circular } from 'graphology-layout';
import forceAtlas2 from 'graphology-layout-forceatlas2';
import Sigma from 'sigma';

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

interface ApiStats {
  total_docs: number;
  total_edges: number;
  by_type: Record<string, number>;
  by_status: Record<string, number>;
  by_risk: Record<string, number>;
  orphans: string[];
  dangling_references: ApiEdge[];
}

interface ApiGraph {
  nodes: ApiNode[];
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
};
const FALLBACK_COLOR = '#8b91a0';
const DIM_NODE = '#2c303b';
const DIM_EDGE = '#1b1e25';
const THREAD_EDGE = '#e8833a';
const SHORT_LABEL_CHARS = 42;
const MAX_LEGEND_COMMUNITIES = 8;

const colorForCommunity = (community: number) =>
  COMMUNITY_COLORS[community % COMMUNITY_COLORS.length];
const sizeFor = (n: ApiNode) => 4 + Math.min(10, Math.sqrt(n.degree_in + n.degree_out) * 2.5);
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
const openStatsSections = new Set<string>();

const container = document.getElementById('graph')!;
const countsEl = document.getElementById('counts')!;
const statusEl = document.getElementById('status')!;
const selectionEl = document.getElementById('selection')!;
const legendEl = document.getElementById('legend')!;
const statsEl = document.getElementById('stats')!;
const filterForm = document.getElementById('filters') as HTMLFormElement;

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
  labelColor: { color: '#d6d9e0' },
  labelFont: 'system-ui',
  labelSize: 12,
  labelRenderedSizeThreshold: 8,
  defaultDrawNodeHover: drawDarkNodeHover,
  nodeReducer(node, data) {
    const expanded = node === hovered || node === selected;
    const label = expanded ? (data.fullLabel as string | undefined) ?? data.label : data.label;
    if (thread) {
      if (thread.nodes.has(node)) {
        return { ...data, label, zIndex: 1, highlighted: node === selected };
      }
      return { ...data, color: DIM_NODE, label: null, zIndex: 0 };
    }
    if (focusedCommunity !== null && data.community !== focusedCommunity) {
      return { ...data, color: DIM_NODE, label: null, zIndex: 0 };
    }
    return { ...data, label, zIndex: focusedCommunity === null ? 0 : 1 };
  },
  edgeReducer(edge, data) {
    if (thread) {
      if (thread.edges.has(edge)) {
        return { ...data, color: THREAD_EDGE, size: (data.size ?? 1) * 2, zIndex: 1 };
      }
      return { ...data, color: DIM_EDGE, zIndex: 0 };
    }
    if (focusedCommunity !== null) {
      const [source, target] = graph.extremities(edge);
      const internal =
        graph.getNodeAttribute(source, 'community') === focusedCommunity
        && graph.getNodeAttribute(target, 'community') === focusedCommunity;
      return internal ? { ...data, zIndex: 1 } : { ...data, color: DIM_EDGE, zIndex: 0 };
    }
    return data;
  },
});

function detectCommunities(api: ApiGraph): Record<string, number> {
  const projection = new UndirectedGraph();
  for (const node of api.nodes) projection.addNode(node.id);
  for (const edge of api.edges) {
    if (edge.resolved && projection.hasNode(edge.source) && projection.hasNode(edge.target)) {
      projection.mergeEdge(edge.source, edge.target);
    }
  }
  if (projection.size === 0) {
    return Object.fromEntries(api.nodes.map((node, index) => [node.id, index]));
  }
  // Keep cluster ids/colors stable across full rebuilds of unchanged data.
  let seed = 0x5eed1234;
  const rng = () => {
    seed ^= seed << 13;
    seed ^= seed >>> 17;
    seed ^= seed << 5;
    return (seed >>> 0) / 0x100000000;
  };
  return louvain(projection, { rng });
}

function applyGraph(api: ApiGraph): void {
  const positions = new Map<string, { x: number; y: number }>();
  graph.forEachNode((id, attrs) =>
    positions.set(id, { x: attrs.x as number, y: attrs.y as number }),
  );
  graph.clear();

  const communities = detectCommunities(api);
  for (const n of api.nodes) {
    // Louvain's runtime mapping can expose numeric-looking string values
    // despite its TypeScript declaration. Normalize once so focus/legend
    // comparisons remain stable.
    const community = Number(communities[n.id] ?? 0);
    graph.addNode(n.id, {
      label: shortLabel(n.title),
      fullLabel: n.title,
      color: colorForCommunity(community),
      size: sizeFor(n),
      x: positions.get(n.id)?.x ?? Math.random(),
      y: positions.get(n.id)?.y ?? Math.random(),
      docType: n.doc_type,
      community,
    });
  }
  for (const e of api.edges) {
    if (e.resolved && graph.hasNode(e.source) && graph.hasNode(e.target)) {
      graph.addEdgeWithKey(String(e.id), e.source, e.target, {
        size: e.edge_type === 'RELATED_TO' ? 1 : 1.6,
        type: 'arrow',
      });
    }
  }

  const fa2Settings = {
    ...forceAtlas2.inferSettings(graph),
    strongGravityMode: true,
    gravity: 0.5,
  };
  if (graph.order > 0) {
    if (positions.size === 0) {
      circular.assign(graph);
      forceAtlas2.assign(graph, { iterations: 300, settings: fa2Settings });
    } else {
      forceAtlas2.assign(graph, { iterations: 30, settings: fa2Settings });
    }
  }

  countsEl.textContent = `${api.stats.total_docs} docs · ${api.stats.total_edges} links`;
  renderLegend();
  renderStats(api.stats);
  if (selected && !graph.hasNode(selected)) clearSelection();
  renderer.refresh();
}

function renderLegend(): void {
  const communities = new Map<number, Array<{ title: string; degree: number }>>();
  graph.forEachNode((_id, attrs) => {
    const community = attrs.community as number;
    const members = communities.get(community) ?? [];
    members.push({
      title: attrs.fullLabel as string,
      degree: graph.degree(_id),
    });
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
      title="Focus ${members.length} documents in this community">
      <i style="background:${colorForCommunity(community)}"></i>
      <span>${escapeHtml(shortLabel(representative))}</span><b>${members.length}</b>
    </button>`;
  }).join('');

  legendEl.innerHTML = `
    <div class="legend-summary">
      <b>${communities.size}</b> communities · <b>${singletons}</b> isolated
      ${hidden > 0 ? ` · ${hidden} smaller hidden` : ''}
    </div>
    <div class="community-list">${buttons}</div>`;
  legendEl.querySelectorAll<HTMLElement>('[data-community]').forEach((button) => {
    button.addEventListener('click', (event) => {
      event.preventDefault();
      event.stopPropagation();
      focusCommunity(Number(button.dataset.community));
    });
  });
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

function bindNodeLinks(container: HTMLElement): void {
  container.querySelectorAll<HTMLElement>('[data-node]').forEach((button) => {
    button.addEventListener('click', (event) => {
      event.preventDefault();
      event.stopPropagation();
      void focusNode(button.dataset.node!);
    });
  });
}

function renderStats(stats: ApiStats): void {
  const dangling = stats.dangling_references
    .map((edge) => `<li>${nodeButton(edge.source)} → <span>${escapeHtml(edge.target)}</span></li>`)
    .join('');
  statsEl.innerHTML = `
    <h2>Corpus stats</h2>
    <div class="stat-total"><b>${stats.total_docs}</b> docs <b>${stats.total_edges}</b> links</div>
    <h3>Types</h3><div class="chips">${countRows(stats.by_type)}</div>
    <h3>Status</h3><div class="chips">${countRows(stats.by_status)}</div>
    <h3>Risk</h3><div class="chips">${countRows(stats.by_risk)}</div>
    <details data-section="orphans"${openStatsSections.has('orphans') ? ' open' : ''}>
      <summary>Orphans (${stats.orphans.length})</summary>
      <ul>${stats.orphans.map((id) => `<li>${nodeButton(id)}</li>`).join('')}</ul>
    </details>
    <details data-section="dangling"${openStatsSections.has('dangling') ? ' open' : ''}>
      <summary>Dangling references (${stats.dangling_references.length})</summary>
      <ul>${dangling}</ul>
    </details>`;
  bindNodeLinks(statsEl);
  statsEl.querySelectorAll<HTMLElement>('details > summary').forEach((summary) => {
    summary.addEventListener('click', (event) => {
      event.preventDefault();
      event.stopPropagation();
      const details = summary.parentElement as HTMLDetailsElement;
      details.open = !details.open;
      const section = details.dataset.section!;
      if (details.open) openStatsSections.add(section);
      else openStatsSections.delete(section);
    });
  });
}

async function select(nodeId: string): Promise<void> {
  selected = nodeId;
  const [threadRes, nodeRes] = await Promise.all([
    fetch(`/api/node/${encodeURIComponent(nodeId)}/thread`),
    fetch(`/api/node/${encodeURIComponent(nodeId)}`),
  ]);
  if (!threadRes.ok) return;
  const t: Thread = await threadRes.json();
  thread = { nodes: new Set(t.node_ids), edges: new Set(t.edge_ids.map(String)) };
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
  if (edges.length === 0) return '<span class="muted">none</span>';
  return edges.map((edge) => {
    const endpoint = direction === 'in' ? edge.source : edge.target;
    return edge.resolved
      ? nodeButton(endpoint)
      : `<span class="dangling">${escapeHtml(endpoint)} (dangling)</span>`;
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
  selectionEl.innerHTML = `
    <button type="button" class="close" aria-label="Close">×</button>
    <div class="doc-type" style="color:${TYPE_COLORS[n.doc_type] ?? FALLBACK_COLOR}">${n.doc_type}</div>
    <h2>${escapeHtml(n.title)}</h2>
    <div class="meta">${escapeHtml(n.id)}<br>
      status: ${escapeHtml(n.status)} · risk: ${escapeHtml(n.risk_level)}
      ${n.created ? ` · ${escapeHtml(n.created)}` : ''}
      ${n.tags.length ? `<br>tags: ${n.tags.map(escapeHtml).join(', ')}` : ''}
      <br><span class="path">${escapeHtml(n.path)}</span>
    </div>
    <h3>Outgoing (${detail.out_edges.length})</h3><div class="links">${edgeLinks(detail.out_edges, 'out')}</div>
    <h3>Incoming (${detail.in_edges.length})</h3><div class="links">${edgeLinks(detail.in_edges, 'in')}</div>
    <h3>Excerpt</h3>
    <p class="excerpt">${escapeHtml(detail.excerpt ?? '')}</p>
    ${detail.excerpt_truncated
      ? '<div class="excerpt-note">Excerpt truncated · full document reading arrives in M3</div>'
      : ''}`;
  selectionEl.classList.add('open');
  bindNodeLinks(selectionEl);
  selectionEl.querySelector<HTMLElement>('.close')?.addEventListener('click', (event) => {
    event.preventDefault();
    event.stopPropagation();
    clearSelection();
  });
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
    // Filter changes alter the graph projection and therefore community ids.
    focusedCommunity = null;
    applyGraph(await response.json());
  }
}

async function focusNode(id: string): Promise<void> {
  if (!graph.hasNode(id)) {
    filterForm.reset();
    await loadFilteredGraph();
  }
  if (graph.hasNode(id)) await select(id);
}

renderer.on('clickNode', ({ node }) => void select(node));
renderer.on('clickStage', clearSelection);
renderer.on('enterNode', ({ node }) => { hovered = node; renderer.refresh(); });
renderer.on('leaveNode', () => { hovered = null; renderer.refresh(); });

filterForm.addEventListener('submit', (event) => {
  event.preventDefault();
  void loadFilteredGraph();
});
filterForm.addEventListener('reset', () => setTimeout(() => void loadFilteredGraph(), 0));

function connect(): void {
  const proto = location.protocol === 'https:' ? 'wss' : 'ws';
  const ws = new WebSocket(`${proto}://${location.host}/api/stream`);
  ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    if (msg.event === 'rebuild') {
      statusEl.textContent = 'live';
      statusEl.className = 'live';
      if (filterQuery()) void loadFilteredGraph();
      else applyGraph(msg.graph as ApiGraph);
    }
  };
  ws.onclose = () => {
    statusEl.textContent = 'reconnecting…';
    statusEl.className = 'down';
    setTimeout(connect, 1500);
  };
}

connect();
