---
id: AILOG-2026-07-26-001
title: Follow-ups mutation + ex-ante creation verbs (CHARTER-01, #355 + #360)
status: accepted
created: 2026-07-26
agent: claude-opus-5-1m
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 0
files_modified: []
observability_scope: none
tags: [follow-ups, cli, governance, adopter-feedback, self-adoption, charter-01]
related: [AIDEC-2026-07-18-001]
---

# AILOG: Follow-ups mutation + ex-ante creation verbs (CHARTER-01)

## Summary

Executed [CHARTER-01](../../charters/01-follow-ups-registry-mutation-and-ex-ante-creation.md): three new verbs — `followups note`, `followups set-status`, `followups new` — closing the last flows that required hand-editing the CLI-parsed registry (Weft field reports #355, #360). Ships as **fw-4.37.0 / cli-3.39.0**. First *executed* Charter of the self-adopted repo.

## Context

Every pre-existing `followups` verb either read the registry or wrote it as a side effect of extraction. Mutating an entry meant hand-editing markdown the CLI parses; creating one at Charter-declaration time was impossible (both population paths assume an ex-post origin). The #360 hazard was live in the adopter's tree: `CHARTER-06` cited `FU-011` three times as a forward reference with nothing reserving it, and ids are minted `max(existing) + 1` at extraction, so the next unrelated `drift --apply` would have silently repointed those citations.

## Actions Performed

1. **Shared write path** — `followups::write_recounted` / `recounted_frontmatter`: recompute the CLI-owned counters *from the body about to be written* and persist in one call. `recount` and `verify` were refactored onto it, so there is one implementation of the arithmetic rather than two that could disagree.
2. **`note`** — appends to `Notes` via the pure `append_note` composer: `[date]` or `[date · source]` stamp, composed onto any existing value (the field is single-line by parser contract, so annotations compose instead of stacking).
3. **`set-status`** — validates against the `FuStatus` vocabulary and writes status + counters together. Two refusals rather than silent damage: an out-of-vocabulary value (the lenient parser would degrade it to `unknown`, dropping the entry from every counter without failing) and `promoted` (redirected to `followups promote`, which writes the TDE the status points at).
4. **`new`** — mints an ex-ante entry through `render_declared_entry` (a deliberate sibling of `render_new_entry`, without its AILOG-derived `Source-hash`), inserts it into the requested bucket, recounts, and prints the assigned id.
5. **Guard for all three** — `guard_parse_warnings` aborts before any write when the registry has parse warnings.
6. **Framework** — pattern doc (EN + es + zh-CN) documents the verbs and the ex-ante origin; the registry template states the write path and that entries are never hand-edited; the `/straymark-followups` skill uses `set-status` + `note` for triage and gains a Charter-declaration step.
7. **Docs + release** — CLI-REFERENCE ×3 locales, `CLAUDE.md` command table, version tables, `CHANGELOG.md`, `dist-manifest.yml` → 4.37.0, `cli/Cargo.toml` → 3.39.0, governance footers.

## Decisions Made

- **The schema needed no change.** The Charter listed "confirm the ex-ante entry shape validates without `Source-hash`; adjust `$defs.entry` if not". It validates as-is: `origin` is required (so `--origin` is required, not optional), `origin_class` already carries **`ex-ante-planning`** in its enum, and `source_hash` is not a schema property at all. **The schema had already named the ex-ante case; only the creation path was missing.** That reframes #360 from "add a field" to "the vocabulary was there, the verb was not".
- **`--origin` is required** rather than defaulted. Schema v1 requires it, and an entry whose origin is guessed is the kind of half-truth the registry exists to prevent.
- **Two fixes to shared helpers, not to my new code** (see `## Risk` R6): the spacing defects belonged to `set_entry_field` / `insert_into_bucket` and were degrading the file on every `drift`/`promote`/`verify` write. Fixing them at the root rather than working around them in `note` is why the diff touches those two functions.
- **Deferred options stayed deferred.** No id-reservation primitive and no Charter-scanning in `drift`, exactly as declared. Nothing in execution argued for either.

## Verification

- Full workspace `cargo test`: **38 groups green**. 10 new integration tests (one per verb path: happy path, composition, malformed-registry abort with byte-identical file, unknown/`promoted` status refusal, no-op same-status, id assignment, bucket routing + `TBD` defaults, required-origin/bucket validation, and the R4 `new`-then-`drift` no-duplicate check) plus 5 new unit tests (`append_note` composition incl. whitespace-only inputs, `render_declared_entry` shape + parser round-trip, premise omission, and the two spacing regressions).
- `cargo clippy -p straymark-cli`: no warnings in `note.rs` / `set_status.rs` / `new.rs`.
- `gen_codex_skills --check`: in sync (15 skills).
- **End-to-end with the built binary** against a scratch registry, running the exact flow the Charter declared: `new` (assigned FU-003, `Origin-class: ex-ante-planning`, no hash) → `note` on an unrelated entry (status unchanged, prior note preserved) → `set-status … closed` (counters moved in the same step) → **`recount` reports "already in sync"** (the desync #355 describes cannot occur) → `list` renders all three → `drift` does not re-extract. Then the guard: appending a malformed `### FU-` heading made `note` exit 1 with nothing written.

## Risk

- **R1 (registry corruption) — mitigated and tested.** The parse-warning guard aborts before any write; the integration test asserts the file is byte-identical after a refused write.
- **R2 (counter desync moves into the verb) — mitigated.** One shared recompute path; the `set-status` test asserts a following `recount` finds nothing to do.
- **R3 (concurrent id minting)** — unchanged and still not claimed: atomicity is scoped to "one command, not two steps", which is the actual #360 hazard.
- **R4 (ex-ante entry confuses `drift` dedup) — verified benign.** `new`-then-`drift --scan-all` produces no duplicate; a duplicate arising from a *later* AILOG mention remains accepted behavior, resolved with `set-status … superseded`.
- **R5 (scope creep into the deferred options)** — did not occur.
- **R6 (new, not in Charter) — two pre-existing markdown-shape defects in shared write helpers.** `set_entry_field` collapsed an entry's trailing `"\n\n"` to `"\n"` (gluing the edited entry to the next `## Bucket:` heading) and `insert_into_bucket` appended entries with no blank line before the following section. Both predate this Charter and were silently degrading the registry's shape on every `drift --apply` / `promote` / `verify` write; making field writes routine via `note` is what exposed them. Fixed at the root with two regression unit tests. Severity low (cosmetic on a human-read artifact, no parse impact), but left unfixed it would have made every new `note` call visibly reshape the file.

## Modified Files

| File | Change |
|------|--------|
| `cli/src/followups.rs` | `write_recounted` / `recounted_frontmatter`, `append_note`, `render_declared_entry`, `CANONICAL_BUCKETS`, two shared-helper spacing fixes, 5 unit tests |
| `cli/src/commands/followups/note.rs` | New — `note` verb + the shared parse-warning guard |
| `cli/src/commands/followups/set_status.rs` | New — `set-status` verb |
| `cli/src/commands/followups/new.rs` | New — `new` verb (`NewArgs`) |
| `cli/src/commands/followups/mod.rs` | Wire the three modules |
| `cli/src/commands/followups/recount.rs` | Use the shared recompute path |
| `cli/src/commands/followups/verify.rs` | Use the shared write path (removes a duplicated recount block) |
| `cli/src/main.rs` | Three subcommands + flags + routing |
| `cli/tests/followups_test.rs` | 10 integration tests |
| `cli/Cargo.toml` | Version 3.39.0 |
| `dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md` (+ i18n es, zh-CN) | Verbs + ex-ante origin section |
| `dist/.straymark/templates/follow-ups-backlog.md` | Write path + never-hand-edit note |
| `dist/.claude/skills/straymark-followups/SKILL.md` | Triage via `set-status`/`note` + Charter-declaration step |
| `dist/.codex/skills/straymark-followups/SKILL.md` | Regenerated (`gen_codex_skills`) |
| `dist/.gemini/skills/straymark-followups/SKILL.md`, `dist/.agent/workflows/straymark-followups.md` | Hand-synced (see Closing notes — the generator covers only `.codex`) |
| `dist/dist-manifest.yml` | Version 4.37.0 |
| `dist/.straymark/00-governance/{QUICK-REFERENCE,AGENT-RULES,DOCUMENTATION-POLICY,C4-DIAGRAM-GUIDE,FOLLOW-UPS-BACKLOG-PATTERN}.md` (+ i18n ×2) | fw version footers (see Closing notes) |
| `docs/adopters/CLI-REFERENCE.md` (+ i18n es, zh-CN) | Three verbs documented |
| `README.md` (+ i18n es, zh-CN) | Version tables |
| `CLAUDE.md` | Command table rows |
| `CHANGELOG.md` | fw-4.37.0 / cli-3.39.0 entry |
| `.straymark/charters/01-follow-ups-registry-mutation-and-ex-ante-creation.md` | Status → closed, closing notes |
| `.straymark/charters/README.md` | Row moved to Closed |

## Follow-ups

- None. The Charter's declared scope shipped in full; both deliberately-deferred options (#360 Options B and C) remain deferred with the reasoning recorded in the Charter, and nothing in execution argued for revisiting them.
