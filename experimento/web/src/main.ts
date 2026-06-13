// Loom M1 — walking skeleton (Spec 001 FR4–FR6):
// force-directed graph colored by doc_type, sized by degree, thread
// highlighting via Sigma reducers, live rebuilds over WebSocket.

import Graph from 'graphology';
import { circular } from 'graphology-layout';
import forceAtlas2 from 'graphology-layout-forceatlas2';
import Sigma from 'sigma';

// ---- API types (mirror experimento/src/snapshot.rs) -----------------------

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

// ---- Visual encoding -------------------------------------------------------

const TYPE_COLORS: Record<string, string> = {
  AILOG: '#e8833a', // Loom orange — the most common doc, the project heartbeat
  AIDEC: '#d4a93c',
  ADR: '#5d9cec',
  ETH: '#9b7fd4',
  REQ: '#4caf78',
  TES: '#3cbfb4',
  INC: '#c75050',
  TDE: '#b06a4f',
  SEC: '#d4607f',
  MCARD: '#7f9ad4',
  SBOM: '#8aa455',
  DPIA: '#b07fd4',
  PIPIA: '#c08fb0',
  CACFILE: '#a0788f',
  TC260RA: '#90a0b8',
  AILABEL: '#74b89a',
};
const FALLBACK_COLOR = '#8b91a0';
const DIM_NODE = '#2c303b';
const DIM_EDGE = '#1b1e25';
const THREAD_EDGE = '#e8833a'; // Loom accent — the lit thread

const colorFor = (docType: string) => TYPE_COLORS[docType] ?? FALLBACK_COLOR;
const sizeFor = (n: ApiNode) => 4 + Math.min(10, Math.sqrt(n.degree_in + n.degree_out) * 2.5);

/// Default view shows a compact label (real corpora have paragraph-length
/// titles); the full title appears on hover/selection and in the panel.
const SHORT_LABEL_CHARS = 42;
function shortLabel(title: string): string {
  if (title.length <= SHORT_LABEL_CHARS) return title;
  const cut = title.slice(0, SHORT_LABEL_CHARS);
  const lastSpace = cut.lastIndexOf(' ');
  return cut.slice(0, lastSpace > 24 ? lastSpace : SHORT_LABEL_CHARS).trimEnd() + '…';
}

// ---- State -----------------------------------------------------------------

const graph = new Graph({ multi: true, type: 'directed' });
let thread: { nodes: Set<string>; edges: Set<string> } | null = null;
let selected: string | null = null;
let hovered: string | null = null;

const container = document.getElementById('graph')!;
const countsEl = document.getElementById('counts')!;
const statusEl = document.getElementById('status')!;
const selectionEl = document.getElementById('selection')!;
const legendEl = document.getElementById('legend')!;

/// Hover/highlight label renderer matching the dark theme (Sigma's default
/// draws a white box, illegible with our light label color).
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
  context.lineWidth = 1;
  context.stroke();

  context.fillStyle = '#e8e9ee';
  context.fillText(label, x + 2, y);

  // Re-draw the node disc on top of its hover halo.
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
    // Hover/selection promotes the compact label to the full title.
    const expanded = node === hovered || node === selected;
    const label = expanded ? (data.fullLabel as string | undefined) ?? data.label : data.label;
    if (!thread) return { ...data, label };
    if (thread.nodes.has(node)) {
      return { ...data, label, zIndex: 1, highlighted: node === selected };
    }
    return { ...data, color: DIM_NODE, label: null, zIndex: 0 };
  },
  edgeReducer(edge, data) {
    if (!thread) return data;
    if (thread.edges.has(edge)) {
      // Unmissable: accent color + double thickness for the lit thread.
      return { ...data, color: THREAD_EDGE, size: (data.size ?? 1) * 2, zIndex: 1 };
    }
    return { ...data, color: DIM_EDGE, zIndex: 0 };
  },
});

// ---- Graph (re)build -------------------------------------------------------

function applyGraph(api: ApiGraph): void {
  // Preserve positions across rebuilds so the picture doesn't jump on save.
  const positions = new Map<string, { x: number; y: number }>();
  graph.forEachNode((id, attrs) =>
    positions.set(id, { x: attrs.x as number, y: attrs.y as number }),
  );
  graph.clear();

  for (const n of api.nodes) {
    graph.addNode(n.id, {
      label: shortLabel(n.title),
      fullLabel: n.title,
      color: colorFor(n.doc_type),
      size: sizeFor(n),
      x: positions.get(n.id)?.x ?? Math.random(),
      y: positions.get(n.id)?.y ?? Math.random(),
      docType: n.doc_type,
      status: n.status,
      risk: n.risk_level,
      created: n.created,
      isNew: !positions.has(n.id),
    });
  }
  // Sigma needs both endpoints; dangling edges are API/stats-only at M1.
  for (const e of api.edges) {
    if (e.resolved && graph.hasNode(e.source) && graph.hasNode(e.target)) {
      graph.addEdgeWithKey(String(e.id), e.source, e.target, {
        size: e.edge_type === 'RELATED_TO' ? 1 : 1.6,
        type: 'arrow',
      });
    }
  }

  // strongGravityMode keeps disconnected components (orphans!) from being
  // repelled out of frame — without it FA2 sends them flying.
  const fa2Settings = {
    ...forceAtlas2.inferSettings(graph),
    strongGravityMode: true,
    gravity: 0.5,
  };
  if (positions.size === 0) {
    // First load: circular seed then a full ForceAtlas2 pass.
    circular.assign(graph);
    forceAtlas2.assign(graph, { iterations: 300, settings: fa2Settings });
  } else {
    // Rebuild: settle briefly so new nodes find their place without
    // reshuffling the existing layout.
    forceAtlas2.assign(graph, { iterations: 30, settings: fa2Settings });
  }

  countsEl.textContent =
    `${api.stats.total_docs} docs · ${api.stats.total_edges} links` +
    (api.stats.orphans.length ? ` · ${api.stats.orphans.length} orphans` : '') +
    (api.stats.dangling_references.length
      ? ` · ${api.stats.dangling_references.length} dangling`
      : '');

  renderLegend(api);

  // Selection may have disappeared in the rebuild.
  if (selected && !graph.hasNode(selected)) clearSelection();
  renderer.refresh();
}

function renderLegend(api: ApiGraph): void {
  const seen = new Map<string, number>();
  for (const n of api.nodes) seen.set(n.doc_type, (seen.get(n.doc_type) ?? 0) + 1);
  legendEl.innerHTML = [...seen.entries()]
    .sort((a, b) => b[1] - a[1])
    .map(
      ([t, c]) =>
        `<span><i style="background:${colorFor(t)}"></i>${t} (${c})</span>`,
    )
    .join('');
}

// ---- Thread highlighting (S2/FR5) ------------------------------------------

async function select(nodeId: string): Promise<void> {
  selected = nodeId;
  const [threadRes, nodeRes] = await Promise.all([
    fetch(`/api/node/${encodeURIComponent(nodeId)}/thread`),
    fetch(`/api/node/${encodeURIComponent(nodeId)}`),
  ]);
  if (!threadRes.ok) return;
  const t: Thread = await threadRes.json();
  thread = {
    nodes: new Set(t.node_ids),
    edges: new Set(t.edge_ids.map(String)),
  };
  if (nodeRes.ok) showDetail(await nodeRes.json());
  renderer.refresh();
}

function clearSelection(): void {
  selected = null;
  thread = null;
  selectionEl.classList.remove('open');
  renderer.refresh();
}

function showDetail(detail: {
  node: ApiNode;
  excerpt: string | null;
  in_edges: ApiEdge[];
  out_edges: ApiEdge[];
}): void {
  const n = detail.node;
  const color = colorFor(n.doc_type);
  selectionEl.innerHTML = `
    <div class="doc-type" style="color:${color}">${n.doc_type}</div>
    <h2>${escapeHtml(n.title)}</h2>
    <div class="meta">
      ${escapeHtml(n.id)}<br/>
      status: ${escapeHtml(n.status)} · risk: ${escapeHtml(n.risk_level)}
      ${n.created ? `· ${escapeHtml(n.created)}` : ''}<br/>
      links: ${detail.out_edges.length} out · ${detail.in_edges.length} in
    </div>
    <p>${escapeHtml(detail.excerpt ?? '')}</p>`;
  selectionEl.classList.add('open');
}

const escapeHtml = (s: string) =>
  s.replace(/[&<>"']/g, (c) => `&#${c.charCodeAt(0)};`);

renderer.on('clickNode', ({ node }) => void select(node));
renderer.on('clickStage', clearSelection);
renderer.on('enterNode', ({ node }) => {
  hovered = node;
  renderer.refresh();
});
renderer.on('leaveNode', () => {
  hovered = null;
  renderer.refresh();
});

// ---- Live updates (S3/FR6) --------------------------------------------------

function connect(): void {
  const proto = location.protocol === 'https:' ? 'wss' : 'ws';
  const ws = new WebSocket(`${proto}://${location.host}/api/stream`);
  ws.onmessage = (m) => {
    const msg = JSON.parse(m.data);
    if (msg.event === 'rebuild') {
      statusEl.textContent = 'live';
      statusEl.className = 'live';
      applyGraph(msg.graph as ApiGraph);
    }
  };
  ws.onclose = () => {
    statusEl.textContent = 'reconnecting…';
    statusEl.className = 'down';
    setTimeout(connect, 1500);
  };
}

connect();
