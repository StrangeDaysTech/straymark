---
id: ADR-2026-07-18-001
title: Follow-ups are dated hypotheses — verify the premise at execution, not at capture
status: draft
created: 2026-07-18
updated: 2026-07-18
agent: claude-opus-4-8-1m
confidence: high
review_required: true
# --- Approval workflow (fill at review time) ---
# reviewed_by: <reviewer-id>
# reviewed_at: YYYY-MM-DD
# review_outcome: approved
risk_level: low
eu_ai_act_risk: not_applicable
iso_42001_clause: []
alternatives_documented: [doc-only, doc-plus-schema, doc-plus-schema-plus-cli, verify-at-capture]
api_changes: [followups-entry-schema, followups-cli-surface]
tags: [follow-ups, backlog, epistemics, speculative-buffer, cli, governance, adopter-feedback]
related: [FOLLOW-UPS-BACKLOG-PATTERN, ADR-2026-06-03-001]
supersedes: []
---

# ADR: Follow-ups are dated hypotheses — verify the premise at execution, not at capture

## Status

**Draft — pending human review.** Recording the decision and its implementation shape across
three layers (pattern doc, entry schema, CLI affordance). No product files change until this ADR
is signed. Evolves — does not supersede — [ADR-2026-06-03-001](ADR-2026-06-03-followups-first-class.md)
(the follow-ups backlog as a first-class entity). The registry stays `schema_version: v1`
(experimental); the additions here are optional and backward-compatible, so hard stabilization
remains gated on design principle #12 as before.

## Context

Field report from the **Weft** adopter ([StrayMark #365](https://github.com/StrangeDaysTech/straymark/issues/365);
greenfield .NET 10 / Rust CRDT library, the same adopter behind #345/#346/#354/#355/#360). Draining
a follow-up backlog before an irreversible publish (7 open → 1) surfaced a consistent, load-bearing
observation about **what a registry entry _is_**.

Of the seven entries, three were genuine unresolved work. **All three rested on a premise that was
false the moment it was checked** — and none was false because the code had drifted under it. Each
was false _at the time it was written_, and the falseness cost only a ~30-second check to surface
months later:

1. **A test to "replicate" that never existed.** FU-017 read "replicate for the Loro shim the
   header↔binding parity test the yrs shim already has." The yrs test **did not exist** — two
   _code comments_ asserted "a CI test validates these match." The line was aspirational; the
   follow-up read a comment as fact and added a hop of authority.
2. **A gate against a reference that can't exist.** FU-016 asked for a Loro determinism gate
   mirroring the yrs↔Yjs parity gate. yrs has an _independent_ reference (Yjs); Loro has none —
   the npm package is a WASM build of the same Rust core, so the comparison is the crate against
   itself. Reasoning by symmetry; the analogy silently failed.
3. **An optimization for a cost that wasn't there.** FU-010 feared persist-before-broadcast would
   put I/O on the actor's hot path. Tracing the actual `await` chain showed the persistence call
   was _already_ awaited on the receive loop — reordering added no I/O to any hot path. The
   follow-up encoded a slightly-stale _mental model_, not the code as built.

Three distinct failure modes — a comment believed, an analogy over-trusted, a mental model gone
stale — one shared shape: **each premise was false at write time and cheap to falsify at read time.**

There are two moments a "verify the premise" rule could live:

- **Write (capture) time is the worst moment to verify.** A follow-up is a note jotted while
  _finishing something else_ — attention on the work being landed, glancing at a subsystem about to
  be left. Verifying means a full context switch away from that work.
- **Read (execution) time is the best moment to verify.** When you act on the entry you are already
  inside that subsystem with the code open; checking the premise is a `grep`, a file read, a traced
  call chain — seconds, and you have already paid the context-switch cost for other reasons.

The reframe: **a follow-up backlog is a _speculative buffer_.** Its value is cheap capture so a
signal is not lost. Demanding verification at capture would defeat that purpose — the rational
response would be to _stop writing follow-ups_. So an under-verified entry is **not an authoring
defect; it is the expected epistemic status of anything in a speculative buffer.** An entry is a
**dated, decaying hypothesis**, and the only real bug is _executing one without re-testing its
premise_.

The pattern's current framing works against this. [`FOLLOW-UPS-BACKLOG-PATTERN.md`](../../dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md)
presents the registry as **"planning input"** and a to-do list ("### The registry as planning
input"). Read as instructions, false premises become wasted Charters. Read as dated hypotheses to
re-check at execution, they become cheap bets. This is a framing + verification-placement gap, not
a correctness bug in any existing command.

## Decision

**Reframe follow-up entries as dated, decaying hypotheses, and place premise verification at
execution (promote / act), never at capture.** Implement the reframe across three layers — the
depth chosen by the operator on 2026-07-18:

### 1. Documentation (governance)

In [`FOLLOW-UPS-BACKLOG-PATTERN.md`](../../dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md)
(EN + `i18n/es` + `i18n/zh-CN`):

- Add a first-class section — **"Epistemic status: entries are dated hypotheses"** — stating: the
  registry is a speculative buffer; cheap capture is the point; an entry is a dated, decaying
  hypothesis, not an instruction/plan; an under-verified entry is the _expected_ status of a buffer
  item, not an authoring defect; **the only real bug is executing an entry without re-testing its
  premise.**
- Make the discipline explicit and directional: **write cheaply at capture; re-verify the premise
  when you promote/act.** Add it to the agent directives summarized from `AGENT-RULES.md §13`.
- Reconcile the existing "The registry as planning input" subsection so "planning input" and
  "dated hypothesis" read as one coherent stance (the buffer _feeds_ planning; each item is a bet
  to re-check when drawn, not a committed task).

### 2. Entry schema (optional, backward-compatible)

In [`follow-ups-backlog.schema.v1.json`](../../dist/.straymark/schemas/follow-ups-backlog.schema.v1.json)
`$defs.entry` (the documented-but-lenient entry vocabulary) and the pattern doc's "Entry schema
(v1)" block, add two **optional** fields:

- **`premise`** — the load-bearing assumption the entry rests on, stated so it can be re-checked
  in seconds at execution ("the yrs shim already has a header↔binding parity test"). Making the
  premise explicit is what turns "re-verify" from a vague nudge into a concrete, targetable check.
- **`verified-at`** — the date the premise was last re-verified against reality, stamped when the
  operator acts/promotes. Absent = never re-checked since capture (the default, honest state).

Both are optional; every existing entry stays valid; the CLI parser stays lenient. `v1` remains
experimental — no schema-breaking change, no bump of `schema_version`.

### 3. CLI affordance at execution time

Surface the premise **when the operator acts**, so the re-check happens at the cheap moment. Two
candidate shapes (final choice deferred to implementation, recommendation **A**):

- **(A, recommended) Extend `followups promote FU-NNN`.** Before promoting, print the entry's
  `Premise` (falling back to `Notes` when absent) and emit an explicit **"Is this still true?
  re-verify against the code before you build on it."** reminder. Keep human judgment out of the
  CLI (as `promote` already does): the command _surfaces_ the premise and _stamps_ `verified-at`
  when the operator passes a confirmation flag (e.g. `--premise-verified`); it never decides truth
  itself.
- **(B, alternative) New `followups verify FU-NNN` verb.** A dedicated read/act step that prints
  the premise, optionally records/updates `premise` (`--premise "..."`), and stamps `verified-at`.
  Heavier surface; keep in reserve if the promote-time reminder proves too narrow (many entries are
  acted on as chores that never promote).

Human prioritization stays out of the CLI, consistent with `promote`. The CLI's job is to put the
premise in front of the operator at the moment of spending, and to record that the check happened.

## Alternatives Considered

### 1. Doc-only reframe

Reframe the pattern doc + `AGENT-RULES.md §13`; no schema or CLI change. Lightest and fully
reversible. **Rejected as insufficient** for the chosen depth: without an explicit `premise` field
the re-verify discipline has nothing concrete to target, and without a CLI touchpoint the reminder
lands only if the operator happens to re-read the doc. Good as a fallback if review trims scope.

### 2. Doc + schema fields, no CLI

Add `premise` / `verified-at` and reframe the doc, but no execution-time affordance. **Rejected**:
the fields exist but nothing surfaces them at the one moment they matter (acting on the entry), so
the load-bearing behavior — re-check before you build — is left entirely to operator memory.

### 3. Doc + schema + CLI affordance (**chosen**)

The full package: reframed doc, optional `premise` / `verified-at` fields, and a promote-time (or
`verify`) affordance that surfaces the premise and stamps the re-check. Puts the reminder at the
cheap moment and gives the discipline a concrete target and an audit stamp.

### 4. Verify the premise at capture time

Require the follow-up author to verify before writing the entry. **Rejected on principle**: it
destroys the speculative buffer. The value of the backlog is cheap capture while finishing other
work; demanding a context switch to verify at capture would make the rational move "stop writing
follow-ups," losing the signal entirely. This is precisely the anti-pattern the ADR names.

## Consequences

### Positive

- **False-premise entries become cheap bets, not wasted Charters.** The three Weft cases show a
  ~30-second re-check at execution prevents implementing against an assumption that was never true.
- **Removes a category of self-inflicted work** without adding capture-time friction — the buffer
  stays cheap to write.
- **`premise` makes entries auditable as hypotheses**: a reader (or Loom node model) can see the
  assumption an entry rests on, and `verified-at` shows whether it has been re-checked since birth.
- **Directly complements the #365 title-fidelity fix** (cli-3.36.2): titles now read faithfully,
  and this ADR settles how the entry _behind_ the title should be read.

### Negative

- **More surface to maintain**: two new (optional) fields, doc text in three locales, and a CLI
  code path with tests. Mitigated by keeping fields optional and the CLI affordance thin.
- **Adoption is behavioral, not enforced**: nothing _forces_ an operator to re-verify. By design —
  the CLI reminds and records; it does not gate. The discipline lives in the doc + the prompt.

### Neutral

- `verified-at` is advisory provenance, not a validation signal; `followups status`/`validate` do
  not fail on its absence (consistent with the pattern's "warn, never fail" stance).
- Existing registries are untouched until an operator adds a `premise` or acts on an entry.

### Quality Impact Assessment

Low risk. All schema additions are optional and lenient-parsed; no existing entry, counter, or
`Source-hash` changes. The CLI affordance is additive (a new flag / verb) with no change to
`drift`, `list`, `status`, or the dedup path. The reframe is words + two optional fields + one
reminder — no data migration, no breaking change.

## Affected Components

| Component | Change |
|-----------|--------|
| `dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md` (+ `i18n/es`, `i18n/zh-CN`) | New "Epistemic status" section; reconcile "planning input"; document `premise` / `verified-at`; verify-at-execution discipline |
| `dist/.straymark/00-governance/AGENT-RULES.md §13` (+ i18n) | Add the "re-verify the premise at promote/act, not capture" directive |
| `dist/.straymark/schemas/follow-ups-backlog.schema.v1.json` | Add optional `premise`, `verified-at` to `$defs.entry` |
| `dist/.straymark/templates/follow-ups-backlog.md` | Optional example entry showing `Premise` / `Verified-at` |
| `cli/src/commands/followups/promote.rs` (or a new `verify.rs`) | Surface premise + stamp `verified-at`; `--premise-verified` flag |
| `cli/src/followups.rs` | Parse/`set` `Premise` / `Verified-at` fields (lenient) |
| `docs/adopters/CLI-REFERENCE.md` (+ i18n) | Document the new affordance |
| `CHANGELOG.md` | Framework + CLI entries |

## Implementation Plan

Ships as one framework + CLI pair once this ADR is signed:

1. **Doc reframe** (framework): the "Epistemic status" section + directive + schema-field docs in
   all three locales. Reviewable on its own.
2. **Schema fields** (framework): optional `premise` / `verified-at` in `$defs.entry` + template
   example.
3. **CLI affordance** (CLI): shape A (promote-time surfacing + `verified-at` stamp) with unit +
   integration tests; lenient parse/set for the two fields.
4. **Version bumps**: framework minor (`fw-4.36.0`) for the pattern-doc + schema change; CLI minor
   (`cli-3.37.0`) for the new affordance. Update the six version tables + `CHANGELOG.md`.
5. **AILOG** for the change set, per governance.

Sub-decision left open for the operator at sign-off: **CLI shape A vs B** (extend `promote` vs a
dedicated `followups verify`).

## Success Metrics

- A follow-up carrying an explicit `premise` can be re-verified at execution without re-reading the
  originating AILOG.
- `followups promote` surfaces the premise and records `verified-at` on confirmation.
- No regression in `drift` dedup, counters, or existing entry parsing (existing suite stays green).
- Adopter signal (Weft / the next N) reports the reframe reduced false-premise implementation.

## Validation Criteria

- Existing follow-ups tests pass unchanged; new tests cover `premise` / `verified-at` parse + set
  and the promote-time surfacing.
- Schema validates a registry with and without the new fields.
- Pattern doc renders in all three locales; `sync:docs` mirrors cleanly to the website.

## References

- [StrayMark #365](https://github.com/StrangeDaysTech/straymark/issues/365) — the field report.
- [ADR-2026-06-03-001](ADR-2026-06-03-followups-first-class.md) — follow-ups as a first-class entity (this ADR evolves it).
- [FOLLOW-UPS-BACKLOG-PATTERN.md](../../dist/.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md) — the pattern being reframed.
- #366 (cli-3.36.2) — the #365 title-fidelity fix (extractor half; this ADR is the framing half).
- Related open adopter issues: #360 (ex-ante creation path), #355 (`note`/`set-status` verbs), #346 (drift extraction fidelity).

## Revision History

| Date | Change |
|------|--------|
| 2026-07-18 | Initial draft (design pass opening for #365 Part 1). Pending human review. |

---

*This document was produced with assistance from generative AI tools (Claude Opus 4.8); all
responsibility for the content rests with the human author.*
