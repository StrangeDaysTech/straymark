---
name: straymark-followups
description: Maintain the follow-ups backlog registry — the canonical answer to "what's pending?". Session-start glance, pre-commit drift --apply, post-Charter-close triage and operator-gated promote. Thin wrapper over the straymark followups CLI; never edits CLI-owned counters by hand.
---

# StrayMark Follow-ups Registry Skill

Maintain the central follow-ups registry (`.straymark/follow-ups-backlog.md`) — the first-class artifact that aggregates `§Follow-ups` and `R<N> (new, not in Charter)` entries across AILOGs *(first-class since fw-4.21.0 / cli-3.19.0)*. The agent is the registry's **primary maintainer**; this skill drives the three directives of `AGENT-RULES.md §13` by delegating every mutation to the CLI (`straymark followups list/status/drift/promote`). The skill contains no extraction or counting logic of its own — parsing, schema validation, counter recomputation, and FU → TDE elevation all live in the CLI.

> See `.straymark/00-governance/FOLLOW-UPS-BACKLOG-PATTERN.md` and `STRAYMARK.md §16` for the pattern; `AGENT-RULES.md §13` for the shipped directives this skill wraps.

## When to use this skill

Trigger on any of:

- Session start, or the operator asks *"what's pending?"* / *"what follow-ups do we have?"*.
- You created or modified an AILOG containing `## Follow-ups` or `R<N> (new, not in Charter)` entries and are about to commit.
- A Charter just closed and the registry entries it resolved need triage.
- The operator asks to promote a follow-up to a TDE document.

If the project does not maintain the registry (the per-AILOG `§Follow-ups` convention alone is enough below ~20 AILOGs — see the pattern doc), this skill does not apply.

## Instructions

### 1. Session start — answer from the registry

The registry is the **canonical source** for "what's pending". Answer from it; do not re-scan AILOGs.

```bash
straymark followups status                      # registry pulse: counters recomputed on the fly,
                                                # per-bucket breakdown, blocking / suspected-closed alerts
straymark followups list                        # full entry table
straymark followups list --severity blocking    # focus on blockers
straymark followups status FU-NNN               # one entry's full field detail
```

Fall back to an AILOG scan **only** when the registry does not exist, or when `straymark followups drift` reports unextracted AILOGs.

### 2. Pre-commit — registry rides the same commit as the AILOG

Created or modified any AILOG with `## Follow-ups` or `R<N> (new, not in Charter)` entries? Sync before committing:

```bash
straymark followups drift            # detect unextracted AILOGs (exit 1 on drift)
straymark followups drift --apply    # extract into `## Bucket: ready`, auto-number FU-NNN ids,
                                     # recompute counters, upgrade v0 registries to v1 in place
```

so the registry extension rides **the same commit** as the AILOG. Bullets whose AILOG text already carries a closure marker (`closed in-Charter`, `fixed in batch N`, a backtick-wrapped commit hash) are extracted as `suspected-closed` automatically — **do not delete them**; the operator confirms at the next triage.

### 3. Post-Charter close — triage what the Charter resolved

Review the registry entries the just-closed Charter resolved:

```bash
straymark followups list --status suspected-closed   # entries awaiting confirm-or-reopen
straymark followups recount                          # reconcile the CLI-owned counters after manual status flips (cli-3.20.0+)
straymark followups promote FU-NNN                   # FU → TDE elevation (operator-approved)
```

- Mark resolved entries `closed` (with the closing Charter id in `Notes`) or `superseded`.
- Confirm or reopen any `suspected-closed` entries that the Charter's AILOGs produced.
- After flipping statuses by hand, run `straymark followups recount` so the CLI-owned counters ride the same commit as the triage.
- For un-resolved entries that meet the TDE criteria of `AGENT-RULES.md §3` (heritage, transversal, dedicated Charter, human prioritization), **propose** promotion via `straymark followups promote FU-NNN` — promotion itself is operator-approved, per the autonomy limits of §3.

### 4. Report result

Surface the CLI output verbatim (counters, alerts, created TDE paths). Example after a pre-commit sync:

```
✓ Extracted 4 entries from 1 AILOG(s) into `## Bucket: ready`.
  ! 1 extracted as suspected-closed (closure marker in source AILOG) — confirm at the next triage.
  Counters recomputed: 3 open / 1 suspected-closed / 0 promoted (total 4).

StrayMark: registry synced — commit it together with the AILOG.
```

## What this skill does NOT do

- **It does not edit the frontmatter counters** (`total_open`, `total_promoted`, `total_suspected_closed`, …). They are CLI-owned: `straymark followups recount` (or any write command) recomputes them. Hand-editing them is a §13 violation.
- **It does not promote without the operator.** `straymark followups promote` is proposed, never auto-run — prioritization and assignment stay human (`AGENT-RULES.md §3`).
- **It does not delete `suspected-closed` entries.** The operator confirms (→ `closed`) or reopens them at the next triage.
- **It does not re-scan AILOGs to answer "what's pending?"** when the registry exists — the registry is canonical; `drift` tells you when it is not trustworthy.
- **It does not replace the per-AILOG `§Follow-ups` section.** That stays the write-time capture point; the registry aggregates it via `drift --apply`.

> **Terminal compatibility**: If the terminal does not support box-drawing characters (Unicode), use plain-text formatting with dashes and pipes instead (e.g., `+--+` instead of `╔══╗`).
