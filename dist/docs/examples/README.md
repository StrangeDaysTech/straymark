# DevTrail examples — browse-only reference

This directory ships a small set of anonymized real-world examples derived from the Sentinel `/plan-audit` experiment that originated the Charter pattern. They exist for **browsing** — not as scaffolding templates.

`devtrail init` does **not** copy these into adopter projects. The auto-installed manifest paths are `.devtrail/`, `DEVTRAIL.md`, agent skills, and the docs-validation workflow (see `dist/dist-manifest.yml`); `dist/docs/` is excluded by design. To scaffold a real document in your project, use:

- `devtrail new --doc-type ailog --title "..."` → AILOG
- `devtrail charter new --type S|M|L --title "..."` → Charter

## What's here

| File | Type | Effort | Status | Pairs with |
|---|---|---|---|---|
| [`charters/CHARTER-01-anomaly-thresholds.md`](charters/CHARTER-01-anomaly-thresholds.md) | Charter | M | closed | `AILOG-2026-01-15-001` (origin) |
| [`charters/CHARTER-02-baseline-recompute.md`](charters/CHARTER-02-baseline-recompute.md) | Charter | XS | closed | — |
| [`ailogs/AILOG-2026-01-15-001-anomaly-detector-introduction.md`](ailogs/AILOG-2026-01-15-001-anomaly-detector-introduction.md) | AILOG | — | accepted | `CHARTER-01-anomaly-thresholds.md` (follow-up) |

## The canonical "Charter as follow-up of an AILOG" pattern

`AILOG-2026-01-15-001` documents the introduction of an anomaly detector to a Go backend service. Its Risk section flagged static thresholds (3σ/5σ) as a tradeoff acceptable for the MVP, with per-service thresholds recorded as a follow-up. That follow-up later materialized as `CHARTER-01-anomaly-thresholds`, whose `originating_ailogs` frontmatter field references the AILOG.

This pair illustrates the loop the framework is built to capture:

1. An AI-implemented change is logged as an AILOG (ex-post).
2. The AILOG identifies a constraint or follow-up.
3. A Charter declares the bounded follow-up work (ex-ante).
4. The Charter closes with telemetry that, when external audit is run, references the calibrated findings.

`CHARTER-02-baseline-recompute` is a smaller, independent example (XS effort) that does not have a paired AILOG — useful as a reference for the minimal-Charter shape.

## Anonymization

Sentinel-specific identifiers (module names, internal issue numbers, PR refs, infrastructure hostnames, reviewer emails, dates) have been replaced with generic placeholders. Technical reasoning, the Decision section structure, the Risk numbering, and the Verification commands are preserved verbatim per the example-anonymization rules in `Propuesta/devtrail-cli-roadmap.md` §3.1.
