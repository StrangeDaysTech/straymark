# Contributors documentation

Resources for people contributing to StrayMark — whether reading the code, writing translations, proposing changes, or trying to understand the *why* behind the framework's shape.

**Languages:** English | [Español](../i18n/es/contributors/README.md) | [简体中文](../i18n/zh-CN/contributors/README.md)

---

## Concepts

The atemporal material — read these to understand *what* StrayMark is and *why* it has the shape it has, before opening a PR that touches the framework's surface.

| Document | What it covers |
|---|---|
| [`DESIGN-PRINCIPLES.md`](DESIGN-PRINCIPLES.md) | The twelve hierarchical principles that govern product decisions. Includes v0.2 empirical annotations from validation cycles on principles #6 (cognitive discipline), #9 (simplicity), and #12 (velocity = velocity of learning). |
| [`WHAT-IS-A-CHARTER.md`](WHAT-IS-A-CHARTER.md) | Conceptual scope of the Charter artifact: an ex-ante declaration of a unit of work with a verification contract and an audit anchor. Maps the boundary between what GitHub SpecKit's `plan.md` covers and what a StrayMark Charter covers — they are not the same thing. |

## Workflow guides

| Document | What it covers |
|---|---|
| [`TRANSLATION-GUIDE.md`](TRANSLATION-GUIDE.md) | Rules and conventions for translating StrayMark documentation into additional languages. Read before submitting a PR that adds or modifies an `i18n/` file. |

## Historical proposals (archived)

Proposals and roadmaps from the project's pre-CLI evolution, preserved for context — they explain how the current shape emerged. They are **not** maintained going forward; the canonical source for current behavior is the code, the schemas under `dist/.straymark/schemas/`, and the CHANGELOG. Browse them on GitHub at [`docs/decisions/proposals/`](https://github.com/StrangeDaysTech/straymark/tree/main/docs/decisions/proposals):

| File | Snapshot date | What it captured |
|---|---|---|
| `2026-04-30-thesis-validation.md` | 2026-04-30 | Empirical validation of the product thesis against six Sentinel cycles (Go backend) — the evidence body that motivated the v0.2 annotations of `DESIGN-PRINCIPLES.md`. |
| `2026-04-30-charter-telemetry.md` | 2026-04-30 | Telemetry instrumentation schema for observing Charter execution in real projects. The normative version now lives in `dist/.straymark/schemas/charter-telemetry.schema.v0.json`. |
| `2026-05-03-cli-roadmap.md` | 2026-05-03 | Three-phase implementation roadmap for the Rust CLI, with closure criteria. Phases 1–3 are now shipped in `cli-3.x`. |
| `2026-05-03-audit-skills-design.md` | 2026-05-03 | Design of the `/straymark-audit-prompt` and `/straymark-audit-review` skills as human-in-the-loop checkpoints. Implemented in `fw-4.8.0`. |
| `2026-05-03-audit-skills-rollout.md` | 2026-05-03 | Operational rollout plan for the audit skills (gating criteria, telemetry, phased shipping). |
| `2026-05-04-audit-cli-flow.md` | 2026-05-04 | Redesign of the external-audit flow after the first empirical encounter with a multi-commit L Charter (Sentinel CHARTER-07). Implemented in `cli-3.10+`. |

Decision records (ADRs) for the live codebase live on GitHub at [`docs/decisions/`](https://github.com/StrangeDaysTech/straymark/tree/main/docs/decisions).

---

*See also: [`../adopters/`](../adopters/) for documentation aimed at teams adopting StrayMark in their own projects, including [`ADOPTION-GUIDE.md`](../adopters/ADOPTION-GUIDE.md), [`CLI-REFERENCE.md`](../adopters/CLI-REFERENCE.md), and [`WORKFLOWS.md`](../adopters/WORKFLOWS.md).*
