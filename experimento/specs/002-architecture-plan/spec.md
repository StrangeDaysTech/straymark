# Feature Spec 002 — Loom Architecture Plan view ("you are here")

> **SpecKit artifact — the WHAT.** The second visualization surface of the Loom dashboard:
> the **architectural plan** of the system, with a live status overlay that answers the
> operator's daily question — *"where are we?"*. Companion to Spec 001 (the knowledge
> graph). The HOW is in `plan.md` (shared with 001); the format/model decision is recorded
> in `docs/decisions/ADR-2026-06-02-002-architecture-plan-format.md`. Status: **draft /
> experimental (v0, N=1)**.

## 1. Problem & intent

A large project (reference: Sentinel) has a *designed* architecture — layers, modules,
data stores, buses, external providers — that an operator carries in their head like a
floor plan. When an operator juggles three or four such projects, every morning starts with
asking the AI agent "¿dónde vamos?" (where are we). The knowledge graph (Spec 001) shows
how *documents* relate, but it does not show the **system's implementation map** nor *where
on it the current work is happening*.

Spec 002 adds the **Architecture Plan view**: a structured diagram of the system's
components and layers — like the hand-drawn Sentinel architecture map, or a civil-
engineering blueprint — with a **"you are here" status overlay** computed from governance
state. Borrowing from **BIM (Building Information Modeling)**: one *model*, many *views*
(plan, and later an axonometric exploded view); layers that light up, shade, or dim to show
what is done, in-progress, in-debt, or uncharted.

This view is the **principal implementation map**. It is part of the experimental MVP and
must be feature-rich enough to attract adopters — not a sparse stub.

## 2. Users & primary stories

- **S1 (operator, daily).** *I open the architecture plan and the components touched by the
  currently active Charter are lit up ("you are here"); finished areas are shaded
  "implemented"; untouched areas are dimmed. I grasp where the project stands in seconds.*
- **S2 (operator).** *I click a component and a panel shows which Charters touch it, which
  documents (ADRs/REQs/AILOGs) concern it, its open technical debt, and the files it owns.*
- **S3 (operator).** *I toggle layers on/off (e.g. hide Persistence and Observability) to
  focus on the Core layer — the 2D analog of peeling floors apart.*
- **S4 (lead/architect).** *I rearrange the plan freely in DrawIO — move boxes, reroute
  edges — and Loom keeps my layout while re-applying the live status overlay on top.*
- **S5 (new contributor).** *I run one command and a first draft of the architecture plan is
  generated from the codebase structure and the C4 diagrams already in our ADRs; I refine
  it instead of drawing from scratch.*
- **S6 (evaluator).** *The plan plus its live overlay is the single most legible answer to
  "what is this project and where is it" — a reason to adopt StrayMark.*

## 3. The architecture model (contract)

One **model**, expressed as two linked artifacts (the BIM "model vs. drawing" split):

### 3.1 Semantic model — `architecture/model.yml`

Human-readable; the source of truth for *meaning*. (Adopter convention:
`.straymark/architecture/model.yml`. This source repo, lacking a root `.straymark/`,
dogfoods at `experimento/architecture/model.yml`.)

```yaml
version: 0
layers:                       # the "floors"; seeded from .straymark stages 00–09, user-editable
  - { id: frontend,   label: Frontend,                order: 0 }
  - { id: core,       label: Core Layer,              order: 1 }
  - { id: operations, label: Operations & Comms,      order: 2 }
  - { id: ai-devex,   label: AI & Developer Experience, order: 3 }
  - { id: persistence,label: Persistence,             order: 4 }
  - { id: observability, label: Observability,        order: 5 }
components:
  - id: identity-module       # stable id — the join key to the DrawIO cell and to status
    label: Identity Module
    layer: core
    globs: ["internal/identity/**", "cli/src/identity/**"]   # files this component owns
    links: [policy-engine, audit-trail]                      # architectural edges
    docs: []                  # OPTIONAL explicit doc ids; normally inferred via globs
    external: false
```

- A **component** is a named group of **file globs** placed in a **layer**, with optional
  **links** (edges) to other components. Globs are the join to governance state (§4).
- **Layers** default to the canonical `.straymark/` stages 00–09 (derived from
  `DocType::directory()`), but are user-definable to match the project's own layering (the
  Sentinel legend: Core / Operations / AI-DevEx / Persistence / Observability / Frontend).

### 3.2 Layout — `architecture/plan.drawio`

DrawIO (**mxGraph XML**); the source of truth for *position, shape, and routing*. Each cell
carries a custom attribute `straymark_component_id` linking it to a `model.yml` component.
The human edits this freely in DrawIO; Loom never overwrites geometry, only restyles.

### 3.3 Linkage & integrity

Model and layout are joined by `component_id`. Loom surfaces integrity signals analogous to
the knowledge graph's orphan/dangling detection:
- **Undrawn component** — in `model.yml` but no cell in `plan.drawio`.
- **Unmodeled cell** — a `straymark_component_id` in the DrawIO not present in `model.yml`.
- **Empty component** — globs match no files on disk.

## 4. Status projection — "you are here" (contract)

For each component, Loom computes a **state** by matching governance-derived file sets
against the component's globs. All inputs already exist in the CLI:

| State | Derivation (reuses existing CLI code) |
|---|---|
| `active` ("you are here") | a Charter with `status: in-progress` whose **declared files** (`charter_files::parse_files_to_modify`) match the globs |
| `in-progress` | within an `active` component, files already modified per `charter drift` (declared∩git-modified) vs. still-pending |
| `implemented` | a Charter with `status: closed` (and its AILOGs) whose files match the globs |
| `has-debt` | an open `TDE` document related to docs/files of the component |
| `wiring-gap` | `analyze declared-vs-wired` findings whose symbols/files fall in the component |
| `uncharted` | globs match files on disk but no document/charter references them |

The projection is a **pure function** of (model + parsed governance state) and lives in
`straymark-core`, so the CLI can answer "where are we" textually too (e.g. a future
`straymark status --where`), not only the web view.

Layer-level rollups reuse `metrics_engine` aggregates (counts by type/risk, review rate)
for a per-layer summary badge.

## 5. The generator (contract)

`straymark architecture generate|sync|validate` (a **CLI** command, because it writes files;
the Loom server stays read-only):

- **generate** (hybrid seed): propose components from the **codebase structure** (top-level
  dirs/modules; reuses the `analyze` file walker's inventory) **enriched** by mining the
  **C4 mermaid blocks** and **"Affected Components"** tables that ADRs already contain
  (Spec-explored: both are structured/parseable). Writes a first `model.yml` and an
  auto-laid-out `plan.drawio` (layered layout via elk/dagre).
- **sync**: detect new code dirs or new ADR components since last generation; *suggest*
  additions without clobbering the human's `model.yml`/layout (append + report, never
  overwrite geometry).
- **validate**: report the integrity signals of §3.3 (undrawn / unmodeled / empty).

## 6. Rendering (contract)

The Loom web app renders `plan.drawio` with **maxGraph** (`@maxgraph/core`, the maintained
successor to the mxGraph engine DrawIO is built on): it loads the mxGraph XML, preserves the
human geometry, and applies the §4 status as **non-destructive cell-style overrides**
(fillColor / opacity / stroke / a badge) keyed on `straymark_component_id`. Layer toggling
(§S3) shows/hides cells by their component's `layer`.

## 7. API surface (additions to Spec 001 §4)

| Method | Path | Returns |
|---|---|---|
| GET | `/api/architecture` | `{layers, components, edges, status}` — model + projected state |
| GET | `/api/architecture/component/:id` | component detail: matched docs, charters touching it, debt, owned files, state |
| GET | `/api/architecture/plan.drawio` | the DrawIO XML with live status styles applied (export) |
| GET | `/api/where` | the "where are we" summary: active charters + declared-vs-modified progress + recent AILOGs + open debt |
| WS | `/api/stream` | (shared) also pushes architecture-status deltas on `.straymark/` or `architecture/` changes |

Generation endpoints are intentionally **absent** — generation is the CLI's job (writes).
The server watches `architecture/model.yml`, `architecture/plan.drawio`, and the governance
docs, and live-updates the overlay.

## 8. The dashboard tie-in (the two views are one dashboard)

Loom is a **development dashboard**; the KG view (001) and the Architecture Plan view (002)
are linked:
- Click a **component** in the plan → filter the **knowledge graph** to the documents that
  concern it.
- Select a **document** in the graph → highlight the **component(s)** it touches in the plan.
- A shared **"Where are we" panel** (`/api/where`) gives the textual companion to the visual
  "you are here".

## 9. Functional requirements

- **FR1.** Parse `architecture/model.yml` (layers, components, globs, links) and validate it.
- **FR2.** Compute the per-component status projection of §4 using `straymark-core` (active
  charter declared files, drift, closed charters/AILOGs, TDE, declared-vs-wired).
- **FR3.** Render `plan.drawio` via maxGraph preserving geometry; apply non-destructive
  status styles keyed on `straymark_component_id`.
- **FR4.** Layer toggle: show/hide components by layer (the 2D "peel floors" analog).
- **FR5.** Component detail panel (S2) and the "Where are we" summary (`/api/where`).
- **FR6.** Live-update the overlay on changes to governance docs or the architecture files
  (shared watcher with Spec 001).
- **FR7.** `straymark architecture generate|sync|validate` CLI command (hybrid seed, §5).
- **FR8.** Cross-view linking with the knowledge graph (§8).
- **FR9.** Integrity signals (undrawn / unmodeled / empty component) surfaced like the KG's
  orphans/dangling refs.

## 10. Non-functional requirements

- **NFR1 (non-destructive).** Loom MUST NOT alter the geometry/routing a human authored in
  `plan.drawio`; only styles are overlaid. Round-trip through real DrawIO must be lossless.
- **NFR2 (language-agnostic).** Status projection uses file globs, never AST/language
  parsing — works for any stack (consistent with the framework's language-agnosticism).
- **NFR3 (consistency).** "you are here"/"implemented" states are consistent with what
  `straymark charter list --status in-progress|closed` and `charter drift` report.
- **NFR4 (read-only server).** The Loom server never writes; generation is an explicit CLI
  action.
- **NFR5 (no framework change required).** The default mapping requires adopters to add **no
  new frontmatter**; components map via globs. (An optional `component:` field is explicitly
  out of scope here — see §13.)

## 11. Acceptance criteria (definition of done for the Architecture Plan MVP)

1. `straymark architecture generate` on a project produces a `model.yml` + `plan.drawio`
   seeded from code dirs and ADR C4/Affected-Components, openable and editable in DrawIO.
2. The Loom dashboard renders the plan; components touched by the active (`in-progress`)
   Charter are visibly **lit ("you are here")**, closed-charter areas **shaded
   "implemented"**, and unreferenced areas **dimmed "uncharted"**.
3. Editing `plan.drawio` in real DrawIO (move a box, reroute an edge) and reloading
   preserves the new layout with the overlay re-applied (NFR1).
4. Toggling a layer off hides its components and back on restores them (FR4).
5. Clicking a component lists the Charters/docs touching it and its owned files (S2); the
   "Where are we" panel matches `straymark charter list --status in-progress` + drift.
6. A change to the active Charter's "Files to modify" (or a new commit moving drift) updates
   the overlay live (< ~1s) (FR6/NFR3).

> **Status (2026-06-16): MVP done — all 6 met.** Criteria 1 and the textual half of 5 shipped
> in **`cli-3.25.0`** (A1: `architecture generate|sync|validate` + `status --where`). Criteria
> 2, 3, 4, the visual half of 5, and 6 shipped in **`loom-0.5.0`** (A2: the maxGraph plan view +
> panels + live overlay), verified on the Sentinel + demo dogfoods. Next is **A3** (the
> axonometric/BIM exploded view, north star — §12).

## 12. Phasing (slots into the shared Loom phasing in `../001-loom-server/plan.md`)

- **A1 — Model + generator + projection.** `straymark-core` status projection (pure) +
  `straymark architecture generate|sync|validate` CLI. Ships as a `cli-` increment; gives
  immediate textual "where are we" value before any new pixels.
- **A2 — Architecture Plan view (the MVP surface).** maxGraph rendering of `plan.drawio` +
  live overlay + layer toggle + component panel + "Where are we" panel + cross-view linking.
  Ships in a `loom-0.x` release alongside/after the KG view's M2.
- **A3 — Axonometric/BIM (north star).** 2.5D stacked, explodable layers (the isometric
  "floors" of the BIM references). Explicitly post-MVP; pursued once the model is proven.

## 13. Out of scope (this spec)

- A new `component:` frontmatter field (deferred; globs cover the MVP without touching the
  Framework — see §10 NFR5).
- The 3D/axonometric exploded view (A3, north star — not MVP).
- AST/dependency-graph extraction of components (language-agnostic globs only).
- Editing the model/diagram *from* the Loom UI (generation/edit happen in CLI + DrawIO; the
  server is read-only).
- Auto-arranging/overwriting a human's DrawIO layout (NFR1 forbids it).

## 14. Open questions

- ~~Home of the architecture artifacts for adopters: confirm `.straymark/architecture/`
  (`model.yml` + `plan.drawio`).~~ **RESOLVED (A1):** `.straymark/architecture/` is the
  default for all three `architecture` subcommands and `status --where`; `--out <dir>`
  overrides it (the StrayMark repo itself dogfoods to `experimento/architecture/` since it has
  no root `.straymark/`).
- ~~Initial auto-layout engine for `generate` (elk.js vs dagre).~~ **DEFERRED to A2:** A1's
  `generate` emits `plan.drawio` with a simple Rust-side grid (no JS layout engine); a real
  auto-layout pass belongs to the visual render (A2), where the engine choice is a `plan.md`
  HOW detail.
- ~~Whether layer defaults should hard-map to `.straymark` stages 00–09 or always start from
  the project's own legend.~~ **RESOLVED (A1.2):** `generate` seeds the layer list from the
  stages 00–09 plus a placeholder `unassigned` layer; the human renames/regroups during
  refinement (confirmed by the dogfood — 9 seeded stages → 3 real layers).
- ~~Whether `/api/where` should also power a CLI `straymark status --where` in the same A1
  increment.~~ **RESOLVED (A1.4):** shipped. `status --where` and the future `/api/where` both
  build a `GovernanceState` and call the one pure `core::architecture::project`, so the
  textual and visual answers cannot disagree (NFR3).
