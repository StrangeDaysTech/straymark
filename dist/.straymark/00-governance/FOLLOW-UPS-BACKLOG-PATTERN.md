# Follow-ups Backlog Pattern - StrayMark

> First-class registry for managing accumulated `§Follow-ups` and `R<N> (new, not in Charter)` entries across many AILOGs and Charters.

**Languages**: English | [Español](i18n/es/FOLLOW-UPS-BACKLOG-PATTERN.md) | [简体中文](i18n/zh-CN/FOLLOW-UPS-BACKLOG-PATTERN.md)

---

## Status

**v1 — first-class entity since fw-4.21.0 / cli-3.19.0** (experimental; hard stabilization gated on a second adopter, per design principle #12 and ADR-2026-06-03-001).

Maturation chronology, mirroring the Charter lane:

| Stage | Release | What landed |
|-------|---------|-------------|
| Convention (v0) | fw-4.10.0 | Pattern doc + adopter-side bash script (Sentinel CHARTER-12, N=47) |
| Refinement (v0.1) | fw-4.13.1 | FU → TDE promotion path (2 shapes), `total_promoted` counter |
| **First-class (v1)** | **fw-4.21.0 / cli-3.19.0** | JSON schema, native `straymark followups` CLI, `explore`/`status` integration, agent directives shipped in `AGENT-RULES.md §13`, registry template |

The registry is a **first-class artifact** like Charter — not one of the 16 document types. It has its own canonical path, its own schema, its own CLI namespace, and its own synthetic group in the `explore` TUI.

---

## When this pattern applies

StrayMark's per-AILOG `§Follow-ups` convention works at write time — when an AILOG is created, the implementer documents what is deferred to subsequent Charters or operational triggers. That works fine until the cumulative list grows past what an operator can scan from memory.

Adopt this pattern when **any** of these conditions hold:

- The project has accumulated **~20 or more AILOGs** with non-trivial `§Follow-ups` sections.
- Operators repeatedly ask agents to "list what's pending across the project" and the answer requires a multi-file scan.
- A "do this when X arrives" follow-up was almost lost because the originating AILOG was never re-read after X arrived.
- A Charter retrospective surfaces follow-ups that should have been classified as `closed` weeks earlier but were never indexed.

Below that volume, the per-AILOG convention alone is sufficient — adopting this pattern early adds maintenance overhead without payoff.

### The registry as planning input

Empirical lesson from the reference adopter (issue #214, N=91 entries): the backlog is more than a list of deferred chores. Follow-ups originate not only from planning (ex-ante) but from **execution reality** — test runs, telemetry readings, staging incidents, bugs observed in real (non-simulated) environments — and they feed planning back: they become chores, mini-charters, or even reshape Charters that were already planned. The registry is the **ex-post counterpart of SpecKit**: SpecKit feeds planning from intent; the backlog feeds it from execution. The v1 dimensions (`Origin-class`, `Severity`, `Labels`, the `Destination` vocabulary) exist to make that planning loop queryable.

---

## Shape

### Registry file

Single markdown file at the canonical path:

```
.straymark/follow-ups-backlog.md
```

A template with empty frontmatter and the five bucket headers ships at `.straymark/templates/follow-ups-backlog.md`.

### Frontmatter (YAML)

```yaml
---
last_scan: 2026-06-03
last_scan_range: AILOG-NNNN-NN-NN-NNN..AILOG-NNNN-NN-NN-NNN  # optional — first..last AILOG covered
schema_version: v1
total_open: 0                # CLI-owned — recomputed on every write
total_promoted: 0            # CLI-owned
total_closed_in_session: 0   # CLI-owned
total_phase_blocked: 0       # CLI-owned
total_suspected_closed: 0    # CLI-owned (new in v1)
buckets:
  - ready
  - time-triggered
  - charter-triggered
  - phase-blocked
  - operational
fully_extracted_ailogs:
  - AILOG-2026-04-11-001
  - AILOG-2026-04-12-001
  # ... one entry per AILOG whose follow-ups have been processed
---
```

**The `total_*` counters are CLI-owned since v1.** Every write command (`straymark followups drift --apply`, `straymark followups promote`) recomputes them from the actual entry statuses. Do not maintain them by hand — stale hand-edited values are corrected on the next write. This closes the silent counter-drift failure mode observed at N=91 (declared `total_open: 47` vs 65 real after 4 weeks — issue #214 Signal 2). `straymark followups status` always shows counts recomputed on the fly, so the pulse is trustworthy even if the file is stale.

The `fully_extracted_ailogs` list records every AILOG whose `§Follow-ups` and `R<N>` entries have been transferred into the registry (or explicitly classified as superseded). Since cli-3.21.0 it is **informational** (shown by `followups status`); drift dedups per follow-up by content hash, not by this list — see "Per-follow-up content-hash dedup" below.

The formal frontmatter schema is `.straymark/schemas/follow-ups-backlog.schema.v1.json` (experimental v1 — see Status above).

### Buckets

Five buckets organize entries by trigger type — *when actionable*:

- `ready` — actionable now, no dependency on external trigger.
- `time-triggered` — calendar-based trigger (audit cycle, periodic review).
- `charter-triggered` — gated on a future Charter that touches the relevant area.
- `phase-blocked` — gated on a future component or phase that does not yet exist.
- `operational` — manual operator decision or external system action.

The vocabulary is stable at N=91 entries in the reference adopter — no sixth bucket has been needed. Severity (*how much it hurts to skip*) is intentionally **not** a bucket: it is an orthogonal per-entry field (see below).

### Entry schema (v1)

Each entry inside a bucket follows this shape (v1 fields marked; all of them optional — v0 entries remain valid):

```markdown
### FU-NNN — <short description>
- **Origin**: AILOG-NNNN-NN-NN-NNN <pointer to source section>
- **Source-hash**: <12 hex chars>                                                     (cli-3.21.0+, auto-managed — drift's dedup key; do not hand-edit)
- **Origin-class**: ex-ante-planning | testing | telemetry | staging | real-env-bug   (v1, optional)
- **Status**: open | in-progress | suspected-closed | closed | superseded | promoted
- **Severity**: normal | blocking                                                     (v1, optional; default normal)
- **Work verb**: design | implement | audit | operate                                 (optional; declared work-classification, Baton #332)
- **Design provenance**: new | upstream                                               (optional; only for implement — upstream degrades to operator)
- **Trigger**: ready | <calendar date> | when <X> | <other>
- **Destination**: chore | mini-charter | charter-replanning | operations | <charter-id> | <TDE id>
- **Cost**: <effort estimate>
- **Labels**: <free tags, comma-separated>                                            (v1, optional)
- **Notes**: <free-form context>
- **Promoted to**: <TDE id, when Status: promoted — see "Promotion to TDE" below>
```

`FU-NNN` is monotonically increasing across the registry's lifetime; do not renumber when entries close.

**The v1 dimensions**, each canonicalizing an empirically observed need (issue #214):

- **`Origin-class`** — where the entry was born: planning artifacts (ex-ante) vs execution reality (testing, telemetry, staging, real-environment bugs). Makes the ex-post planning loop queryable.
- **`Severity`** — `blocking` marks reliability-class issues that must land before a production cutover. Canonicalizes the `PROD-BLOCKER` prose convention that emerged in the reference adopter's `Notes` field (Signal 3). Orthogonal to the bucket: a `charter-triggered` entry can be `blocking`.
- **`Labels`** — free tags for grouping entries into planned Charters / mini-charters / chores during triage. Queryable via `straymark followups list --label <tag>`.
- **`Destination` vocabulary** — formalizes where the work lands when triggered: `chore`, `mini-charter`, `charter-replanning` (the entry reshapes an already-planned Charter rather than adding a task to it), `operations`, a specific Charter id, or a TDE id. Free-form values remain accepted (lenient parsing).

### Status vocabulary

- `open` — pending, not yet acted on.
- `in-progress` — a Charter has been declared or is executing that addresses this entry.
- `suspected-closed` *(new in v1)* — auto-extracted by `drift --apply` from an AILOG whose text carries an explicit closure marker (`closed in-Charter`, `fixed in batch N`, a commit hash, or a born-resolved idiom like `updated atomically in this PR` — see "Canonical closure-marker idioms" below). The operator confirms (→ `closed`) or reopens (→ `open`) at the next triage. See "Drift detection" below.
- `closed` — entry resolved (Charter merged, operational task done, time elapsed and reviewed).
- `superseded` — addressed by other work that did not reference this entry directly.
- `promoted` — the entry was elevated to a TDE document because it met the transversal-debt criteria (see "Promotion to TDE" below). The `Promoted to:` field carries the TDE id.

Closed, superseded, and promoted entries stay in the file (auditable history). Operators may move them to a `## Bucket: closed` section at the bottom for visual decluttering, but they are never deleted.

---

## Promotion to TDE

Some FU entries are not just deferred tasks — they describe **transversal technical debt** that warrants its own governance document (TDE). The criteria for promotion mirror the TDE-vs-`R<N>` disambiguation in `AGENT-RULES.md §3`:

- The entry is *heritage from a prior Charter* (already lived through ≥1 Charter close without remediation).
- The entry *applies to multiple modules or multiple Charters* — the central registry has fragmented it into bullets that share a root cause.
- The entry *requires a dedicated Charter outside the current scope envelope* to remediate.
- The entry *requires human prioritization or assignment* that the periodic operator review cannot decide from the bullet alone (impact × effort matrix, ownership).

When any of these holds, promote the FU entry to a TDE document under `.straymark/06-evolution/technical-debt/`:

```bash
straymark followups promote FU-NNN
```

The command automates the three-step flow that was manual in v0:

1. Creates the TDE document (same machinery as `straymark new --type tde`), pre-filling `impact`, `effort`, `type`, and body context from the FU entry.
2. Adds `promoted_from_followup: FU-NNN` to the TDE frontmatter for traceability.
3. In the FU entry, sets `Status: promoted`, `Destination: TDE-YYYY-MM-DD-NNN`, and `Promoted to: TDE-YYYY-MM-DD-NNN`; recomputes the frontmatter counters.

The FU entry is **not deleted** after promotion — its presence in the registry is the audit trail showing where the TDE came from.

### Two promotion shapes — promotion-of-existing vs retroactive-at-creation

The flow above covers the **standard case**: an `open` FU entry already exists in the registry and gets elevated to a TDE during periodic review. There is a second case that is equally valid and that emerged empirically from the Sentinel CHARTER-13 retrospective:

- **Promotion of existing entry** — an FU was registered (typically via `drift --apply`) as `open` weeks or Charters ago, lived through ≥1 Charter close without resolution, and meets the four criteria above. Standard flow.
- **Retroactive promotion at creation** — the debt is recognized as TDE-worthy *during* a retrospective (Charter close ceremony, audit cycle, RFC writeup) and never existed as an `open` FU. The TDE is created first; an FU entry is added to the registry *with `Status: promoted`* from birth, providing the audit trail back from the TDE to the originating context (an `R<N>` in an AILOG, a calibrator finding, a deferred classification).

Both shapes produce the same end state in the registry: an entry with `Status: promoted` and a `Promoted to: TDE-YYYY-MM-DD-NNN` pointer. The difference is whether the entry pre-existed as `open` or was born `promoted`. Drift detection treats them identically, and analytics counting `total_promoted` get the same number either way.

When in doubt, prefer creating the FU entry — even retroactively — because it cross-references the TDE back to the AILOG / R-number / source context that triggered the recognition. A TDE with `promoted_from_followup: FU-NNN` pointing to an entry that exists in the backlog is more navigable than one pointing to a fictional FU.

### When to promote

- **Periodic review** — when the operator does the manual reclassification pass, promote any entry that has lived through ≥2 Charter closes without resolution and meets the criteria above.
- **Charter close** — when reviewing entries the just-closed Charter resolved, if you find entries that were *not* resolved and meet the criteria above, promote them rather than leaving them as `open`.
- **Pre-Charter declaration** — if you're about to declare a Charter and notice the registry contains entries that this Charter would *partially* address, the un-addressed portion may belong as a TDE rather than as another deferred FU.

---

## Drift detection — native since cli-3.19.0

Drift detection keeps the registry in sync with new AILOGs. Since cli-3.19.0 it is a **native CLI command** — no external script:

```bash
straymark followups drift              # scan AILOGs changed in git diff origin/main..HEAD (fallback HEAD~1..HEAD) UNIONED with the working tree (git status --porcelain); exit 1 on drift
straymark followups drift --apply      # same scan + extract new entries into the registry
straymark followups drift --scan-all   # periodic full sweep over every AILOG
```

Since cli-3.21.0 the default scan unions the committed git range with the working tree (`git status --porcelain`), so an uncommitted/untracked AILOG is visible to the documented pre-commit flow — you no longer need `--scan-all` to see a just-written AILOG before it is committed (issue #229).

### What `--apply` does

1. Extracts every `§Follow-ups` bullet and `R<N> (new, not in Charter)` risk **whose content hash is not yet in the registry**, appending them under `## Bucket: ready` with auto-numbered `FU-NNN` ids and a stored `Source-hash`. The operator reclassifies bucket/trigger/destination at the next triage. (Already-extracted AILOGs are re-scanned and deduped per follow-up — see "Per-follow-up content-hash dedup" below.)
2. **Anti-noise refinement** *(v1 — resolves issue #214 Signal 1)*: bullets whose AILOG text carries an explicit closure marker (`closed in-Charter`, `fixed in batch N`, a commit hash reference) are extracted with `Status: suspected-closed` instead of `open`, instead of polluting the `ready` bucket as TBD noise. Across both documented occurrences in the reference adopter, 20–75% of auto-appended entries per batch were already resolved in-Charter — this refinement removes the single recurring cost of the v0 workflow.
3. Appends the AILOG id to `fully_extracted_ailogs`.
4. **Recomputes all `total_*` counters** from actual entry statuses (Signal 2).
5. If the registry is `schema_version: v0`, upgrades it to `v1` in place — non-destructively and idempotently (all v1 fields are optional; nothing is rewritten except the version marker and the counters).

Since cli-3.20.0, `--apply` recomputes the counters **even when there is nothing to extract** — so a pre-commit `drift --apply` also reconciles counters left stale by a manual-triage session (first external adopter feedback, issue #222 Finding 1).

### Canonical closure-marker idioms

The anti-noise refinement recognizes a fixed vocabulary, case-insensitively. AILOG authors should converge on these phrasings at write time so born-resolved entries land as `suspected-closed` instead of TBD noise:

| Idiom family | Examples |
|---|---|
| In-Charter closure | `closed in-Charter`, `closed in Charter`, `resolved in-Charter`, `resolved in Charter` |
| Batch fix | `fixed in batch 3` (requires the number) |
| Commit reference | a backtick-wrapped commit hash: `` `ab12cd34ef` `` (7–40 hex chars, at least one digit) |
| Born-resolved *(cli-3.20.0+, #222 Finding 2)* | a closure verb — `updated` / `corrected` / `remediated` / `resolved` / `fixed` / `closed` — followed by `in this PR` or `in this commit`, e.g. `Charter row updated atomically in this PR` |

Phrasings outside this vocabulary (e.g. `done earlier`, `no longer relevant`) extract as `open`; the operator flips them at triage. When a new closure idiom recurs in your AILOGs, propose it upstream rather than hand-editing extractions.

### Per-follow-up content-hash dedup

Since cli-3.21.0, drift dedups **per follow-up by a stable content hash** (`fu_content_hash` of the source AILOG id + origin section + description), stored as each entry's `Source-hash`. Already-extracted AILOGs are re-scanned and individual follow-ups deduped against the registry — so a follow-up **appended to an already-extracted AILOG** (the multi-batch Charter case, where one AILOG's `§Follow-ups` grows across batches) is caught instead of silently missed (issue #231).

The original objection to per-bullet matching was paraphrase false positives: curated registry entries reword the AILOG bullet, so recomputing a hash from the *registry* text would re-flag already-extracted content. The stored `Source-hash` resolves this — it is captured at extraction time from the AILOG's original text and never recomputed from the (later paraphrased) registry heading. The zero-false-positive property is preserved for every hash-bearing entry.

Legacy entries created before cli-3.21.0 have no `Source-hash`; for them drift falls back to recomputing the hash from `Origin` + `description` — best-effort, and the one residual paraphrase-vulnerability vector, shrinking as legacy entries close out. `fully_extracted_ailogs` is retained (it records which AILOGs have been scanned and is shown by `followups status`) but is **no longer the skip gate** — dedup is by content hash, not whole-AILOG id.

### Legacy bash script (deprecated)

The v0 reference implementation (`scripts/check-followups-drift.sh`, ~296 lines of POSIX bash in the Sentinel adopter's repo) is **deprecated as of cli-3.19.0**. It keeps working for v0 registries but is no longer maintained and lacks the anti-noise refinement and counter recompute. Migration path: delete the script, run `straymark followups drift --scan-all --apply` once (this also upgrades the registry to v1), and update any pre-commit hook to call the CLI instead.

**Run that first post-migration sweep with `--scan-all` even if the script reported "in sync"**: the bash extractor was format-sensitive (it required both a `## Risk` heading and the exact `- **R<N> (new` bullet shape) and produced **silent false-negatives** on format variants — AILOGs writing risks as bare paragraphs never registered as having follow-up content at all. In the reference adopter's migration ([issue #225](https://github.com/StrangeDaysTech/straymark/issues/225)), the native lenient parser caught **8 AILOGs / 29 entries** that the script had reported as "in sync" the day before. Silent false-negatives on drift detection are the exact failure mode the tool exists to prevent — which is why the script is deprecated rather than maintained.

---

## CLI surface

```bash
straymark followups list                  # enumerate entries: FU id, status, severity, bucket, destination
straymark followups list --bucket ready --status open --severity blocking --label <tag>
straymark followups status                # registry pulse: counters (recomputed on the fly), per-bucket/severity breakdown
straymark followups status FU-NNN         # detail view of one entry
straymark followups drift [--apply|--scan-all]   # drift detection (see above)
straymark followups recount               # recompute the CLI-owned counters after a manual-triage session (cli-3.20.0+)
straymark followups promote FU-NNN        # automate FU → TDE promotion (see above)
```

The registry also appears as a synthetic **Follow-ups** group in the `straymark explore` TUI (sub-nodes per bucket) and as a counts block in `straymark status`.

---

## Agent integration

Since fw-4.21.0 the agent directives **ship with the framework** in [`AGENT-RULES.md §13`](AGENT-RULES.md) — adopters no longer copy a block into their own `CLAUDE.md` / `AGENT.md`. In summary:

- **Session start**: glance at `.straymark/follow-ups-backlog.md` (or run `straymark followups status`) to know what is pending across the project.
- **Pre-commit**: created or modified any AILOG with `## Follow-ups` or `R<N> (new, not in Charter)` entries? → run `straymark followups drift --apply` in the same commit.
- **Post-Charter close**: review entries the Charter resolved; mark them `closed` (with the closing Charter id in `Notes`) or `superseded`; confirm or reopen any `suspected-closed` entries; then run `straymark followups recount` so the CLI-owned counters ride the same commit as the triage; promote un-resolved entries that meet the TDE criteria via `straymark followups promote`.

This makes the agent the registry's primary maintainer, the CLI the verification layer, and the operator the periodic reviewer (re-bucketing, confirming suspected-closed, pruning superseded, promoting to TDE when criteria apply).

---

## Adoption walkthrough

For an adopter starting fresh:

1. Copy `.straymark/templates/follow-ups-backlog.md` to `.straymark/follow-ups-backlog.md` (empty `fully_extracted_ailogs:` list, five `## Bucket:` headers).
2. Run `straymark followups drift --scan-all --apply` to seed the registry from existing AILOGs.
3. Reclassify the auto-generated `## Bucket: ready` entries into the correct buckets manually; fill `Origin-class`/`Severity`/`Labels` where they add signal. This is one-time triage, typically 30-60 min for a backlog of ~50 entries.
4. Done — the agent directives in `AGENT-RULES.md §13` are already active; no `CLAUDE.md` edits needed.

For an adopter migrating from the v0 convention: run `straymark followups drift --apply` once (auto-upgrades the registry to v1), delete the local bash script, and update any pre-commit hook to call the CLI.

---

## Reference implementation

`StrangeDaysTech/sentinel` — the originating adopter:

- v0 pattern: CHARTER-12, merged 2026-05-06 ([sentinel#53](https://github.com/StrangeDaysTech/sentinel/pull/53), [sentinel#54](https://github.com/StrangeDaysTech/sentinel/pull/54)). 47 entries seeded from the CHARTER-08 → CHARTER-11 retrospective.
- Scale validation: post-Etapa-3 triage at **N=91 FUs / 76 AILOGs extracted / 65 open** ([issue #214](https://github.com/StrangeDaysTech/straymark/issues/214)) — the empirical input that drove the v1 schema and the native CLI (ADR-2026-06-03-001).

---

## Open questions

Resolved in v1:

- ~~**Schema validation.**~~ → `.straymark/schemas/follow-ups-backlog.schema.v1.json` (frontmatter), entry-shape validation in the CLI parser.
- ~~**Cristalization as `straymark followups` CLI.**~~ → native `list / status / drift / promote` since cli-3.19.0.
- ~~**Bucket classification heuristic** (partially).~~ → `suspected-closed` removes the dominant noise class; full bucket suggestion (using AILOG `tags` / Charter `effort_estimate`) remains open.

Still open for future revisions:

- **Integration with the audit cycle.** When `straymark charter audit --merge-reports` produces real-debt findings that are not remediated atomically pre-close, those findings live only in `.straymark/audits/<id>/review.md`. They do not auto-flow into the central registry. Surfacing them automatically would close a known gap.
- **`closed` vs `superseded` semantics.** Today the difference is whether the resolving work explicitly referenced the entry. A stricter convention may emerge.
- **Soft integration with `charter close`** (issue #135 Tier 3): auto-invoking `followups drift --apply` after a Charter close, with an interactive promotion prompt. Gated on a friction signal from a second adopter.
- **Hard schema stabilization (v1.0).** Gated on validation by a second adopter in another domain, per design principle #12.

---

## Credits

Contributed via [issue #111](https://github.com/StrangeDaysTech/straymark/issues/111) by the Sentinel adopter; matured to first-class via [issue #214](https://github.com/StrangeDaysTech/straymark/issues/214) and ADR-2026-06-03-001. Empirical foundation: CHARTER-08 → CHARTER-11 chain and the post-Etapa-3 triage at N=91 in `StrangeDaysTech/sentinel`. Author: José Villaseñor Montfort.

*This document was produced with assistance from generative AI tools (Claude 4.7 / Opus 4.8); all responsibility for the content rests with the human author.*

---

## Related

- [EMERGENT-OBSERVATION-DESIGN.md](EMERGENT-OBSERVATION-DESIGN.md) — the meta-pattern that this drift-detection convention instantiates at the per-AILOG ↔ registry surface.
- [CHARTER-CHAIN-EVOLUTION.md](CHARTER-CHAIN-EVOLUTION.md) — sibling pattern that operates at chain level (Pattern 1) and cycle level (Pattern 2).
- [AGENT-RULES.md §3](AGENT-RULES.md) — TDE-vs-`R<N>` escalation criteria that may promote follow-ups to dedicated debt entries; §13 — the shipped agent directives for registry maintenance.
- `STRAYMARK.md §16` — the onboarding-level summary of the registry as a first-class artifact.

---

*StrayMark fw-4.34.0 | [Strange Days Tech](https://strangedays.tech)*
