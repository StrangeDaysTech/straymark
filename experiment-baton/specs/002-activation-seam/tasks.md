# Tasks 002 — Baton activation seam

> **Spec:** [spec.md](spec.md) · **Plan:** [plan.md](plan.md) · **Charter:** [CHARTER-02-activation-seam.md](../../CHARTER-02-activation-seam.md)

- [x] T1 — Reconnaissance: read Sentinel's real `.specify/extensions/{git,agent-context}/`
  (`extension.yml` + a `command.md` + `config-template.yml`) to mirror the format.
- [x] T2 — `Finding.contract` field; `CoherenceReport::build_scoped` / `for_spec`;
  `--spec <id>` CLI flag (FR1). C4 confidence → `Medium`.
- [x] T3 — Extension package `extension/straymark/`: `extension.yml` (hook
  `before_implement` → command, FR2), command `.md`, `config-template.yml`,
  `coherence-check.sh` (binary discovery + feature resolution + gate, FR3/FR4/FR5),
  `README.md`.
- [x] T4 — Tests: manifest validation (hook wired), extension files present,
  `--spec` scoping keeps only feature contracts, unknown spec → empty.
- [x] T5 — Dogfood read-only on Sentinel: `before_implement` for
  `005-frontend-dashboard` surfaces the real #304-class C4; `git status` unchanged.
- [x] T6 — AILOG + Charter closure (status → closed); spec set authored.

## Verification

- [x] `cargo test --workspace` green (40 baton tests); `cargo clippy` clean.
- [x] Hook dogfood on Sentinel surfaces the C4 at `min_confidence: medium`,
  advisory exit 0, zero mutation.
