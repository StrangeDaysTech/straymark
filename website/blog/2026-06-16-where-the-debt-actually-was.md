---
slug: where-the-debt-actually-was
title: Where the debt actually was
authors:
  - jose
tags:
  - straymark
  - loom
  - architecture
  - technical-debt
  - visualization
  - rust
  - sentinel
date: 2026-06-16T00:00:00.000Z
description: The operator's real daily question isn't "show me the documents," it's "where are we?" Loom grew a second surface — the architecture as a building you can walk, in 2D and in 3D — and its technical-debt overlay was empty. Not because there was no debt. Because it didn't know where to put it.
---
*Loom started as a knowledge graph of documents. Then the reference adopter said, more or less: that's a beautiful picture of my paperwork, but my daily question is "where are we?" against the **system** — the components, the layers, what's built and what's still owed. So Loom grew a second surface: the architecture as a building, authored once and rendered as both a 2D plan and a 3D exploded model, with a status overlay that lights components up. The overlay for technical debt came up empty on the first run. There was plenty of debt. The overlay just had no idea which components carried it — and getting that right took three tries, two of which looked finished.*

<!-- truncate -->

> *The empty overlay was the most honest bug of the month: it wasn't wrong about the debt. It was wrong about where to draw it.*

This is the third post in the Loom arc — after [extracting the shared parser](what-the-second-reader-demanded) and [making the document graph's edges actually resolve](what-the-graph-couldnt-draw-yet). It's the one where Loom stops being a view of the docs and becomes a view of the *system*, and where a deceptively simple overlay taught a lesson about every dashboard that's ever shown you a confident zero.

## The reframe

The pivot is recorded in [`ADR-2026-06-02-002`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-002-architecture-plan-format.md). Loom was scoped as a knowledge-graph view of documents; operator feedback from Sentinel reframed it. For a large project, the operator's daily question isn't "show me the document web," it's *"where are we?"* against the **implementation map** — the architecture. The metaphor came from civil engineering and BIM: **one model, many views**, with layers that light up to show status.

That reframe ran into a wall the exploration had already mapped: **StrayMark has no `component` concept.** It maps documents to *files* — a Charter's "Files to modify," an AILOG's "Modified Files," `api_changes`. So an architecture surface needs three new things: a way to *model* components and layers, a way to *project* live governance state onto them, and an editable format that carries a human-authored layout without a tool stomping on it. The enabling facts were already in hand — Charter status (`declared` / `in-progress` / `closed`) and declared-vs-modified drift are computed today, ADRs already embed C4 diagrams and "Affected Components" tables, and `.straymark/` stages 00–09 form a natural layering.

## You are here, from the terminal

The first half shipped *without any graphics at all* — [`cli-3.25.0` / `core-0.5.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.25.0), Loom track A1. The bet: answer "where are we?" in the terminal first, prove the projection, then draw it.

- **`straymark architecture generate`** mines the codebase structure into a first-draft `model.yml` plus a `plan.drawio`, enriched with signal from ADRs (C4 diagrams, Affected-Components tables improve the labels and links).
- **`straymark architecture sync`** is append-only — it detects new source dirs or ADR components and adds them, **never clobbering human edits or DrawIO geometry**. Dry-run by default.
- **`straymark architecture validate`** reports model↔plan integrity signals (`undrawn` / `unmodeled` / `empty`) and exits non-zero on any of them, so it's CI-gateable.
- **`straymark status --where`** is the textual "you are here": it projects each component's state — `active` / `in-progress` / `implemented` / `has-debt` / `uncharted` — from live governance signals and prints a summary.

The load-bearing decision is in `straymark-core`: the projection is a **pure** `(model + GovernanceState) -> Projection` function, zero I/O, shared by the CLI's `status --where` and the Loom server. That's not tidiness for its own sake — it's the same principle that drove the whole arc. The terminal answer and the visual answer are computed by *the same code*, so they **cannot disagree** (the spec calls it NFR3). A dashboard that contradicts the CLI is worse than no dashboard; this makes the contradiction unrepresentable.

## One model, two views

Then the views, both reading that one projection. [`loom-0.5.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.5.0) (A2) renders the human-authored `plan.drawio` with **maxGraph**, preserving the geometry you laid out (NFR1) and overlaying the §4 status as non-destructive cell colors. [`loom-0.6.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.6.0) ([#268](https://github.com/StrangeDaysTech/straymark/issues/268)) added the Spec 002 north star: a `2D | 3D` toggle that swaps the plan for a **Three.js axonometric scene** — each layer a stacked, translucent floor, each component a box colored by the same "you are here" palette, dependency edges drawn between boxes, and an **explode slider** that peels the floors apart on the vertical axis while the dependency lines re-route to the boxes' new positions. The BIM exploded view, literally: the CLI authors the structure, Loom renders it as a 2D plan *and* a 3D model, and the status overlay is always the one shared projection. Switching views disposes the GL context cleanly; clicking a box opens the same component detail panel the 2D plan uses.

<figure>
<img src="/img/blog/loom/arch-2d-thumb.png" data-zoom-src="/img/blog/loom/arch-2d.png" alt="Loom 2D Architecture Plan: flat component boxes — cmd (binaries), Agents (AIOps), Audit Trail, CommsHub, Identity, MCP Server — feeding into Core (eventbus / storage / ids), each box colored red for has-debt or blue for implemented" loading="lazy" />
<figcaption><em>A detail of the 2D Architecture Plan — the human-authored <code>plan.drawio</code> rendered by maxGraph, overlaid live with the shared status projection (red = <code>has-debt</code>, blue = <code>implemented</code>). Click for the full plan.</em></figcaption>
</figure>

<figure>
<img src="/img/blog/loom/arch-3d-thumb.png" data-zoom-src="/img/blog/loom/arch-3d.png" alt="Loom 3D axonometric BIM view: architecture layers as stacked translucent floors (CORE, MODULES, ENTRYPOINTS), components as boxes colored red (has-debt) or blue (implemented), dependency lines drawn between them" loading="lazy" />
<figcaption><em>The same model as a 3D exploded view — each layer a translucent floor, each component a box in the same status palette, dependency edges between boxes; the explode slider peels the floors apart on the vertical axis. Click for the full scene.</em></figcaption>
</figure>

It is, genuinely, a pleasure to fly around your own architecture and watch the active component glow. And on the very first real run, the overlay that was supposed to be the point — *where is the technical debt?* — was blank.

## Where the debt actually was

The `has-debt` overlay lied three times before it told the truth, and each lie looked like a finished feature.

**Lie #1 — always empty.** An open TDE (Transversal Debt Entry) declares its `related` documents in frontmatter. Those are *governance documents* — AILOGs, audit reviews, Charters — **not source paths**. So when the projection tried to match a TDE's debt against a component (which is defined by file globs), it matched nothing, every time. The overlay wasn't reporting "no debt"; it was reporting "I looked for debt in a place debt is never recorded." [#273](https://github.com/StrangeDaysTech/straymark/issues/273) ([`cli-3.26.0` / `core-0.6.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.26.0)) bridged the gap: resolve each `related` AILOG to the files *it* recorded as modified, and feed those source paths into the match. With a subtlety that bit immediately — an AILOG's modified files are the **union** of its `files_modified` frontmatter list and its `## Modified Files` table, because older logs often carry only the list, and reading just the table silently dropped them (it lit Identity/Core/Database on Sentinel but missed AuditTrail/CommsHub).

**Lie #2 — over-attributed.** Now the debt showed up, and it showed up *everywhere*. A TDE about the AuditTrail module lit `cmd`, `db`, and `integration` too — because a single AILOG had wired all of them in the same change, and the projection inherited that AILOG's entire footprint. The overlay went from blank to a smear. Technically it was attributing debt to "components touched by the AILOGs the TDE references," which is a defensible sentence and the wrong answer. The debt was about AuditTrail; the footprint was the whole subsystem.

**Lie #3 — precise.** [#276](https://github.com/StrangeDaysTech/straymark/issues/276) ([`fw-4.28.0` / `cli-3.28.0` / `core-0.8.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.28.0)) gave a TDE a way to say what it's actually about: an **`affects`** field, a list of file globs (`["internal/modules/audittrail/**"]`). When present, the projection attributes the debt to **exactly** those paths and ignores the broader AILOG footprint. Absent `affects`, the footprint attribution stays as the fallback — additive, no migration, no regression for TDEs that don't scope themselves. The framework ships the field in `TEMPLATE-TDE.md`; Loom's [`loom-0.6.2`](https://github.com/StrangeDaysTech/straymark/releases/tag/loom-0.6.2) needed no code change at all — it just rebuilt against `core-0.8.0`, because the projection is shared and the visual overlay rode the same fix the terminal did.

There was a fourth, quieter correction worth naming. Even once the debt landed on the right component, a module that was *both* implemented and in debt painted calm "implemented" blue and the debt never showed. So the overlay gained a **priority**: `has-debt` and `wiring-gap` rank above `implemented`, in the 2D plan, the 3D view, and the legend. A component paints the state that needs attention, not the state that feels good.

<figure>
<img src="/img/blog/loom/arch-3d-explode-thumb.png" data-zoom-src="/img/blog/loom/arch-3d-explode.png" alt="A close orbit of the Loom 3D view: a descending row of component boxes, the red ones carrying technical debt (CommsHub, Identity) next to blue implemented ones (MCP Server, Policy Engine, Status Center)" loading="lazy" />
<figcaption><em>Orbiting the same building: red boxes carry open debt, blue are implemented — the overlay that took three tries to land the color on the right component. Click for the full scene.</em></figcaption>
</figure>

## Opening it

Two releases made the architecture surface usable beyond the two repos that birthed it. [#279](https://github.com/StrangeDaysTech/straymark/issues/279) ([`cli-3.27.0` / `core-0.7.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.27.0)) made the seed scanner **language- and structure-agnostic**: a `ScanConfig`, resolved from an optional `architecture:` section in `.straymark/config.yml`, that recognizes ~30 languages out of the box and skips build scaffolding like `src/main/java` so a Maven/Gradle multi-module project gets one component per module instead of collapsing into a single `main` box. A project in a non-default language is no longer an empty map. And [#280](https://github.com/StrangeDaysTech/straymark/issues/280) shipped the adopter guide — [`docs/adopters/LOOM.md`](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/LOOM.md) in EN, ES, and zh-CN, live on [straymark.dev](https://straymark.dev) — with the recommended `generate → refine → sync → validate` workflow.

## What we deliberately didn't do

The restraint section. All of this is still **EXPERIMENTAL (Loom v0 / N=1)** and may change without a deprecation cycle — the architecture model, the CLI verbs, the overlay semantics, the 3D view. The agnostic scanner's defaults are unchanged for existing Go/Rust/JS layouts (Sentinel still seeds the same 14 components) — `#279` only *adds* reach, it doesn't reshape what already worked. And `affects` is additive: the footprint fallback stays, so the field is an opt-in for the TDEs that want precision, not a new obligation for the ones that don't. The temptation with a north-star feature is to stabilize it because it finally looks impressive. It looks impressive *and* it stays v0 until a second project beyond Sentinel has walked its own building.

## If you've read this far

The portable exercise is the one the empty overlay handed us. Find a dashboard, report, or metric in your world that shows a reassuring number — zero open incidents, zero failing checks, zero debt on this service. Now ask whether that zero means "I looked and there is none" or "I looked in a place this thing is never recorded." The `has-debt` overlay wasn't lying about the amount of debt; it was lying about its own competence to find it, and it did so with a perfectly calm blue. The three iterations — empty, smeared, scoped — are what it took to make the color mean what a person reading it assumes it means. Every confident zero you didn't personally trace deserves the same suspicion: not *is the number right*, but *does the thing producing it even know where to look?*

---

*StrayMark `cli-3.25.0` → `cli-3.28.0`, `core-0.5.0` → `core-0.8.0`, `fw-4.27.0` → `fw-4.28.0`, `loom-0.5.0` → `loom-0.6.2` — ADR [2026-06-02-002](https://github.com/StrangeDaysTech/straymark/blob/main/docs/decisions/ADR-2026-06-02-002-architecture-plan-format.md) · Spec [002-architecture-plan](https://github.com/StrangeDaysTech/straymark/blob/main/experimento/specs/002-architecture-plan/spec.md) · Adopter guide [LOOM.md](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/LOOM.md) · Issues [#268](https://github.com/StrangeDaysTech/straymark/issues/268) · [#273](https://github.com/StrangeDaysTech/straymark/issues/273) · [#276](https://github.com/StrangeDaysTech/straymark/issues/276) · [#279](https://github.com/StrangeDaysTech/straymark/issues/279) · [#280](https://github.com/StrangeDaysTech/straymark/issues/280). Predecessors: [What the second reader demanded](what-the-second-reader-demanded) · [What the graph couldn't draw yet](what-the-graph-couldnt-draw-yet).*

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all responsibility for the content rests with the human author.*
