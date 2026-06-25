---
id: AILOG-2026-06-25-006
title: Baton — activation seam (SpecKit before_implement hook) implemented + dogfooded
status: accepted
created: 2026-06-25
agent: claude-code-opus-4.8
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 520
files_modified: [experiment-baton/src/coherence.rs, experiment-baton/src/main.rs, experiment-baton/Cargo.toml, experiment-baton/extension/straymark, experiment-baton/specs/002-activation-seam, experiment-baton/tests/activation.rs, experiment-baton/CHARTER-02-activation-seam.md]
observability_scope: none
tags: [baton, activation, speckit, extension, hook, coherence]
related: [CHARTER-02-activation-seam, AILOG-2026-06-25-005]
---

# AILOG: Baton — activation seam

## Summary

Implemented `CHARTER-02-activation-seam` (closes #316): a StrayMark **SpecKit
extension** that hooks **`before_implement`** to run the Phase-1 coherence engine
**at authoring time**, scoped to the active feature, so a consumer is never coded
against a stale/assumed contract (the #304 pattern). Dogfooded read-only on
Sentinel: the hook for `005-frontend-dashboard` surfaces the real #304-class C4
finding, advisory, zero mutation. Touches no models. Charter closed.

## Context

Phase 1 (CHARTER-01) detected drift on demand; the activation seam (the 2nd of the
three integration seams in `02-speckit-integration-research.md`) closes the loop
so the signal arrives **before** code is written. The precedent is SpecKit's
`agent-context` extension (runs `after_specify`/`after_plan`); this is the
coherence analogue on `before_implement`.

## Actions Performed

1. **Reconnaissance** — read Sentinel's real `.specify/extensions/{git,agent-context}/`
   (`extension.yml`, `command.md`, `config-template.yml`) to mirror SpecKit's
   extension/hook/command format exactly.
2. **CLI scope (FR1)** — `Finding.contract: Option<String>` (set for C2/C3/C4,
   `None` for repo-wide C1); `CoherenceReport::build_scoped` / `for_spec` filter
   to the contracts a feature consumes; `--spec <id>` flag.
3. **C4 confidence → Medium** — a decision-propagation finding's strength comes
   from the precise decision↔contract link (B5), not from whether a code producer
   was keyed; previously it inherited the (Low) spec-consumer edge confidence and
   was hidden at the `medium` default. Now it surfaces correctly.
4. **Extension package** `extension/straymark/` (FR2–FR5): `extension.yml` (hook
   `before_implement` → `speckit.straymark.coherence-check`), the command `.md`,
   `config-template.yml`, `scripts/bash/coherence-check.sh` (binary discovery →
   feature resolution → scoped run → advisory/block gate, graceful degradation),
   `README.md` (manual install while experimental).
5. **Tests** — manifest wiring + files present + `--spec` scoping (only the
   feature's contracts; repo-wide C1 dropped) + unknown-spec → empty.
6. **Dogfood** — ran the hook script from Sentinel (`BATON_REPO` dev discovery):
   `before_implement` for `005-frontend-dashboard` emits the C4 (`services.public-visibility`
   consumed without referencing PM-001 / `AILOG-2026-04-21-002`); `git status`
   unchanged.
7. Authored `specs/002-activation-seam/`; closed the Charter.

## Modified Files

| File | Description |
|---|---|
| `experiment-baton/src/coherence.rs` | `Finding.contract`; `build_scoped`/`for_spec`; C4 confidence → Medium |
| `experiment-baton/src/main.rs` | `coherence --spec <id>` flag |
| `experiment-baton/Cargo.toml` | `serde_yaml` dev-dependency (manifest test) |
| `experiment-baton/extension/straymark/**` | New — the SpecKit extension (manifest, command, config, script, README) |
| `experiment-baton/specs/002-activation-seam/**` | New — WHAT/HOW/tasks |
| `experiment-baton/tests/activation.rs` | New — manifest + scoping tests |
| `experiment-baton/CHARTER-02-activation-seam.md` | status → closed + closing notes |

## Verification

- `cargo test --workspace` ✓ — 40 `straymark-baton` tests (4 new activation), no
  regressions; `cargo clippy` clean; `bash -n` on the hook script clean.
- Sentinel dogfood: hook surfaces the real C4 at `min_confidence: medium`,
  advisory exit 0, `git status` unchanged (NFR1).

## Impact

Read-only and non-breaking by design: advisory default, and a missing binary
skips with a note rather than blocking the SpecKit `implement` flow. The blocking
field/enum signal (C2/C3) still depends on contract keying (#313), so on Sentinel
the authoring-time finding is the C4 decision-propagation warning.

## EU AI Act Considerations

Not applicable — local developer tooling; no automated decision-making, no
personal data, no model inference.

## Additional Notes

Distribution is manual install while Baton is experimental (extension README). A
formal `straymark-baton` release + bundling the extension via `dist/` are deferred
to graduation. Next strategic decision (deferred by the user): Baton Phase 2 (the
economic router) — gated on the routable-unit decision and a source of cost data.
