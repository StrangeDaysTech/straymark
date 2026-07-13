# Audit Rounds Pattern - StrayMark

> Per-round namespacing for Charters that need more than one external-audit round.

**Languages**: English | [Español](i18n/es/AUDIT-ROUNDS-PATTERN.md) | [简体中文](i18n/zh-CN/AUDIT-ROUNDS-PATTERN.md)

---

## Status

**Since fw-4.35.0 / cli-3.33.0** (Issue #341). Optional and back-compatible: single-round Charters need nothing new.

## The problem

The external-audit subsystem (`straymark charter audit`, `/straymark-audit-execute`, `/straymark-audit-review`) originally assumed **exactly one audit round per Charter**. But multi-phase Charters are a first-class concept, and phase-scoped auditing is the recommended practice for them. When one Charter needs **more than one** external-audit round (e.g. one per code-heavy phase), the flat, single-round layout breaks in two ways:

1. **Fixed flat paths overwrite.** Every round wrote `audit-prompt.md`, `report-*.md`, and `review.md` to the same `.straymark/audits/<CHARTER-ID>/` directory, so a second round silently clobbered the first's prompt, and preserving history needed a manual `git mv` archival dance.
2. **Cross-round glob pollution.** The `report-*.md` glob (both `--merge-reports` and the review skill) is flat and non-recursive. If a prior round's reports sat flat with any name still matching `report-*.md`, they were pulled into the **current** round's consolidated `review.md` and telemetry — mixing rounds.

## The pattern: `--round <label>`

Pass an optional round label to namespace the whole triad under a per-round subfolder:

```bash
# Round 1 — security phase
straymark charter audit CHARTER-01 --prepare --round fase-1 --range <phase-1-first-commit>..HEAD
# → .straymark/audits/CHARTER-01/fase-1/audit-prompt.md

# ...auditors write reports into the same subfolder, then:
straymark charter audit CHARTER-01 --merge-reports --round fase-1 \
  --merge-into .straymark/charters/CHARTER-01.telemetry.yaml
```

The label must be a simple slug (`[A-Za-z0-9._-]`, starts alphanumeric, no path separators or spaces) — it becomes a directory name.

### Resulting layout

```
.straymark/audits/CHARTER-01/
  fase-1/  { audit-prompt.md, report-*.md, review.md, external-audit-pending.yaml }
  fase-2/  { audit-prompt.md, report-*.md, review.md, external-audit-pending.yaml }
  fase-3/  { audit-prompt.md, report-*.md, review.md, external-audit-pending.yaml }
```

Because each round lives in its own subfolder and the glob is non-recursive, rounds never overwrite each other (fixes problem 1) and the merge scopes to exactly the current round's reports (fixes problem 2).

### Threading the label

The same `--round <label>` flows through the whole triad — the CLI's `--prepare` guidance and the skills echo it:

- `/straymark-audit-prompt <CHARTER-ID>` → `charter audit --prepare --round <label>`
- `/straymark-audit-execute <CHARTER-ID> --round <label>` → reads/writes under the subfolder
- `/straymark-audit-review <CHARTER-ID> --round <label>` → consolidates only that subfolder

## Telemetry: multiple rounds coexist

Each `external_audit` entry merged with `--round` carries a `round:` field, so rounds stay distinguishable inside one telemetry file:

```yaml
charter_telemetry:
  external_audit:
    - auditor: "gpt-5-2-codex"
      round: "fase-1"
      findings_total: 5
      # ...
    - auditor: "claude-sonnet-5"
      round: "fase-2"
      findings_total: 2
      # ...
```

`--merge-into` **appends** a new round to an already-populated `external_audit:` block instead of bailing — provided the round label is new. Re-merging the **same** round is still rejected (the same-round guard prevents silent duplication); merging into a populated block **without** a round label is also rejected (rounds must stay distinguishable). Use a fresh `--round <label>` per round.

## Back-compat

Omit `--round` entirely and everything behaves exactly as before fw-4.35.0: flat paths under `.straymark/audits/<CHARTER-ID>/`, a single `external_audit` block with no `round:` field, and `--merge-into` rejecting any populated array. Single-round Charters (the common case) need no change.

## Related

- [AGENT-RULES.md §12](AGENT-RULES.md) — the Audit Checkpoint that frames when to run an external audit, plus the stable-state and multi-round bullets.
- [FOLLOW-UPS-BACKLOG-PATTERN.md](FOLLOW-UPS-BACKLOG-PATTERN.md) — sibling pattern doc (registry layout convention this one mirrors).

---

*StrayMark fw-4.35.0 | [Strange Days Tech](https://strangedays.tech)*
