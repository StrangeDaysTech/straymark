---
id: AILOG-2026-07-18-001
title: Implement follow-ups premise/verify reframe (AIDEC-2026-07-18-001, #365 Part 1)
status: accepted
created: 2026-07-18
agent: claude-opus-4-8-1m
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 0
files_modified: []
observability_scope: none
tags: [follow-ups, epistemics, cli, governance, self-adoption, adopter-feedback]
related: [AIDEC-2026-07-18-001-followups-as-hypotheses.md]
---

# AILOG: Implement follow-ups premise/verify reframe (AIDEC-2026-07-18-001)

## Summary

Implemented the three layers of the signed [`AIDEC-2026-07-18-001`](../decisions/AIDEC-2026-07-18-001-followups-as-hypotheses.md) — reframing follow-up registry entries as **dated, decaying hypotheses** and placing premise verification at execution, not capture (Weft field report #365 Part 1). Ships as **fw-4.36.0 / cli-3.37.0**. First substantive governed change of the self-adopted repo, so this AILOG lives in the in-force `/.straymark/` tree.

## Context

Three genuine Weft follow-ups (FU-017/FU-016/FU-010) each rested on a premise that was false *at write time* and cheap to falsify at read time. The registry is a speculative buffer — cheap capture is its value — so the only real bug is executing an entry without re-testing its premise. The AIDEC chose the full depth: doc + schema + CLI affordance (both `promote --premise-verified` and a new `verify` verb).

## Actions Performed

1. **CLI parsing** — added optional `premise` / `verified_at` to `followups::Entry` with lenient field aliases (`Verified-at` / `Verified at` / `verified_at`); the generic `set_entry_field` writes them.
2. **CLI shape A** — `followups promote` now surfaces the entry's `Premise` (falling back to `Notes`) with a re-verify reminder and, with `--premise-verified`, stamps `Verified-at`. Advisory: promotion proceeds either way. Threaded the new arg through `charter close`'s auto-promotion caller.
3. **CLI shape B** — new `followups verify FU-NNN` verb (`verify.rs` + `mod.rs` + `main.rs` routing): surfaces the premise, `--premise "..."` records/updates it, `--verified [--at DATE]` stamps `Verified-at`. Read-only with no flags.
4. **CLI status** — `followups status FU-NNN` shows `Premise` / `Verified-at` and nudges to re-verify when an actionable entry's premise is unverified.
5. **Schema + template** — added optional `premise` / `verified_at` to `$defs.entry`; template documents both and the discipline.
6. **Docs (EN + es + zh-CN)** — new "Epistemic status" section in the pattern doc, reconciled the "planning input" framing, documented the fields + CLI verbs, and added the "before acting on an entry (execution)" directive to `AGENT-RULES.md §13`. Updated `CLI-REFERENCE` command docs.
7. **Versioning** — `dist-manifest.yml` → 4.36.0, `cli/Cargo.toml` → 3.37.0 (+ `Cargo.lock`), governance footers, README/CLI-REFERENCE version tables (×3 locales), `CHANGELOG.md`, `CLAUDE.md` command table.

## Decisions Made

- **`premise`/`verified_at` are optional and lenient-parsed** — every existing registry stays valid; `schema_version` stays `v1` (experimental). No data migration.
- **The CLI reminds and records; it never gates.** Promotion proceeds without `--premise-verified`; `verify` with no flags is read-only. Human judgment stays out of the CLI, consistent with `promote`.
- **Both affordance shapes ship** (operator decision 2026-07-18): `verify` covers chores acted on without promotion; `promote`'s reminder covers TDE graduation.

## Verification

- Full workspace `cargo test` green (38 result groups). New unit test (parse + `set_entry_field` round-trip) and 6 integration tests (promote surfacing ± flag, verify record/read-only/unknown, status nudge).
- Clippy: no new warnings in `verify.rs`/`promote.rs`/`status.rs`.
- End-to-end with the release binary: `status` nudge when unverified → `verify` read-only surfacing → `verify --premise --verified` records the corrected premise + stamps `Verified-at` → nudge gone → `promote --premise-verified` surfaces the confirmed premise, promotes, and stamps. Schema JSON well-formed; loads via the `status` advisory validation.

## Risk

No new risk. All schema/CLI additions are additive and optional; no change to `drift` dedup, counters, or existing entry parsing. Behavioral (not enforced) adoption is by design — the discipline lives in the doc + the prompts.

## Modified Files

| File | Change |
|------|--------|
| `cli/src/followups.rs` | `Entry.premise` / `verified_at` + lenient parse + unit test |
| `cli/src/commands/followups/verify.rs` | New `verify` verb |
| `cli/src/commands/followups/promote.rs` | Premise surfacing + `--premise-verified` stamp |
| `cli/src/commands/followups/status.rs` | Show premise/verified-at + re-verify nudge |
| `cli/src/commands/followups/mod.rs` | Wire `verify` module |
| `cli/src/commands/charter/close.rs` | Thread `premise_verified: false` into auto-promote |
| `cli/src/commands/charter/new.rs` | `strip_inline_markup` / `leading_sentences` → `pub(crate)` (reused) |
| `cli/src/main.rs` | `verify` subcommand + `--premise-verified` flag + routing |
| `cli/tests/followups_test.rs` | Integration tests for verify + promote surfacing |
| `cli/Cargo.toml` | Version 3.37.0 |
| `dist/.straymark/schemas/follow-ups-backlog.schema.v1.json` | Optional `premise` / `verified_at` |
| `dist/.straymark/templates/follow-ups-backlog.md` | Document the fields + discipline |
| `dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md` | Epistemic-status section (EN + es + zh-CN) |
| `dist/.straymark/00-governance/AGENT-RULES.md` | §13 execution directive (EN + es + zh-CN) |
| `dist/dist-manifest.yml` | Version 4.36.0 |
| `docs/adopters/CLI-REFERENCE.md` | `verify` + `--premise-verified` docs (EN + es + zh-CN) |
| `CHANGELOG.md` | Framework 4.36.0 / CLI 3.37.0 entry |

## Follow-ups

- None. The AIDEC's "read-only surfacing" and "both shapes" scope shipped in full; the historical-decisions migration is tracked separately in issue #368.
