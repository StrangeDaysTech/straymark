---
id: AILOG-2026-08-02-005
title: Baton gate #5 — Loom intent plane (loom 0.7.0 consumes the baton lib)
status: accepted
created: 2026-08-02
agent: qoder
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 280
files_modified: [experiment-loom/Cargo.toml, experiment-loom/src/architecture.rs, experiment-loom/web/index.html, experiment-loom/web/src/plan.ts, experiment-loom/web/src/axon.ts, experiment-loom/web/src/main.ts, experiment-loom/web/src/i18n.ts, experiment-baton/Cargo.toml, Cargo.lock, CHANGELOG.md, docs/adopters/LOOM.md, docs/i18n/es/adopters/LOOM.md, docs/i18n/zh-CN/adopters/LOOM.md, website/i18n/es/docusaurus-plugin-content-docs/current/adopters/LOOM.md, website/i18n/zh-CN/docusaurus-plugin-content-docs/current/adopters/LOOM.md, experiment-baton/PLAN-avance-post-calibracion.md]
observability_scope: none
work_verb: design
design_provenance: new
tags: [baton, loom, intent-overlay, speckit, i18n, gate-5]
related: [AILOG-2026-06-25-004, PLAN-avance-post-calibracion]
---

# AILOG: Baton gate #5 — the intent overlay reaches a server (Loom 0.7.0)

## Summary

Closes graduation gate #5 ("Loom overlay integrado — falta integración con
servidor"). Baton's intent overlay (B4, `overlay.rs`) existed as a pure
computation + CLI report; nothing served it. Loom now consumes the
`straymark-baton` **lib** as a path dependency and folds the overlay into its
existing `GET /api/architecture` response, rendering it as a toggleable third
plane — **Status | Intent** — over the same `plan.drawio` / axonometric
projections. Baton itself needed zero code changes: `speckit::load` and
`overlay::compute` were already `pub`, pure, and codescan-free.

## Design decisions

- **Composition, not duplication.** Loom calls Baton's `speckit::load` +
  `overlay::compute` with its own `load_model(arch_dir)` result and the same
  `collect_source_files` inventory the projection already uses — so
  `--arch-dir` is respected (Baton's private `find_model` candidates would not
  be) and there is exactly one source scan per request.
- **One plane, not stacked badges.** A Status ⇄ Intent toggle beside the
  2D|3D switch; legend, cell/box coloring, and the detail badge all switch
  with the plane. The intent palette deliberately avoids the status hues;
  components the overlay doesn't cover paint a muted neutral.
- **Degraded cases hide the UI.** No `.specify/` memory or zero intended
  components → `intent: null` and the toggle never appears; no `model.yml` →
  the existing `plan.empty` message wins. The 2D renderer, the 3D renderer,
  and the detail panel share one intent cache populated by whichever fetched
  `/api/architecture` last.
- **Baton 0.2.1 is a lib-consumption bump only** — no CLI-visible change, no
  new `baton-*` GitHub release.

## What changed

- `experiment-loom`: 0.6.2 → **0.7.0**; `ArchResponse` gains
  `speckit_present` + `intent: Option<Vec<ComponentIntent>>`; two fixture
  tests (SpecKit project → expected overlay states incl. matched intent and
  `modeled` flags; non-SpecKit root → `intent: None`).
- `experiment-loom/web`: `#plane-mode` toggle + CSS (gated on
  `body.view-plan.plane-available`), plane machinery in `plan.ts`
  (`activePlane`/`setPlane`/`intentColor`/availability hook), plane-aware box
  coloring + attention glow for `intended-not-implemented` in `axon.ts`,
  legend branching + button wiring in `main.ts`, 8 i18n keys ×3 locales.
- Docs: CHANGELOG (Loom 0.7.0 + Baton 0.2.1 note), LOOM.md adopter paragraph
  (en/es/zh-CN, docs + website mirrors), gate #5 ✅ in
  `PLAN-avance-post-calibracion.md`.

## Verification

Fixture-based backend tests cover the fold (intended-and-implemented /
intended-not-implemented / implemented-not-intended + the absent case).
`cargo test -p straymark-loom -p straymark-baton` + `npm run build` in
`experiment-loom/web` handed to the user (agent terminal non-functional this
session). Manual dogfood: the straymark repo itself has SpecKit memory under
`experiment-baton/.specify` and `experiment-loom/architecture/model.yml`.

## EU AI Act Considerations

Not applicable — read-only visualization of project metadata; no model
inference, no personal data.
