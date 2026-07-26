---
id: ADR-2026-06-03-001
title: Follow-ups backlog as a first-class entity — schema v1, native CLI, bash script deprecation
status: draft
created: 2026-06-03
updated: 2026-06-03
agent: claude-opus-4-8-1m
confidence: high
review_required: true
# --- Approval workflow (optional, fill at review time) ---
# reviewed_by: <reviewer-id>
# reviewed_at: YYYY-MM-DD
# review_outcome: approved
risk_level: medium
eu_ai_act_risk: not_applicable
iso_42001_clause: []
alternatives_documented: []
api_changes: []
tags: [follow-ups, backlog, crystallization, design-principle-12, cli, governance]
related: [FOLLOW-UPS-BACKLOG-PATTERN, ADR-2026-06-02-001]
supersedes: []
---

# ADR: Follow-ups backlog as a first-class entity — schema v1, native CLI, bash script deprecation

## Status

draft

**Note**: This document was created by an AI agent and requires human review.

> **Immutability Rule**: Once an ADR reaches `accepted` status, it MUST NOT be modified. If
> the decision changes, create a new ADR with `supersedes: ADR-2026-06-03-001`.

## Context

The follow-ups backlog pattern (`dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md`,
introduced from issue #111, refined in fw-4.13.1) is today a **documented convention with a
fully adopter-side execution layer**: the registry file is hand-maintained markdown, the
drift-detection script lives only in the Sentinel adopter's repo, and the agent directives
exist only as a suggested block the adopter must copy into their own `CLAUDE.md`.

Issue #214 (Sentinel post-Etapa-3 triage report, N=91 FUs / 76 AILOGs extracted / 65 open)
plus an operator design review on 2026-06-03 surfaced that the follow-up is structurally a
**second-class citizen** of the framework:

1. **CLI-invisible.** `discover_documents()` requires the `TYPE-YYYY-MM-DD-NNN` pattern with
   a registered `DocType` prefix; `follow-ups-backlog.md` matches neither, so `status`,
   `metrics`, `validate`, `audit` and the `explore` TUI (hardcoded `GROUP_DEFS`) never see
   it. The only follow-up datum the CLI knows is the `outcome_new_followups: u32` telemetry
   scalar at `charter close`.
2. **Agent-invisible.** Because the session-start / pre-commit directives do not travel in
   `STRAYMARK.md` / `AGENT-RULES.md`, agents asked "what's pending?" scan AILOGs directly and
   ignore the registry — defeating its purpose.
3. **Recurring empirical friction (issue #214).** Signal 1: the `--apply` extractor appends
   entries already resolved in-Charter (20–75% noise per batch, second occurrence). Signal 2:
   frontmatter counters drift silently (`total_open: 47` declared vs 65 real after 4 weeks).
   Signal 3: a severity dimension (`PROD-BLOCKER`) grew ad-hoc inside `Notes`, outside the
   v0 schema.
4. **The backlog is more than deferred chores.** Sentinel's experience shows FUs feed
   planning directly — they originate not only ex-ante but from execution reports
   (telemetry, staging bugs in real environments) and reshape upcoming Charters,
   mini-charters and chores. The registry is the **ex-post counterpart of SpecKit**:
   SpecKit feeds planning from intent; the backlog feeds it from execution reality.
5. **Internal roadmap demand.** Loom (ADR-2026-06-02-001) will visualize the governance
   graph; the FU is a missing node category — precisely the transversal artifact (AILOG
   origin → Charter/TDE destination) the knowledge graph exists to show.

The Charter entity provides the in-repo precedent for promoting a non-`DocType` artifact to
first-class: its own file pattern and canonical path, its own JSON schema, its own CLI
namespace, a synthetic `explore` group, and its own `STRAYMARK.md` section (§15). Charter
matured along fw-4.4.0 (entity) → fw-4.12.0 (discoverability) → fw-4.16.0 (evolution
patterns); the follow-up sits today where Charter was before fw-4.4.0.

## Decision

Crystallize the follow-ups backlog as a **first-class framework entity** following the
Charter lane, across one framework release (fw-4.21.0) and one CLI release (cli-3.19.0):

1. **Registry stays a single file** (`.straymark/follow-ups-backlog.md`) — the per-AILOG
   extraction model is the empirically strongest part of v0 (0 false positives across 76
   AILOGs). No file-per-FU.
2. **Schema v1 (experimental)** published as
   `dist/.straymark/schemas/follow-ups-backlog.schema.v1.json`, adding four optional entry
   dimensions: `severity` (canonicalizing Sentinel's ad-hoc `PROD-BLOCKER` as `blocking`),
   `origin_class` (`ex-ante-planning | testing | telemetry | staging | real-env-bug`),
   `labels` (free tags for grouping FUs into planned Charters/mini-charters), and a formal
   `destination` vocabulary (`chore | mini-charter | charter-replanning | TDE | operations |
   <charter-id>`); plus a new `suspected-closed` entry status.
3. **Native CLI namespace** `straymark followups list / status / drift / promote`,
   replacing the adopter-side bash script. `drift --apply` gains the anti-noise refinement
   (in-AILOG closure markers → `suspected-closed` instead of `ready`; resolves #214
   Signal 1) and **recomputes the `total_*` counters on every write** — counters become
   CLI-owned (resolves #214 Signal 2). `promote FU-NNN` automates the 3-step FU → TDE flow.
4. **Discoverability**: synthetic `Follow-ups` group in the `explore` TUI (sub-nodes per
   bucket), a Follow-ups block in `straymark status`, a registry template in
   `dist/.straymark/templates/`, and a new `STRAYMARK.md` §16.
5. **Agent directives travel with the framework**: session-start glance, pre-commit
   `followups drift --apply`, and post-charter-close review/promotion move into
   `AGENT-RULES.md` (shipped, not copy-pasted).
6. **Zero-friction migration**: permanent lenient parsing of v0 registries; idempotent,
   non-destructive auto-upgrade to v1 on the first write command. No `migrate` subcommand.
7. **Loom hook (deferred)**: FU becomes a knowledge-graph node type in Loom M1, gated on
   the `straymark-core` extraction (Loom M0). The parser is born in `cli/src/followups.rs`
   as pure functions with an explicit "move target: straymark-core" annotation; Loom specs
   are not touched now.

### Design principle #12 — what is relaxed, why, and what would revert it

Issue #135 deferred the CLI surface ("Tier 4") behind a second-adopter gate, citing
principle #12 (*the product's velocity is the velocity of learning*). This ADR explicitly
relaxes that reading, per the "How to use" protocol of `DESIGN-PRINCIPLES.md`:

- **What the principle protects** is not the number "N≥3" but premature crystallization of
  a single adopter's vocabulary without empirical pressure. Its own v0.2 annotation accepts
  *structural diversity* as evidence equivalent to N≥3.
- **The evidence now available**: 91 FUs across a 4-week registry lifetime; schema already
  iterated under empirical pressure (v0 → v0.1, fw-4.13.1); 0 extraction false positives
  across 76 AILOGs and ~10 `--apply` runs; bucket vocabulary stable at N=91 (no sixth
  bucket needed); two prose conventions (`severity`, reclassification provenance) grown
  organically and now canonicalized rather than invented; and **internal demand from the
  framework's own roadmap** (Loom node model) independent of any adopter.
- **What is preserved**: the schema ships as **v1 experimental** (same mark as
  `charter.schema.v0.json`); hard stabilization (v1.0 stable, breaking-change freeze)
  remains gated on a second adopter in another domain, per the principle's operational
  form. Issue #214 is reclassified from "gate currency" to direct design input; issue #135
  is updated accordingly.
- **What would revert this decision**: a second adopter whose usage contradicts the v1
  vocabulary (buckets, statuses, the four new dimensions) before stabilization — in that
  case v1 iterates or is superseded, which is exactly the experimental mark's purpose.

## Alternatives Considered

### 1. FU as a `DocType` (one file per follow-up)
- **Description**: Promote FU to the 16-type document taxonomy: `FU-YYYY-MM-DD-NNN-*.md`
  files, template, `straymark new -t fu`.
- **Pros**: Maximum citizenship for free — all existing discovery/validation machinery
  applies; trivially visible to Loom as document nodes.
- **Cons**: 91 files in Sentinel today; FUs are born as AILOG bullets and most die as a
  one-line closure — per-file overhead is hostile to that lifecycle; destroys the proven
  per-AILOG extraction granularity; the registry's single-glance "pulse" is lost.
- **Why not**: The single-registry model is the load-bearing, empirically validated design
  choice of #111. First-class citizenship must come to the registry, not replace it.

### 2. Ship the bash script in `dist/` (Tier 2 of #135) and stop there
- **Description**: Move `check-followups-drift.sh` into `dist/.straymark/scripts/`,
  keep everything else as-is.
- **Pros**: Cheapest (~1-2h); zero CLI risk; respects #135's literal tier ladder.
- **Cons**: Resolves none of the structural problems: still invisible to `explore`/`status`,
  directives still don't travel, counters still drift, noise pattern needs bash surgery,
  and Loom still has no parser to reuse. #214 itself reports no Tier-2 signal.
- **Why not**: It mechanizes the convention without granting citizenship — the actual gap
  identified by the operator review.

### 3. Parser born in `straymark-core` (anticipate the Loom workspace refactor)
- **Description**: Create the `core/` crate now and place `followups.rs` there from birth.
- **Pros**: No later move; Loom M0 gets the FU parser for free.
- **Cons**: Pays the workspace + crates.io publication cost before Loom exists; couples
  this release to the Loom timeline; violates "prototypes before features" — exactly what
  principle #12 *does* prohibit on current evidence.
- **Why not**: `charter.rs` proves the pattern: pure parsing functions in `cli/src/`,
  mechanically movable at M0. A doc-comment marks the move target.

### 4. Dedicated `straymark followups migrate` subcommand for v0 → v1
- **Description**: Explicit migration step for existing registries (Sentinel's 91 FUs).
- **Pros**: Explicit, auditable migration moment.
- **Cons**: All v1 fields are optional and additive — there is no destructive
  transformation to gate; a manual step will be forgotten; extra CLI surface and tests for
  a one-shot event.
- **Why not**: Lenient parsing + idempotent auto-upgrade on first write achieves zero
  migration friction with no new surface.

## Consequences

### Positive
- The registry becomes visible to every CLI surface (`explore`, `status`) and to agents
  (directives ship in `AGENT-RULES.md`) — the root cause of "agents ignore the file" is
  removed.
- The three friction signals of #214 are resolved structurally (`suspected-closed`,
  CLI-owned counters, canonical `severity`), not absorbed manually each triage.
- The four new schema dimensions make the backlog a queryable planning input
  (`followups list --severity blocking`, `--label <charter-candidate>`), supporting the
  Charter/mini-charter planning loop Sentinel performs by hand today.
- Loom gains a ready, pure-Rust FU parser to lift into `straymark-core` at M0.

### Negative
- Maintenance of a markdown-body parser (semi-structured, lenient) in the CLI — mitigated
  by mirroring the proven `charter.rs` approach and validating against Sentinel's real
  registry.
- The bash reference implementation is deprecated; Sentinel carries a (functional but
  unmaintained) script until it switches to `followups drift`.
- Counters become CLI-owned: operators who hand-edited `total_*` lose that habit (first
  `--apply` corrects any divergence — by design).

### Neutral
- One more entity follows the Charter lane (schema + namespace + synthetic TUI group),
  reinforcing the non-`DocType` first-class pattern as a framework idiom.
- Issue #135's tier ladder is superseded for Tiers 2/4 (collapsed into the native CLI);
  Tier 3 (`charter close` soft-integration) remains open and gated.

### Quality Impact Assessment

| Quality Characteristic (ISO 25010:2023) | Impact | Description |
|-----------------------------------------|--------|-------------|
| Functional Suitability | + | The pending-work registry becomes discoverable, queryable and agent-visible |
| Reliability | + | CLI-owned counters and drift detection remove two silent-divergence classes |
| Maintainability | ~ | New lenient markdown parser to maintain; mirrors existing `charter.rs` idiom |
| Compatibility | + | v0 registries parse forever; upgrade is additive and idempotent |
| Interaction Capability | + | `explore`/`status` surface the backlog pulse without opening the file |
| Flexibility | + | Schema v1 dimensions absorb the two ad-hoc prose conventions with room for N=2 feedback |

## Affected Components

| Component | Type of Change | Impact |
|-----------|----------------|--------|
| `dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md` (+ i18n es/zh-CN) | Rewrite v0 → v1 | High |
| `dist/.straymark/schemas/follow-ups-backlog.schema.v1.json` | New | Medium |
| `dist/.straymark/templates/follow-ups-backlog.md` | New | Low |
| `dist/STRAYMARK.md` (§16) + `AGENT-RULES.md` + `QUICK-REFERENCE.md` + `DOCUMENTATION-POLICY.md` (+ i18n) | Modified | Medium |
| `cli/src/followups.rs` + `followups_schema.rs` + `commands/followups/*` | New | High |
| `cli/src/main.rs`, `cli/src/tui/index.rs`, `cli/src/commands/status.rs` | Modified | Medium |
| Skills `/straymark-followups` (`.claude`/`.gemini`/`.codex`/`.agent` surfaces) | New | Low |
| Issues #214, #135 | Process update | Low |

## Implementation Plan

1. PR 1 — this ADR (`docs/decisions/`, no release).
2. PR 2 — Framework fw-4.21.0: schema v1, pattern doc rewrite, registry template,
   `STRAYMARK.md` §16, `AGENT-RULES.md` directives, i18n ×2, version bump.
3. PR 3 — CLI cli-3.19.0: `followups.rs` parser (lenient v0/v1), `list/status/drift/promote`,
   TUI group, `status` block, ~12 integration tests, version bump.
4. PR 4 — process close-out: #214 / #135 comments, closing AILOG (dogfooded through the new
   `followups drift --apply`).

Sequential PRs, squash-merged — no stacking (per the #129/#131/#133 lesson in `CLAUDE.md`).

## Success Metrics

- The new CLI parses Sentinel's production registry (91 FUs) with zero errors and reports
  the real open count (65) regardless of stale frontmatter counters.
- A drift `--apply` batch over AILOGs containing in-Charter closure markers produces
  `suspected-closed` entries instead of `ready`/TBD noise (Signal 1 cost eliminated).
- After any write command, `grep -c '**Status**: open'` equals frontmatter `total_open`
  (Signal 2 invariant, previously manual).

## Validation Criteria

| Metric | Target Value | Measurement Method | Timeline |
|--------|-------------|-------------------|----------|
| v0 lenient parsing | 0 errors on Sentinel registry | run `followups status` on a copy | cli-3.19.0 |
| Counter invariant | recomputed == real on every write | integration test + Sentinel smoke | cli-3.19.0 |
| Anti-noise refinement | closure-marked bullets → `suspected-closed` | integration test | cli-3.19.0 |
| Test suite | green (121 existing + ~12 new) | `cargo test` | cli-3.19.0 |
| Agent adoption | agents consult registry at session start | observed in Sentinel sessions post-update | post-release |

## References

- Issue #111 — original RFC (registry + drift script, Sentinel CHARTER-12)
- Issue #135 — automation roadmap, 4 tiers + crystallization gates (superseded in part)
- Issue #214 — post-stage triage report at N=91 (Signals 1–4; design input for this ADR)
- `docs/contributors/DESIGN-PRINCIPLES.md` §12 + v0.2 annotation ("the spirit of N≥3")
- `cli/src/charter.rs` — reference implementation of the non-`DocType` first-class lane
- ADR-2026-06-02-001 — Loom stack (straymark-core extraction; FU node type gated on M0)

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-06-03 | claude-opus-4-8-1m | Initial creation (draft, pending human review) |

<!-- Template: StrayMark | https://strangedays.tech -->
