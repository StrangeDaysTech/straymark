---
charter_id: CHARTER-01-follow-ups-registry-mutation-and-ex-ante-creation
status: closed
closed_at: 2026-07-26
effort_estimate: M
execution_ailogs: [AILOG-2026-07-26-001]
trigger: "Two adopter field reports from Weft (#355, #360) filed against the same seam: every path that mutates or creates a registry entry currently goes through hand-editing a CLI-parsed file. #360 documents a live id-collision hazard in the adopter's tree — CHARTER-06 cites FU-011 three times as a forward reference with nothing reserving it."
work_verb: implement
design_provenance: new
---

# Charter: Follow-ups registry: mutation and ex-ante creation verbs

> **Status (mirrored from frontmatter — source of truth is above):** closed 2026-07-26. Effort: M (declared 1–3 days; executed in one session). Execution: `AILOG-2026-07-26-001`. Released as fw-4.37.0 / cli-3.39.0.
>
> **Origin:** adopter field reports [#355](https://github.com/StrangeDaysTech/straymark/issues/355) and [#360](https://github.com/StrangeDaysTech/straymark/issues/360), both from the Weft adopter (registered in #357). No originating AILOG — this Charter precedes execution.

## Context

`straymark followups` exposes `list`, `status`, `drift`, `recount`, `promote`, `verify`. Every one of them either **reads** the registry or writes it **as a side effect of extraction**. There is no verb that mutates an existing entry, and no verb that creates one. The registry can only be populated two ways: `drift --apply` (extracts from AILOGs) and hand-edit + `recount`.

That leaves two gaps, filed independently by the same adopter:

- **#355 — mutation.** Recording that FU-002 received a partial mitigation, or flipping `open → done`, means hand-editing a CLI-parsed markdown file. `recount`'s own help acknowledges this ("reconcile counters after a manual-triage session (statuses flipped **by hand**)"), which sits awkwardly next to the operating guidance *"never edit CLI-owned counters by hand."* The distinction is real — counters are CLI-owned, entry bodies are hand-owned — but the flow it leaves is fragile: a hand-edit can malform an entry and break `list`/`status`/`drift`, the two-step is desyncable (forget `recount` and the counters silently lie), and nothing enforces dated traceability back to the Charter or AILOG that motivated the change.

- **#360 — ex-ante creation.** Both population paths assume the origin is an **ex-post** document. There is no path to mint a follow-up whose origin is a **Charter declaration**. Declaring CHARTER-06, the adopter decided at declaration time to defer a CI job and register the coverage gap — but `drift` scans AILOGs, not Charters, and no AILOG exists yet by design (Charters are ex-ante; AILOGs are ex-post; the gap between them is days). They fell back to forward-referencing `FU-011` in the Charter body with nothing reserving it. Since ids are minted as `max(existing) + 1` at extraction time ([`followups.rs:516`](../../cli/src/followups.rs)), the next unrelated `drift --apply` hands `FU-011` to a different entry and the Charter's three citations silently point at the wrong follow-up — cross-document incoherence produced by the registry's own id assignment.

The registry model itself (buckets, statuses, `Source-hash` dedup, counters, `promote`, the premise/`Verified-at` reframe of fw-4.36.0) is sound. This is a missing-affordance Charter, not a correctness one.

## Scope

**In scope:**

1. **`straymark followups note <FU-NNN> "<text>"`** — append a dated note to the entry's `**Notes**` bullet in one validated edit. `--source AILOG-… | CHARTER-…` records what motivated it, so the annotation carries traceability the hand-edit path never enforced.
2. **`straymark followups set-status <FU-NNN> <status>`** — set `**Status**` *and* recompute the CLI-owned counters in one step, collapsing the edit-then-`recount` two-step and its desync window. Rejects out-of-vocabulary statuses against the existing `Status` enum rather than writing them.
3. **`straymark followups new`** — mint an entry with an **atomically-assigned** id, `--title`, `--bucket` (default `charter-triggered`), `--origin`, and the optional `--cost` / `--trigger` / `--destination` / `--premise` fields, then recompute counters. Prints the assigned `FU-NNN` so the Charter body can cite an id that **already exists**.
4. **Structural safety for all three verbs** — every write goes through the existing surgical helpers (`set_entry_field`, `insert_into_bucket`, `fm_apply_counters_and_v1`) so an entry stays schema-valid by construction; a malformed registry fails loudly *before* any write rather than being half-edited.
5. **Docs** — `FOLLOW-UPS-BACKLOG-PATTERN.md` (EN + es + zh-CN) documents the ex-ante origin (`Origin: CHARTER-NN §Scope`) as a first-class case alongside the ex-post one; `CLI-REFERENCE.md` (×3 locales) documents the three verbs; the `straymark-followups` skill gains the new verbs in its workflow.
6. **`.straymark/charters/README.md`** — the Charter index the closure protocol references. Created with this declaration (this is the repo's first Charter); execution only moves the row.

**Out of scope:**

- **A separate id-reservation primitive (#360 Option C).** Deliberately not built: it exists only to make a forward-reference safe *before* the entry exists, and `new` removes that need by making creation atomic and instant — declare the deferral, get `FU-NNN` back, cite it. A reservation registry would add a second source of truth for id assignment (reserved-but-never-created ids leaking into `next_fu_number` forever, needing their own expiry/GC rules) to solve a window that `new` closes by construction. **If a real case survives `new`** — an id needed while the entry's content is genuinely undecided — it comes back as its own follow-up with that evidence attached.
- **Teaching `drift` to scan Charters (#360 Option B).** `drift`'s dedup is keyed on `Source-hash(ailog_id, section, description)`; extending it to a second document class with different lifecycle semantics (a Charter is edited across its whole life, an AILOG is written once) is a distinct design question. `new` covers the declared case explicitly, which is the one the adopter hit.
- **Human prioritization of any kind.** Consistent with `promote` and `verify`: the CLI records and validates, it never decides. No auto-triage, no severity inference.
- **Bulk / scripted mutation** (`set-status --all-in-bucket` and similar). No adopter signal; would widen the blast radius of a mis-typed command against a governance artifact.

## Files to modify

| File | Change |
|---|---|
| `cli/src/followups.rs` | Reuse `set_entry_field` / `insert_into_bucket` / `next_fu_number` / `compute_counters`; add a `render_declared_entry` sibling to `render_new_entry` (no `Source-hash` — an ex-ante entry has no AILOG to hash) and note-appending support |
| `cli/src/commands/followups/note.rs` | New — the `note` verb |
| `cli/src/commands/followups/set_status.rs` | New — the `set-status` verb |
| `cli/src/commands/followups/new.rs` | New — the `new` verb |
| `cli/src/commands/followups/mod.rs` | Wire the three new modules |
| `cli/src/commands/followups/recount.rs` | Extract the counter-recompute + write into a shared helper the new verbs call (avoid three copies of the frontmatter update) |
| `cli/src/main.rs` | Three subcommands + flags + routing |
| `cli/tests/followups_test.rs` | Integration tests per verb (happy path, unknown id, malformed registry, counter reconciliation, id assignment) |
| `cli/Cargo.toml` | Version bump (minor — new commands) |
| `dist/.straymark/schemas/follow-ups-backlog.schema.v1.json` | Confirm the ex-ante entry shape validates without `Source-hash`; adjust `$defs.entry` required-fields if it does not |
| `dist/.straymark/templates/follow-ups-backlog.md` | Document the ex-ante origin form and the new verbs |
| `dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md` | Ex-ante origin as a first-class case + mutation verbs (EN) |
| `dist/.straymark/00-governance/i18n/es/FOLLOW-UPS-BACKLOG-PATTERN.md` | Same (es) |
| `dist/.straymark/00-governance/i18n/zh-CN/FOLLOW-UPS-BACKLOG-PATTERN.md` | Same (zh-CN) |
| `dist/.claude/skills/straymark-followups/SKILL.md` | New verbs in the skill workflow (source of record for the generated variants) |
| `dist/.codex/skills/straymark-followups/SKILL.md` | Regenerated via `cargo run --bin gen_codex_skills --features dev-tools` — CI checks drift |
| `dist/.gemini/skills/straymark-followups/SKILL.md` | Regenerated (same generator) |
| `dist/.agent/workflows/straymark-followups.md` | Regenerated (same generator) |
| `dist/dist-manifest.yml` | Framework version bump (governance docs + template changed) |
| `docs/adopters/CLI-REFERENCE.md` | Document the three verbs (EN) |
| `docs/i18n/es/adopters/CLI-REFERENCE.md` | Same (es) |
| `docs/i18n/zh-CN/adopters/CLI-REFERENCE.md` | Same (zh-CN) |
| `README.md` | Version table |
| `docs/i18n/es/README.md` | Version table |
| `docs/i18n/zh-CN/README.md` | Version table |
| `CLAUDE.md` | CLI command table rows for the three verbs |
| `CHANGELOG.md` | Framework + CLI entry |
| `.straymark/charters/README.md` | Move this Charter's row to `## Closed` with the PR reference (created alongside this declaration) |
| `.straymark/07-ai-audit/agent-logs/AILOG-...md` | New, `risk_level: low` |

## Verification

### Local checks

```bash
# Build & test (workspace root)
cargo build
cargo test

# Lint — the new command modules must not add warnings
cargo clippy -p straymark-cli

# Generated agent-skill variants must not drift (CI enforces the same check)
cargo run --quiet --bin gen_codex_skills --features dev-tools -- --check

# Governance self-check on this repo (self-adopted install)
cargo run --bin straymark -- validate .
cargo run --bin straymark -- charter drift CHARTER-01
```

**End-to-end against a scratch registry** (the flow the adopter described, run with the built binary):

```bash
# 1. Ex-ante: declare a deferral at Charter-declaration time, get a real id back.
straymark followups new --title "Redis CI job deferred" \
  --bucket charter-triggered --origin "CHARTER-06 §Scope" --cost S

# 2. Mutation: annotate a partial mitigation without changing status.
straymark followups note FU-002 "Part-a shipped (message-size cap in the codec); part-b deferred." \
  --source CHARTER-04

# 3. Mutation: flip a status and see the counters move in the same step.
straymark followups status                     # counters before
straymark followups set-status FU-002 closed
straymark followups status                     # counters after — no `recount` needed

# 4. The registry is still machine-readable after all three writes.
straymark followups list
straymark followups drift                      # no spurious re-extraction
```

### Production smoke (after deploy)

Not applicable — this Charter ships a CLI, not a deployed service. All verification is local.

## Risks

- **R1 — A write verb corrupts a governance artifact.** Medium severity, low probability: the registry is the canonical answer to "what's pending?", and a half-applied edit is worse than no verb at all.
  Mitigation: every write reuses the existing surgical span-based helpers rather than re-serializing the file; parse-then-validate happens **before** any write, and a registry with parse warnings aborts the mutation with a message pointing at the malformed entry. If the mitigation itself fails (a write lands malformed anyway), `list`/`status` surface the parse warning on the next invocation — the failure is loud, not silent. Lessons go to this Charter's AILOG `§Risk`.

- **R2 — Counter desync moves from "forgot `recount`" to "the verb miscounted".** Low severity, medium probability: the whole point of `set-status` is atomicity, so a bug there re-creates the exact problem in a place the operator now trusts.
  Mitigation: the new verbs call the same `compute_counters` + `fm_apply_counters_and_v1` path `recount` uses — one implementation, not two. Integration tests assert the counters after each verb equal what a subsequent `recount` would produce (`recount` stays idempotent as the escape hatch).

- **R3 — `new` mints an id that a concurrent `drift --apply` also mints.** Low severity, low probability: both read `max + 1` from the same file, so two processes racing on one registry could collide.
  Mitigation: read → assign → write happens within a single command invocation against a file that is a git-tracked, single-operator artifact; the atomicity claim is scoped to "one command, not two steps", which is the actual #360 hazard. True concurrent access is out of the registry's model and is not claimed. Documented as such rather than papered over with a lock file.

- **R4 — Ex-ante entries without a `Source-hash` confuse `drift` dedup.** Medium severity, low probability: `drift` keys dedup on `Source-hash(ailog_id, section, description)`, and an entry created by `new` has no AILOG to hash. If the same deferral is later mentioned in an AILOG's `§Follow-ups`, `drift --apply` could append a duplicate.
  Mitigation: verification step 4 runs `drift` after `new` to confirm no spurious extraction. A duplicate arising from a *later* AILOG mention is accepted behavior for this Charter — it is the same "the human said it twice" case the existing dedup cannot see either, and the operator resolves it with `set-status … superseded`. Explicitly named here so an auditor does not classify it as an undetected gap.

- **R5 — Scope creep back into the deferred options.** Low severity, medium probability: #360 lists three options, and the temptation during execution is to "just also add" reservation or Charter-scanning.
  Mitigation: both are declared out of scope above with reasons. If execution surfaces a case `new` genuinely cannot cover, it is documented as `R<N+1> (new, not in Charter)` in the AILOG and filed as a follow-up — not absorbed into this Charter.

## Tasks

1. Sync main, branch `feat/followups-mutation-and-creation-verbs`.
2. Extract the shared counter-recompute helper out of `recount.rs`; confirm `recount` behavior is unchanged (existing tests stay green untouched).
3. Implement `note` (+ `--source`, dated append).
4. Implement `set-status` (status validation + atomic counter recompute).
5. Implement `new` (id assignment, bucket insertion, field population, counter recompute).
6. Integration tests per verb, including the malformed-registry abort and the "counters equal a subsequent `recount`" assertion.
7. Schema/template/pattern-doc updates (EN + es + zh-CN); regenerate the agent-skill variants and confirm `gen_codex_skills --check` is clean.
8. Docs ×3 locales, `CLAUDE.md` command table, version bumps, `CHANGELOG.md`.
9. `.straymark/charters/README.md` created with this Charter listed.
10. AILOG (`risk_level: low`, `review_required: false`).
11. Local verification passes clean (including the end-to-end scratch-registry flow above).
12. `straymark charter drift CHARTER-01 --range <range>` before commit; document any drift in the AILOG or remediate atomically.
13. Commit + push + open PR referencing #355 and #360.

Single-batch execution — no `## Batch Ledger` needed.

## Charter Closure

When closing this Charter:

1. **Atomic update (format v4)**: if the drift check reported drift not already captured in the AILOG, edit `## Files to modify` and/or add `## Closing notes` **in the same PR**.
2. **Post-merge drift check**: `straymark charter drift CHARTER-01 --range origin/main..HEAD`.
3. **Move the row** in `.straymark/charters/README.md` to `## Closed` and reference the PR.
4. **Status frontmatter** moves from `in-progress` to `closed` (+ `closed_at:`).
5. **Do not delete** this file.
6. Close #355 and #360 with a comment stating what shipped and what was deliberately not built (the two deferred options above), so the adopter sees the reasoning rather than silence.

## Closing notes

`straymark charter drift CHARTER-01` reports **1 omission and 14 scope expansions** against `## Files to modify` (29 declared, 43 modified). All four causes, remediated atomically in this same PR:

- **`dist/.gemini/skills/…` and `dist/.agent/workflows/…` were declared as "Regenerated (same generator)" — they are not.** `gen_codex_skills` emits only the `.codex` surface; the Gemini and `.agent` mirrors have been hand-maintained since fw-4.22.0 (#221). Hand-synced with the same content edits instead. The declaration was authored from the CI check's name (`gen_codex_skills --check`) rather than from what the generator actually writes — the reconnaissance discipline (#210) was applied to the *files* but not to the *mechanism* that maintains them.
- **The governance version footers were not declared.** A framework bump requires updating the `fw-X.Y.Z` footer in `QUICK-REFERENCE`, `AGENT-RULES`, `DOCUMENTATION-POLICY`, `C4-DIAGRAM-GUIDE` and `FOLLOW-UPS-BACKLOG-PATTERN` (×3 locales each) per the release protocol in `CLAUDE.md`; `## Files to modify` listed only `dist-manifest.yml`. Applied.
- **`dist/.straymark/schemas/follow-ups-backlog.schema.v1.json` was declared and NOT modified** — correctly. The Charter declared it conditionally ("confirm the ex-ante shape validates; adjust `$defs.entry` if it does not"), and it validates unchanged: `origin_class` already contains `ex-ante-planning`, and `source_hash` is not a schema property at all. Kept in the declaration rather than deleted: the check was real work, and its outcome sharpened what #360 was — the vocabulary for the ex-ante case already existed, only the creation path was missing.

- **`cli/src/commands/followups/verify.rs` and `Cargo.lock` were not declared.** `verify.rs` was refactored onto the shared `write_recounted` path — the same consolidation the Charter declared for `recount.rs`, but only that one file was named; the declaration under-counted its own de-duplication. `Cargo.lock` moves with any version bump and is mechanical.

One new risk emerged, documented in the AILOG as `R6 (new, not in Charter)`: two pre-existing markdown-shape defects in the shared write helpers (`set_entry_field` collapsing an entry's trailing blank line; `insert_into_bucket` omitting the one before the next section), which every `drift`/`promote`/`verify` write had been degrading the registry with. Fixed at the root with regression tests, since `note` would otherwise have made the reshaping visible on every call.
