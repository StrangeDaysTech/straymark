# Implementation Plan 002 — Baton activation seam

> **Spec:** [spec.md](spec.md) · **Charter:** [CHARTER-02-activation-seam.md](../../CHARTER-02-activation-seam.md)

## Architecture

```
SpecKit  before_implement hook  ──▶  .specify/extensions/straymark/
                                       extension.yml  (hook → command)
                                       commands/…coherence-check.md
                                       scripts/bash/coherence-check.sh
                                                 │ discovers binary, resolves feature
                                                 ▼
                                       straymark-baton coherence . --spec <feature>
                                                 │ (Phase-1 engine, read-only)
                                                 ▼
                                       findings surfaced to the agent (gate: advisory|block)
```

The engine is unchanged except for the `--spec` scope filter; the extension is
pure packaging + a thin shell that wraps the existing CLI.

## Pieces

1. **CLI scope** — `Finding.contract: Option<String>` (set for C2/C3/C4, `None`
   for repo-wide C1) + `CoherenceReport::build_scoped(root, Some(spec))` /
   `for_spec`, which keeps findings whose contract the feature consumes. C4
   confidence is now `Medium` (a decision-propagation finding's strength comes
   from the precise decision↔contract link, not from a code producer).
2. **Extension package** (`extension/straymark/`): `extension.yml`,
   `commands/speckit.straymark.coherence-check.md`, `config-template.yml`,
   `scripts/bash/coherence-check.sh`, `README.md` — mirroring SpecKit's `git` /
   `agent-context` extensions (verified format in Sentinel's `.specify/`).
3. **Binary discovery** in the script: config `binary:` → `PATH` →
   `cargo run -p straymark-baton` in `$BATON_REPO` (dev). Absent → skip + note.

## Risks (from Charter)

R1 binary discovery (mitigated: 3-tier + graceful skip), R2 hook cost (mitigated:
feature-scoped), R4 false positives at authoring (mitigated: advisory default,
`min_confidence: medium`), R5 experimental binary distribution (manual install
documented in the extension README).

## Phasing

Single increment (effort M): CLI scope → extension package → tests → Sentinel
dogfood → close.
