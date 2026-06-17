---
slug: what-the-graph-couldnt-draw-yet
title: What the graph couldn't draw yet
authors:
  - jose
tags:
  - straymark
  - loom
  - knowledge-graph
  - visualization
  - rust
  - sentinel
date: 2026-06-14T00:00:00.000Z
description: Loom's first release rendered the document graph live in the browser in under a second. It also drew 330 edges to nowhere. The visible feature was the easy part — making the references actually resolve was the work, and it taught us when a dangling edge is a bug and when it's a signal.
---
*The walking skeleton shipped fast and looked great: a force-directed graph of every StrayMark document, colored by type, rebuilding in the browser within a second of saving a file. Then we pointed it at a real corpus — [Sentinel](https://github.com/StrangeDaysTech/sentinel), 395 references — and 330 of them dangled. Not because the documents were broken. Because they referenced each other the way humans write, not the way a naive graph builder matches. The render was a weekend. Making the edges land took the rest of Loom M1's follow-ups, and it changed what a "broken link" even means.*

<!-- truncate -->

> *330 of 395 references dangling. The graph was technically correct and practically useless.*

The first post in this arc was about the plumbing — extracting [`straymark-core`](what-the-second-reader-demanded) so the CLI and Loom parse documents with the same code. This one is about what happened when that shared graph builder met a corpus it hadn't been tuned against, and discovered that drawing a graph is the part nobody should be impressed by.

## The walking skeleton

[`loom-0.1.0` / `cli-3.24.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.24.0) ([#240](https://github.com/StrangeDaysTech/straymark/issues/240)) is Loom M1: a loopback-only, read-only web dashboard that renders the project's document graph live.

The stack, recorded in [`ADR-2026-06-02-001`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-loom-stack.md): an **axum + tokio** server that builds the graph via `straymark-core` and serves a small read API plus a WebSocket; a **`notify`** filesystem watcher (250 ms debounce) that re-parses on settled `.md` changes and pushes a rebuild over the socket — an open browser reflects an edit in well under a second (measured ~255 ms); a **Sigma.js + graphology** frontend, force-directed, colored by document type, where selecting a node lights up its full transitive thread and dims the rest. The whole web UI is **embedded via rust-embed** — adopters never run `npm`; they download one binary.

Two non-negotiables came from the spec's security posture, and they're worth naming because they shaped everything after: the server **binds `127.0.0.1` exclusively** and **rejects non-loopback `Host` headers** (anti DNS-rebinding), and it is **read-only by construction**. There is no write path to add later and forget to secure, because there is no write path at all. The CLI's `straymark loom serve` is a download-on-demand launcher — the download gate *is* the experimental opt-in boundary — and it falls back to the cached binary when offline.

## From picture to instrument

M2 and M3 turned the picture into something you'd actually reach for. [`loom-0.2.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.2.0) ([#241](https://github.com/StrangeDaysTech/straymark/issues/241)) added **Louvain community detection** with cluster coloring, a corpus stats panel (counts by type/status/risk, navigable orphans, dangling references), and server-side filters by type, status, risk, tag, and date range. [`loom-0.3.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.3.0) ([#243](https://github.com/StrangeDaysTech/straymark/issues/243)) added **incremental rebuilds** — a parse cache re-parses only changed files and the SPA patches the graph in place, preserving the layout of everything that didn't move instead of re-shuffling the whole canvas on every keystroke — plus **dependency-cycle (SCC) reporting** over the directed semantic edges, **centrality-based node sizing** (Betweenness by default, so the bridge documents grow), search-with-camera-focus, "pin subgraph" to isolate a thread, open-in-editor deep-links, and full UI internationalization (`en` / `es` / `zh-CN`) driven by the project's configured language.

<figure>
<img src="/img/blog/loom/kg-graph-thumb.png" data-zoom-src="/img/blog/loom/kg-graph.png" alt="Loom Knowledge Graph view: a force-directed graph of StrayMark documents, nodes colored by community and sized by centrality, with labels such as 'Decommissioning Saga step 3' and 'Observability bundle: 7 instruments'" loading="lazy" />
<figcaption><em>A detail of Loom's Knowledge Graph view — documents as a force-directed graph, nodes colored by Louvain community and sized by betweenness centrality, so the bridge documents grow. Click through for the full corpus.</em></figcaption>
</figure>

All of that is genuine, and all of it was sitting on top of a graph where most of the edges went nowhere.

## 330 of 395

Here is the number that reframed the project. Dogfooding `loom-0.1.0` against Sentinel, **330 of 395 references were dangling** — drawn as edges to nodes that didn't exist. A knowledge graph that can't connect five-sixths of its own corpus isn't a knowledge graph; it's a scatter plot with delusions.

The cause was not corruption. It was that StrayMark documents reference each other the way a person writes a citation, and the graph builder was matching like a database expecting primary keys. An AILOG says `CHARTER-12`, or names a file by its basename, or gives a relative path suffix — and the builder, looking for an exact node id, found nothing. Worse, whole categories of target *weren't even nodes*: the graph discovered AILOGs and TDEs and ADRs, but Charters, PLAN telemetry, and audit reviews lived in `.straymark/` and had never been injected, so every reference to them dangled by definition.

[`loom-0.4.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.4.0) ([#246](https://github.com/StrangeDaysTech/straymark/issues/246)) fixed both halves, and — because the fix lives in the shared `straymark-core` graph builder — `straymark audit` gained the same connectivity for free:

- **Reference normalization (R1).** The builder now resolves an edge target by exact id and, failing that, by unique file basename, unique relative-path suffix, `CHARTER-NN` prefix, or leading dated id prefix — **never resolving an ambiguous match** (two candidates means it stays dangling, because a wrong edge is worse than a missing one). Resolved targets canonicalize to the node id.
- **Entity nodes (R2).** A new `core::entities` module discovers `.straymark/charters/*.md`, `plans/PLAN-*.telemetry.yaml`, and `audits/*/review.md` and injects them as `CHARTER` / `PLAN` / `AUDIT` nodes, so references to them have something to land on.

Measured on Sentinel: dangling references **330 → 87**, nodes **131 → 193** (+41 charters, +5 plans, +16 audits), orphans **2 → 0**. And the remaining 87 *correctly* stay dangling — they point at files outside the governance corpus (`.specify/memory/…`, `constitution.md`). Which raised the real question.

## When a dangling edge is a feature

If 87 unresolved references are *correct*, then "dangling" can't be a synonym for "broken." A panel that screams about all 87 is crying wolf, and a panel that hides them is hiding real broken links in the same bucket. The category itself was wrong.

[#262](https://github.com/StrangeDaysTech/straymark/issues/262) (shipped alongside the Architecture view in [`loom-0.5.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.5.0)) split the unresolved targets into three kinds: a **broken governance link** (an id that *should* resolve and doesn't — the actionable case), a **file reference** (a path to code, a spec, a sidecar — not a document, correctly absent from the doc graph), and an **external link** (a URL). On the Sentinel corpus the dangling-references panel dropped from **92 false alarms to 0** real broken links. The dangling edge didn't get suppressed; it got *classified*. It became a signal with a type, which is the only form in which a warning is worth showing.

This is the same lesson the [previous arc](what-the-bash-script-said-was-in-sync) learned about a deprecated bash script reporting "in sync": be lenient about what you read, exclusive about what you claim. R1 is leniency (parse the citation a human actually wrote); the never-resolve-ambiguous rule and the three-way classification are exclusivity (don't claim an edge you can't justify, don't cry broken when it isn't). Same discipline, different surface.

## The bug that ate the clicks

One fix from this stretch deserves its own paragraph because the symptom was maddening and the cause was elegant. Operators reported that Loom's side-panel buttons — dangling-ref links, community toggles, stats actions — felt *dead*, clicks landing nowhere, intermittently. [`loom-0.4.1`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.4.1) ([#247](https://github.com/StrangeDaysTech/straymark/issues/247)) found it: the watcher was broadcasting a WebSocket "rebuild" every time a file's modification time moved **even when its content hadn't changed** — an editor save-without-edits, a formatter, a `touch`, a cloud-sync rewrite. Each no-op broadcast re-rendered every open client's panels via `innerHTML`, destroying the freshly-bound click handlers a fraction of a second after they were attached. The graph looked fine; the panels were being lobotomized on a timer. Two fixes: the watcher now diffs the new graph against the old and stays silent when they're identical, and the panels use **event delegation** on stable containers so clicks survive a legitimate re-render. [`loom-0.4.2`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.4.2) ([#248](https://github.com/StrangeDaysTech/straymark/issues/248)) then made 100+ node graphs legible — label density capped to the most central node per screen region, a "hide isolated" toggle, on-screen zoom/fit controls.

## What we deliberately didn't do

The server stays **strictly read-only**. Open-in-editor is a client-side deep-link (a `vscode://` URL the browser hands off) and copy-path is a clipboard write in the page — the server never touches your editor, your files, or anything outside the loopback socket. There is no "and Loom can also edit the doc for you" feature on the roadmap, because the read-only-by-construction posture is worth more than the convenience, and you can't add the convenience without throwing the posture away.

## If you've read this far

The portable exercise: take the cross-references in your own project — issue mentions, `@see` doc links, "depends on" notes in tickets, the import graph if you want to be literal — and ask what fraction would actually resolve if a tool tried to follow them. Then ask the harder question the 330-of-395 moment forced on us: of the ones that *don't* resolve, can your tooling tell a genuinely broken link from a reference that's correctly pointing outside its own world? Until it can, every "N broken references" number it shows you is some unknown mix of fires and false alarms — and a warning you've learned to ignore is worse than no warning at all.

---

*StrayMark `loom-0.1.0` → `loom-0.5.0`, `cli-3.24.0` — ADR [2026-06-02-001](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-loom-stack.md) · Spec [001-loom-server](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-loom/specs/001-loom-server/spec.md) · Issues [#240](https://github.com/StrangeDaysTech/straymark/issues/240) · [#241](https://github.com/StrangeDaysTech/straymark/issues/241) · [#243](https://github.com/StrangeDaysTech/straymark/issues/243) · [#246](https://github.com/StrangeDaysTech/straymark/issues/246) · [#247](https://github.com/StrangeDaysTech/straymark/issues/247) · [#262](https://github.com/StrangeDaysTech/straymark/issues/262). Predecessor: [What the second reader demanded](what-the-second-reader-demanded). Next: [Where the debt actually was](where-the-debt-actually-was).*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*
