## Loom — intent plane: Baton's overlay served in the Architecture panel (loom 0.7.0)

Closes Baton graduation gate #5 (*"Loom overlay integrado — falta integración con servidor"*). Baton's intent overlay (`overlay.rs`, B4) existed as a pure computation + CLI report; nothing served it. Loom now consumes the `straymark-baton` **lib** as a path dependency and folds the overlay into its existing `GET /api/architecture` response, rendering it as a toggleable third plane — **Status | Intent** — over the same `plan.drawio` / axonometric projections. Baton needed **zero code changes**: `speckit::load` and `overlay::compute` were already `pub`, pure, and codescan-free.

## Changes

### Backend (`experiment-loom`)

- [`Cargo.toml`](experiment-loom/Cargo.toml): `straymark-baton` path dependency; version 0.6.2 → **0.7.0**.
- [`src/architecture.rs`](experiment-loom/src/architecture.rs): `ArchResponse` gains `speckit_present` + `intent: Option<Vec<ComponentIntent>>`. `build_architecture` composes Baton's `speckit::load` + `overlay::compute` with Loom's own `load_model(arch_dir)` result (`--arch-dir` respected — Baton's private `find_model` candidates would not be) and the same `collect_source_files` inventory the projection already uses: one source scan per request, planes can't disagree. No new endpoint; `component_detail` untouched.
- Two fixture tests: SpecKit project → all three overlay states with matched-intent and `modeled` assertions; non-SpecKit root → `intent: None`.

### Frontend (`experiment-loom/web`)

- **Status | Intent toggle** (`#plane-mode`) beside the 2D|3D switch, visible only when the backend reports intent entries (`body.view-plan.plane-available`); hidden otherwise — non-SpecKit projects never see it.
- Intent palette deliberately off the status hues (teal / amber / violet); components the overlay doesn't cover paint a muted neutral. Legend, 2D cell coloring, 3D box coloring (attention glow moves to `intended-not-implemented`), and the component-detail badge (state + matched SpecKit slug) all switch with the plane. 2D renderer, 3D renderer, and detail panel share one intent cache populated by whichever fetched last.
- 8 i18n keys × 3 locales (en/es/zh-CN).

### Bookkeeping

- `straymark-baton` 0.2.0 → **0.2.1** — lib-consumption bump only; no CLI-visible change, no new `baton-*` release.
- CHANGELOG: Loom 0.7.0 section. LOOM.md adopter paragraph (en/es/zh-CN, docs + website mirrors). Gate #5 ✅ in [`PLAN-avance-post-calibracion.md`](experiment-baton/PLAN-avance-post-calibracion.md).

## Verification

- Fixture tests cover the fold both ways (`cargo test -p straymark-loom -p straymark-baton`).
- `npm run build` in `experiment-loom/web` (release-loom.yml asserts `dist/index.html`).
- Dogfooded live: with scratch SpecKit memory docs, the toggle appeared, all three states rendered in 2D and 3D, legend/badge switched; without them the toggle stayed hidden (degraded case). Scratch files removed afterwards.

## After merge

```bash
git tag loom-0.7.0 && git push origin loom-0.7.0
```

`release-loom.yml` builds the frontend + binary; the path dependency compiles baton from the tag checkout — no workflow change needed.
