---
id: AILOG-2026-08-14-002
title: guard_closure in remediation AILOGs — GUARD-001 + amend template (#419, PR 3 of CHARTER-02)
status: accepted
created: 2026-08-14
agent: qoder-cli-v1.0
confidence: high
review_required: false
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
files_modified:
  - core/src/document.rs
  - core/Cargo.toml
  - experiment-loom/Cargo.toml
  - experiment-baton/Cargo.toml
  - cli/src/validation.rs
  - cli/src/commands/charter/amend.rs
  - cli/Cargo.toml
  - Cargo.lock
  - cli/tests/charter_amend_test.rs
  - STRAYMARK.md
  - dist/STRAYMARK.md
  - dist/dist-manifest.yml
  - dist/.claude/skills/straymark-audit-review/SKILL.md
  - dist/.qoder/skills/straymark-audit-review/SKILL.md
  - dist/.qwen/skills/straymark-audit-review/SKILL.md
  - dist/.codex/skills/straymark-audit-review/SKILL.md
  - dist/.agent/skills/straymark-audit-review/SKILL.md
  - CHANGELOG.md
  - README.md
  - docs/i18n/es/README.md
  - docs/i18n/zh-CN/README.md
  - docs/adopters/CLI-REFERENCE.md
  - docs/i18n/es/adopters/CLI-REFERENCE.md
  - docs/i18n/zh-CN/adopters/CLI-REFERENCE.md
  - dist/.straymark/00-governance/AGENT-RULES.md
  - dist/.straymark/00-governance/C4-DIAGRAM-GUIDE.md
  - dist/.straymark/00-governance/DOCUMENTATION-POLICY.md
  - dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md
  - dist/.straymark/00-governance/QUICK-REFERENCE.md
  - dist/.straymark/00-governance/i18n/es/AGENT-RULES.md
  - dist/.straymark/00-governance/i18n/es/C4-DIAGRAM-GUIDE.md
  - dist/.straymark/00-governance/i18n/es/DOCUMENTATION-POLICY.md
  - dist/.straymark/00-governance/i18n/es/FOLLOW-UPS-BACKLOG-PATTERN.md
  - dist/.straymark/00-governance/i18n/es/QUICK-REFERENCE.md
  - dist/.straymark/00-governance/i18n/zh-CN/AGENT-RULES.md
  - dist/.straymark/00-governance/i18n/zh-CN/C4-DIAGRAM-GUIDE.md
  - dist/.straymark/00-governance/i18n/zh-CN/DOCUMENTATION-POLICY.md
  - dist/.straymark/00-governance/i18n/zh-CN/FOLLOW-UPS-BACKLOG-PATTERN.md
  - dist/.straymark/00-governance/i18n/zh-CN/QUICK-REFERENCE.md
observability_scope: none
tags: [guard-closure, remediation, charter-amend, reference-resolution, adopter-feedback, core, cli, charter-02]
related:
  - AILOG-2026-08-14-001
---

# AILOG: guard_closure in remediation AILOGs (#419, PR 3)

## Summary

Issue #419 case 3: a remediation AILOG closed a Critical finding with prose
alone — the fix landed, but no mechanical check stood between the defect and
its recurrence, and nothing asked for one. PRs 1–2 gave id citations and
registry code claims a resolver; this PR makes the *absence of a recurrence
guard* a first-class, validatable fact. CHARTER-02 closes here.

Defect class covered (design constraint 3): **unguarded remediations** —
findings "fixed" with no barrier against recurrence.

## Decision

Remediation AILOGs carry a `guard_closure:` list — one item per finding being
closed, each declaring **exactly one** of:

- `guard:` — the mechanical check (rule id, CLI flag, CI gate) that prevents
  recurrence; or
- `unguardable:` — a specific rationale when no mechanical guard exists.

An AILOG counts as a remediation AILOG by the presence of `trigger:`
(`external_audit | production_incident | deferred_implementation`) — the
signature `charter amend` already writes. Detection stays structural: no
heuristic guesses at prose intent.

`GUARD-001` (Warning, warn-first — design constraint 1) fires when:

- a remediation AILOG lacks `guard_closure:` or leaves it empty;
- an item sets both `guard` and `unguardable`, or neither; or
- `unguardable` is generic — a stock phrase ("human review", "n/a", "not
  applicable", …) or a rationale under 30 characters. The generic-rationale
  check is the precision-sensitive half: it is heuristic by nature, which is
  exactly why the rule is a warning and not an error.

Pre-existing remediation AILOGs legitimately lack the field, so the flip to
Error waits for a measured adopter baseline (same discipline as REF-003).

`charter amend` scaffolds the field: the amendment template renders a
commented `guard_closure:` block with one placeholder item, so the operator
fills it at authoring time instead of discovering GUARD-001 at validation
time. The `straymark-audit-review` skill's remediation template now requires
a **Guard** line per item, so external audit cycles feed the field upstream.

## Actions Performed

- `core/src/document.rs`: `GuardClosureItem { finding, guard, unguardable }`;
  `Frontmatter` gains `trigger` + `guard_closure` (additive → core 0.10.0;
  experiment-loom / experiment-baton pins follow).
- `cli/src/validation.rs`: `check_guard_closure` wired after
  `check_type_specific` — the three warning conditions above. 5 unit tests.
- `cli/src/commands/charter/amend.rs`: template renders the commented
  `guard_closure:` block after `findings_closed:`.
- `STRAYMARK.md` + `dist/STRAYMARK.md`: §8 frontmatter snippet, §13
  type-table note, §15.B guard-closure paragraph.
- `straymark-audit-review` SKILL.md: Guard line required per remediation
  item — canonical `.claude` edited; `.qoder` / `.qwen` copied;
  `.codex` / `.agent` regenerated via `gen_minimal_skills` (`--check`
  clean).
- Framework 4.44.0 / CLI 3.48.0: CHANGELOG entry, `dist-manifest.yml`
  version, footer stamp sweep (21 files).
- 1 integration test (amend template renders `guard_closure:` and the
  rendered frontmatter parses).

## Risks

- R2 (Charter): GUARD-001 false positives on remediation AILOGs whose guard
  genuinely is "a human looks at it". Mitigation: warn-first, and the
  `unguardable:` escape hatch only demands specificity (≥30 chars, no stock
  phrases), not a different decision. No new instance surfaced during
  implementation.
- R4 (new, not in Charter): release-mechanics files not in the original
  declaration — the experiment-crate core pins, `dist/dist-manifest.yml`,
  and the 21-file footer stamp sweep. Same category as PR 1's Cargo.lock
  row: mechanical followers of a version bump, recorded here and added to
  the Charter table in this same PR per the drift-check protocol.
- R5 (new, not in Charter): `STRAYMARK.md` (repo root) is declared in the
  Charter and was edited on disk — §8, §13, §15.B — but can never appear
  in a git drift range: `.gitignore` excludes it (operator-local mirror
  kept in sync by `straymark update`). The tracked half of the change is
  `dist/STRAYMARK.md`, modified in this PR.

## Validation

- `cargo test --workspace` green (429 unit tests incl. the 5 new GUARD-001
  cases; 5/5 `charter_amend` integration tests).
- Dogfood: `straymark validate . --include-charters` — 0 errors; warning
  count steady (no GUARD-001 fire: this repository's remediation AILOGs
  predate the field, and none is in the changed set — warn-first means even
  a fire would not block).
- Dogfood: `charter amend` against a fixture charter renders the
  `guard_closure:` block; the rendered frontmatter parses.
- `gen_minimal_skills --check` clean after regeneration.

## Follow-ups

- GUARD-001 severity flip to Error: only after the warn-first baseline is
  measured across adopters (tracked alongside REF-003 in the Charter's
  out-of-scope note; FU-007 covers the REF-003 side).
