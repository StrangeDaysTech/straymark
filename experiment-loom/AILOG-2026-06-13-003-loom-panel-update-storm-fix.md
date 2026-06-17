---
id: AILOG-2026-06-13-003
title: Loom fix — panel clicks swallowed by no-op WebSocket rebuild storm
status: accepted
created: 2026-06-13
agent: claude-fable-5
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 90
files_modified: [CHANGELOG.md, Cargo.lock, README.md, cli/Cargo.toml, core/Cargo.toml, core/src/graph.rs, docs/adopters/CLI-REFERENCE.md, docs/i18n/es/README.md, docs/i18n/es/adopters/CLI-REFERENCE.md, docs/i18n/zh-CN/README.md, docs/i18n/zh-CN/adopters/CLI-REFERENCE.md, experimento/CHANGELOG.md, experimento/Cargo.toml, experimento/src/snapshot.rs, experimento/src/watcher.rs, experimento/web/package.json, experimento/web/src/main.ts]
observability_scope: none
tags: [loom, bugfix, websocket, watcher, ui, knowledge-graph]
related: [AILOG-2026-06-13-002, AILOG-2026-06-12-003]
---

# AILOG: Loom fix — panel clicks swallowed by no-op rebuild storm

## Summary

An operator reported that Loom's left-hand panels (corpus stats, legend) stopped responding
to clicks after some interaction — the item just clicked and its neighbours went dead while
others worked, and it persisted across page reload and across browsers, with **no console
errors**. The DevTools Elements view showed the `#stats` DOM **rewriting itself continuously
at high speed**.

Root cause: a **no-op WebSocket rebuild storm**. When something re-saves a watched `.md`
without changing its content (an editor save, a formatter, a `touch`, a cloud-sync rewrite),
the parse cache re-parses (its mtime moved), the graph is rebuilt **identical**, and the
watcher still broadcast a `delta`. Each delta re-renders the panels (`innerHTML` rewrite),
destroying the per-button click handlers bound on the previous render — so clicks landing
between renders were lost. Fixed at two layers.

## Actions Performed

1. **Server — suppress no-op broadcasts** (`watcher.rs` + `snapshot.rs`): the watcher diffs
   the rebuilt graph against the previous one and only broadcasts when something actually
   changed. Added `GraphDelta::is_noop(&prev)` (no node added/removed/changed and identical
   edges) and `GraphDelta::to_event`; derived `PartialEq` on `core::graph::Edge` and the
   Loom `ApiEdge` to compare edge sets.
2. **Client — event delegation** (`web/src/main.ts`): the stats and legend panels now handle
   clicks via one delegated listener on their stable container element instead of binding a
   listener per rendered button. Clicks survive the `innerHTML` rewrites the panels perform on
   every rebuild, so the UI stays responsive even under legitimately frequent updates.
   Native `<details>` toggling drives the open/closed state, mirrored into the persisted
   open-section set.

## Decisions Made

- **Both layers, not one.** Server suppression removes the common cause (mtime-only churn);
  client delegation makes the panels robust to *any* update frequency, including real frequent
  edits. Defense in depth for a UI that re-renders on a live data feed.
- **Equality over a dirty flag.** Comparing the new graph to the previous one (via the existing
  diff plus edge-set equality) is exact and cheap for these corpus sizes; no heuristic.

## Impact

- **Functionality:** the panels stay clickable under filesystem churn; real changes still
  propagate live. No API or visual change.
- **Performance:** fewer needless rebuilds reach clients; the panels stop thrashing.
- **Security:** unchanged.

## Verification

- [x] `cargo test` — core 42/42, loom 9/9 (added a no-op-delta assertion); clippy core+loom
      clean; `npm run build` ok.
- [x] Reproduction (temp project, WS client): rewriting identical content 5× produced **10
      no-op deltas before the fix → 0 after**; a real change (new document) still produces a
      delta after the fix.

## Follow-ups

- R3 (visual density at 100+ nodes) and the Architecture track (A1/A2) remain the next Loom
  frontiers.

---

<!-- Template: StrayMark | https://strangedays.tech -->
