---
id: AILOG-2026-06-14-001
title: Loom R3 — legibility at 100+ nodes (label density + hide isolated)
status: accepted
created: 2026-06-14
agent: claude-fable-5
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 140
files_modified: [CHANGELOG.md, Cargo.lock, README.md, docs/adopters/CLI-REFERENCE.md, docs/i18n/es/README.md, docs/i18n/es/adopters/CLI-REFERENCE.md, docs/i18n/zh-CN/README.md, docs/i18n/zh-CN/adopters/CLI-REFERENCE.md, experimento/CHANGELOG.md, experimento/Cargo.toml, experimento/web/index.html, experimento/web/package.json, experimento/web/src/i18n.ts, experimento/web/src/main.ts]
observability_scope: none
tags: [loom, ui, density, labels, knowledge-graph]
related: [AILOG-2026-06-13-002, AILOG-2026-06-12-002]
---

# AILOG: Loom R3 — legibility at 100+ nodes

## Summary

Closes the last Loom UX follow-up (R3). After R2 the Sentinel graph reached ~195 nodes and the
visualization got noisy. The operator picked the two pain points to address: **noisy labels**
and **isolated nodes cluttering the canvas**. Frontend-only; ships as `loom-0.4.2`.

## Actions Performed

1. **Label density cap.** Added Sigma `labelDensity: 0.6` / `labelGridCellSize: 180` and raised
   `labelRenderedSizeThreshold` to 10. Sigma labels the highest-`size` node per screen-grid
   cell; since nodes are sized by centrality (`applySizing`, default betweenness), the surviving
   labels are the important ones. Zooming in reveals more. The hovered/selected node gets
   `forceLabel: true` so its full label always shows.
2. **"Labels" toggle** (header checkbox, default on): `renderer.setSetting('renderLabels', …)`
   for a pure-structure view.
3. **"Hide isolated" toggle** (header checkbox, default off): a precomputed `isolated` set
   (`graph.degree(id) === 0`, recomputed per rebuild) drives an early `nodeReducer` branch that
   returns `{ hidden: true }`, removing singleton/orphan nodes from the canvas and from label
   consideration. The toggle shows the hidden count.
4. **i18n** for the two controls (`view.labels`, `view.hideIsolated`) in en/es/zh-CN; listeners
   wired once (consistent with the 0.4.1 delegation fix).

## Decisions Made

- **Tuned defaults over a manual density slider.** Centrality sizing already ranks nodes, so
  Sigma's per-cell label heuristic needed only coarser grid settings — no new ranking code.
- **Hide isolated via the reducer, not by dropping nodes.** Keeps the data/graph intact (stats
  and the legend's isolated count stay accurate); only the render hides them.
- **Did not touch the force layout or edges.** The operator did not flag a hairball; R3 stays
  scoped to labels + isolated nodes.

## Impact

- **Functionality:** additive view controls; default render is less cluttered at scale. No API,
  backend or core change (`straymark-core` stays 0.4.1).
- **Security/perf:** unchanged; the isolated set is precomputed per rebuild, not per frame.

## Verification

- [x] `npm run build` — tsc + vite pass (Sigma settings, `forceLabel`/`hidden`/`setSetting`
      types valid).
- [x] Served HTML exposes both toggles; `/api/stats` healthy (197 nodes on Sentinel); forged
      `Host` → 403.
- [ ] Visual acceptance (operator): fewer/important labels by default, zoom reveals more,
      Labels-off hides them, Hide-isolated removes the loose nodes.

## Follow-ups

- Loom's Knowledge-Graph track (M0–M3) and all connectivity/UX follow-ups (R1–R3) are now done.
  The next frontier is the Architecture track (A1/A2, Spec 002) and graduation (N=2).

---

<!-- Template: StrayMark | https://strangedays.tech -->
