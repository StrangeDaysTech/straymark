# Loom — knowledge-graph visualization server (EXPERIMENTAL)

> ⚠️ **EXPERIMENTAL — v0 (N=1).** Loom is an opt-in, unstable experiment. Its API,
> CLI surface, on-disk layout, and very existence may change or be removed without a
> deprecation cycle until it is **graduated**. It is **not** part of the supported
> Framework (`dist/`) or CLI (`cli/`) contract. Do not build automation against it yet.

Loom is StrayMark's third component (alongside the **Framework** and the **CLI**): an
experimental **development dashboard** that makes a StrayMark project legible to the human
eye. It has two complementary visualization surfaces over the same project, plus shared
dashboard chrome:

1. **Knowledge Graph view** (Spec `specs/001-loom-server/`) — the web of StrayMark
   *documents*: Markdown files whose YAML frontmatter cross-links them (`related`,
   `supersedes`, `alternatives_documented`, `originating_ailogs`, …), rendered as a live,
   navigable, force-directed graph. Select a node and its line of relationships across
   documents lights up.
2. **Architecture Plan view** (Spec `specs/002-architecture-plan/`) — the *system's*
   implementation map: a structured blueprint of components and layers (like the hand-drawn
   Sentinel architecture diagram), with a **"you are here" status overlay** computed from
   governance state. The components touched by the active Charter light up; finished areas
   are shaded "implemented"; untouched areas are dimmed. It answers the operator's daily
   question — *"¿dónde vamos?"* — visually, the way a transit map says "you are here" or a
   BIM drawing shades a building's systems.

The AI already follows the document threads (the CLI builds the relationship graph
internally for `straymark audit`) and already computes the governance state (active/closed
Charters, declared-vs-modified drift, technical debt). The TUI `straymark explore` lets a
human *read* documents one at a time. **Loom lets a human *see* both the threads and the
map** — the whole entramado, and where the work is happening on it.

## Why

In a small project the document set fits in your head. In a project that grows (the
reference case is **Sentinel**), the TUI is progressively outpaced as a tool for a human
to understand *what is happening across the corpus*: which decisions supersede which,
which requirements are orphaned, where a Charter originates, which clusters of work have
formed. Loom is the **self-explanatory visual layer** for that — and, per early adopter
feedback, a potential **adoption attractor**: the graph is the most legible possible
answer to "what does StrayMark actually give me?"

The reference for the *idea* (not the implementation) is **Infranodus**: a force-directed
entity graph plus side panels for summaries, relations, stats, and topical clusters;
selecting a node highlights its relationship path. We borrow the shape — entity map +
panels — not the product.

## What it does

- Watches the project's `.straymark/` directory (or `docs/` in this source repo) for
  filesystem changes and **rebuilds both views in real time**.
- Serves a **web UI on `localhost`**.
- **Knowledge Graph:** renders the documents as a force-directed graph colored by type and
  community; selecting a node lights up its thread (in/out links) and dims the rest; side
  panels for per-node summary and corpus stats (orphans, dangling references, clusters).
- **Architecture Plan:** renders a human-authored DrawIO blueprint of the system's
  components/layers and overlays a live **"you are here"** status — components touched by the
  active Charter lit, closed-Charter areas shaded "implemented", debt-bearing areas badged,
  unreferenced areas dimmed "uncharted"; toggle layers on/off; a "Where are we" panel
  mirrors `straymark charter list --status in-progress` + drift.
- **Cross-linked:** click a component → filter the graph to its documents; select a document
  → highlight the component(s) it touches.

## Non-goals

Loom deliberately does **not**:

- **Use a graph database.** The graph is built in-memory from the Markdown frontmatter
  using the *same parser the CLI uses*. We only revisit this if parsing/rebuild becomes
  too slow at scale (see the spec's performance section for the explicit trigger).
- **Replace `straymark explore`.** The TUI stays the terminal-native, zero-dependency
  reader. Loom is the richer, graphical, browser-based complement.
- **Replace `straymark audit`/`validate`.** Loom *visualizes* the same model; it is the
  CLI commands that remain the source of truth and the gate.
- **Expose anything beyond loopback.** It binds `127.0.0.1` only, is read-only, and never
  writes into `.straymark/`.
- **Be a general Markdown/Obsidian graph.** It is specific to the StrayMark document model
  (its document types and its typed relationship fields).
- **Own or overwrite your architecture layout.** The Architecture Plan is a human-authored
  DrawIO file; Loom only overlays status styles non-destructively — it never rewrites your
  geometry or routing.
- **Require a graph/AST extractor or new frontmatter.** The "you are here" overlay maps
  documents to components via **file globs** in a small architecture model — language-
  agnostic, no Framework change, no per-document annotation (v0).
- **Ship the axonometric/BIM exploded view in the MVP.** The MVP is the 2D plan + overlay +
  layer toggle; the isometric "exploded floors" view is the north star, not v0.

## Status & maturity

| Property | Value |
|---|---|
| Maturity | **v0 (N=1)** — validated in one domain only (this project, dogfooding) |
| Distribution | opt-in, downloaded on demand by `straymark loom serve` |
| Release tags | `loom-X.Y.Z` (independent from `fw-` and `cli-`) |
| Binary | `straymark-loom` (single self-contained executable) |
| Graduation | criteria declared in the Charter's ex-ante scope (`CHARTER-01-loom-server.md`) |

"Experimental" is expressed here in the docs (v0/N=1), **not** in the version tag —
consistent with the rest of the project.

## How it will reach adopters

Loom is **not** bundled into the CLI binary. The CLI gains a thin `straymark loom serve`
subcommand that, on first use, downloads the latest `loom-*` release asset for the host
platform (reusing the same machinery that installs the Framework), caches it, prints a
loud EXPERIMENTAL banner, and launches it pointed at the project. The download-on-demand
gate *is* the opt-in boundary. This keeps the CLI small (no async/web stack in its
dependency tree) and lets Loom ship on its own cadence.

## Layout (planned)

```
experimento/
├── README.md                     # this file
├── CHANGELOG.md                  # independent release history (loom-X.Y.Z)
├── CHARTER-01-loom-server.md     # dogfood: the work-block Charter for building Loom
├── specs/
│   ├── 001-loom-server/          # SpecKit set — Knowledge Graph view
│   │   ├── spec.md               # WHAT — the feature constitution
│   │   ├── plan.md               # HOW — shared architecture & phasing
│   │   └── tasks.md              # ordered, checkable task list
│   └── 002-architecture-plan/    # SpecKit set — Architecture Plan view ("you are here")
│       └── spec.md               # WHAT — model, generator, status overlay, API
├── architecture/                 # dogfood model (created at implementation time)
│   ├── model.yml                 # semantic model: components → file globs, layers, links
│   └── plan.drawio               # human-authored layout (DrawIO / mxGraph XML)
├── Cargo.toml                    # straymark-loom crate (created at implementation time)
├── src/                          # axum server + notify watcher (created later)
└── web/                          # Sigma.js + graphology + maxGraph frontend (created later)
```

The shared document/graph model lives in a **new `straymark-core` crate** (extracted from
`cli/src/document.rs` + `cli/src/audit_engine.rs`) so that Loom and the CLI parse
frontmatter with the exact same code and the graph can never drift from the CLI's truth.
See `specs/001-loom-server/plan.md` and `docs/decisions/ADR-2026-06-02-loom-stack.md`.

## Dogfooding

Loom's own construction is documented with StrayMark's own document types: two **ADRs** —
the stack decision (`docs/decisions/ADR-2026-06-02-loom-stack.md`, `ADR-2026-06-02-001`) and
the Architecture Plan format decision (`docs/decisions/ADR-2026-06-02-002-architecture-plan-format.md`,
`ADR-2026-06-02-002`) — and a **Charter** for the work block (`CHARTER-01-loom-server.md`,
whose `originating_spec` points at `specs/001-loom-server/spec.md` — the SpecKit→Charter
bridge). The corpus Loom renders therefore includes the governance docs of Loom itself: a
worked example of StrayMark's traceability, visualized by the very tool it describes — and
its own architecture plan will show "you are here" as Loom is built.
