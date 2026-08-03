# PR & Commit Documentation Standard

> The git/PR history is a **consultable research record**. A reader months from
> now must reconstruct full context without asking anyone.

---

## Title / Subject line

- Format: `type(scope): brief description (#issue)` — single line, max ~80 chars.
- `type`: `fix` | `feat` | `chore` | `docs` | `refactor` | `test` | `perf`
- `scope`: affected area (`cli`, `followups`, `charter`, `audit`, `blog`, `website`…). Optional if obvious.
- Must be **specific and readable in isolation**: someone scrolling `git log --oneline` must understand what changed without opening it.
  - Bad: `fix bug`
  - Good: `fix(cli): charter drift recognizes build/manifest files in declared-set (#354)`
- If the PR closes multiple issues, reference the main one in the title; the rest go in the body.

---

## PR Body — required depth

1. **Causal opening**: Link closed issues (`Closes #NNN`). Name the origin (adopter report, Charter, AIDEC, field observation). State the concrete impact — real numbers, real output, real examples. Not "it was broken" but "1,092 of 1,581 embedded diff lines were prior-round audit prose."

2. **Fix with rationale**: The chosen approach AND why it was chosen over alternatives. Name the design principle (e.g., "prevention at --prepare beats detection at --review", "hash-neutral by construction", "ecosystem-agnostic — no per-extension treadmill").

3. **Surprises and discoveries**: Pre-existing defects found during implementation, schema invariants, problem reframings. If something unexpected emerged, document it explicitly.

4. **Deferred scope**: Name what was NOT done and why. "Tracked, not dropped." Prevents future sessions from re-investigating whether something was forgotten or deliberately excluded.

5. **Verification with concrete scenarios**: Test counts, what they assert, and an end-to-end scenario reproducing the original problem with real data showing it now resolves correctly. Include clippy/validate status.

6. **Release bookkeeping**: Version bumps (from → to), docs updated (which locales), CHANGELOG, governed AILOG path.

---

## Commit message — required depth

- First line: conventional commit format (`type(scope): description (#issue)`).
- Body: **15–30 lines** of dense narrative covering problem → fix → rationale → verification → version. The commit message is a *condensed* version of the PR body, not a one-liner. It must stand alone as a readable record.
- Include real examples when they illuminate the defect (truncated titles, false-positive output, etc.).
- End with version info and AILOG reference.

---

## Variants by change type

### Fixes (`fix`)

The record must answer: "what was broken, how did it manifest, and why this fix?"

- Causal chain: what was wrong → concrete impact → root cause → fix → why this approach.
- Real adopter data/examples whenever available.
- Name the reporting source (adopter, issue, field observation).

### Features (`feat`)

The record must answer: "what can the system do now that it couldn't before?"

- **The gap**: What capability was missing, what workflow forced the workaround, who hit it.
- **Design layers**: What was added at each layer (CLI verbs, schema fields, docs, templates, agent skills) and the operator-chosen depth. If deliberately scoped to fewer layers, say which and why.
- **Behavioral contract**: What the new commands/fields *do* and what they *refuse to do*. Refusal boundaries matter as much as the happy path (e.g., "remind and record — they never gate").
- **Composability**: How the new capability interacts with existing flows. Name invariants preserved.
- Same verification / release / deferred-scope sections as fixes.

### Housekeeping (`chore`, `docs`, `refactor`)

The record must answer: "what moved, and what did NOT change?"

- **Motivation**: Why now? Name the trigger or the debt being paid (e.g., "deferred from #341 to avoid widening a sensitive diff").
- **What moved / changed mechanically**: Concrete inventory — files relocated, args restructured, naming normalized. Enough for a reader to find things in their new location.
- **Behavioral invariant**: Explicitly state there is no user-facing behavior change (or name the minor one). E.g., "Pure refactor — audit.rs is now clippy-clean."
- **Lighter verification**: "Full suite green, no regressions" suffices. No end-to-end scenario needed unless the refactor touched a hot path.
- **Provenance**: If docs/decisions were migrated or restructured, note the old → new path mapping so cross-references in older issues/PRs remain traceable.

---

## Exclusions

- Do **not** include AI-tool attribution footers ("Generated with X", "Co-Authored-By: [AI agent]").
- Do **not** include emoji decorations.

---

## Tone

Technical narrative, impersonal. Writes like engineering documentation, not marketing. Assumes the reader is a future engineer (human or agent) investigating project state.
