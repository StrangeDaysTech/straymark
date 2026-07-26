---
id: AILOG-2026-07-25-002
title: Migrate historical decision records (5 ADRs + 1 AIDEC) into the governed .straymark/ tree (#368)
status: accepted
created: 2026-07-25
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
tags: [self-adoption, governance, migration, decisions, docs, website]
related: [AIDEC-2026-07-13-001]
---

# AILOG: Migrate historical decision records into the governed tree (#368)

## Summary

Moved the repo's own decision records out of the pre-adoption location (`docs/decisions/`, invisible to the CLI) into the governed tree: 5 ADRs → `.straymark/02-design/decisions/`, the self-adoption AIDEC → `.straymark/07-ai-audit/decisions/`. All 7 moves used `git mv` (history preserved). `straymark validate .` goes from "these documents do not exist as far as governance is concerned" to **0 errors across 8 documents**, and the `REF-001` warning that #368 was filed to clear is gone. No release: nothing in `dist/` changed, so this is repo governance hygiene, not a product change.

## Context

Since self-adoption ([`AIDEC-2026-07-13-001`](../decisions/AIDEC-2026-07-13-001-straymark-self-adoption.md), installed in PR #362) the governed tree is `/.straymark/`. The repo's own decisions still lived in `docs/decisions/` — the historical, pre-adoption location — so `validate` and `status` could not see them, and the review/approval flow did not apply. The concrete signal was a benign-but-telling warning: the newly-governed `AIDEC-2026-07-18-001` pointed at `ADR-2026-06-03-001`, which the resolver could not find because the target sat outside `.straymark/`. Every future cross-reference from a governed doc into the historical tree would have warned the same way.

## Actions Performed

1. **Moved the records** — 5 ADRs to `.straymark/02-design/decisions/`, `AIDEC-2026-07-13-001-straymark-self-adoption.md` to `.straymark/07-ai-audit/decisions/`, all via `git mv`.
2. **Normalized three filenames** that carried no sequence number while their frontmatter id did (`ADR-2026-05-08-rebranding-straymark.md` with id `ADR-2026-05-08-001`, and the same shape for the loom-stack and follow-ups ADRs) → `ADR-YYYY-MM-DD-001-slug.md`. This cleared the `NAMING-002` warnings **and**, as a side effect, the `REF-001` ones: the reference resolver matches by filename prefix, so `related: [ADR-2026-06-02-001]` could never resolve against `ADR-2026-06-02-loom-stack.md`. The naming drift and the dangling references were the same defect seen twice.
3. **Resolved the id collision** — `AIDEC-2026-07-13-001-implementation-plan.md` shared its parent's id, had no frontmatter, and is a *living checklist* rather than a decision record (it would have failed validation as an AIDEC). Renamed and relocated to `docs/decisions/proposals/2026-07-13-self-adoption-implementation-plan.md` with a header stating what it is and is not.
4. **Swept inbound references** — 27 files rewritten to the new paths (`CHANGELOG.md`, `CLAUDE.md`, `experiment-loom/**` charters/specs/README, `website/blog/**` and its `es`/`zh-CN` translations, the governed `AIDEC-2026-07-18-001`). Recomputed the relative-link depth in the moved documents themselves.
5. **Updated six public-facing claims** — `docs/intro.md` and `docs/contributors/README.md` (×3 locales) told readers the ADRs live in `docs/decisions/`, which after the move pointed at a directory holding only `proposals/`. They now point at the governed tree and say why (StrayMark governs its own development with StrayMark).
6. **Corrected two stale statements in `CLAUDE.md`** surfaced by touching that paragraph: self-adoption is described as "preparing to adopt" and `/.straymark/` as "does not exist yet" — both untrue since PR #362.
7. **Amended the migrated AIDEC's `## Approval` note**, which asserted the file is *kept* in `docs/decisions/` — a statement the migration falsified. Reworded to the past tense with a dated migration note; the signature itself is untouched.

## Decisions Made

- **`proposals/` stays in `docs/`** (#368 recommendation (a)). The seven files there are exploratory working documents, not governance-in-force, and no StrayMark doctype fits them. This leaves a clean invariant: **`docs/decisions/` now holds only non-doctype working documents; every real decision record lives in `.straymark/`.**
- **The implementation-plan companion is not folded into the parent AIDEC.** The AIDEC is signed and immutable; the checklist keeps changing. Merging a mutable checklist into a signed decision would make the signature cover a moving target.
- **Historical prose is left as written.** The ADR's own `## Implementation Plan` still says "PR 1 — this ADR (`docs/decisions/`, no release)", and an `experiment-loom` AILOG's file-change table still lists the old paths. Those record what was true at the time; rewriting them would falsify the record. Only *links* (which serve a reader now) were updated.
- **No redirect stubs.** Verified no straymark.dev URL depends on the old paths: the website excludes `**/decisions/**` from the docs build, and every inbound link from blog posts is an absolute `github.com/.../blob/main/...` URL that this PR rewrites. Stubs would add clutter with no reader to serve.
- **No CHANGELOG entry and no version bump.** Nothing under `dist/` changed; adopters receive no behavior or content difference.

## Verification

- `straymark validate .` → **0 errors, 1 warning** across 8 documents. The one remaining warning is `REF-002` on the rebranding ADR (no traceability links) — honest and left alone: nothing else in the governed tree relates to it, and inventing a `related:` entry to silence a warning would be exactly the kind of decorative compliance the framework exists to avoid.
- `npm run build` in `website/` → exit 0, all three locales generated plus the RSS/Atom feeds (27 posts × 3). Log has no broken-link warnings (`onBrokenLinks: 'warn'` means the log, not the exit code, is the signal).
- Spot-checked the built HTML: `build/docs.html`, `build/docs/contributors.html`, `build/es/docs.html`, `build/zh-CN/docs.html` and `build/blog/the-rebrand-to-straymark.html` all carry the new `.straymark/02-design/decisions/...` targets.
- `git status` confirms all 7 moves registered as renames (`R`), so `git log --follow` still reaches the pre-migration history.
- Grepped the tree for every old path form; the only remaining occurrences are the two deliberate historical-prose cases above.

## Risk

Low. The migration is path-only — no decision content was altered beyond the AIDEC's location note and the three filename normalizations. The reversible failure mode is a missed inbound link; mitigated by the exhaustive grep sweep plus the website build. The one irreversible-feeling change (renaming three ADR files) is what makes their `related:` references resolve, and `git mv` keeps the history attached.

## Modified Files

| File | Change |
|------|--------|
| `.straymark/02-design/decisions/ADR-2026-05-08-001-rebranding-straymark.md` | Moved + renamed (was `docs/decisions/ADR-2026-05-08-rebranding-straymark.md`) |
| `.straymark/02-design/decisions/ADR-2026-06-02-001-loom-stack.md` | Moved + renamed |
| `.straymark/02-design/decisions/ADR-2026-06-02-002-architecture-plan-format.md` | Moved; internal path reference updated |
| `.straymark/02-design/decisions/ADR-2026-06-03-001-followups-first-class.md` | Moved + renamed |
| `.straymark/02-design/decisions/ADR-2026-06-26-001-code-weave-source.md` | Moved |
| `.straymark/07-ai-audit/decisions/AIDEC-2026-07-13-001-straymark-self-adoption.md` | Moved; `related:` re-pointed, approval note amended with the migration record |
| `.straymark/07-ai-audit/decisions/AIDEC-2026-07-18-001-followups-as-hypotheses.md` | Relative links re-pointed at the co-located ADR (clears the `REF-001` that motivated #368) |
| `docs/decisions/proposals/2026-07-13-self-adoption-implementation-plan.md` | Moved + renamed (id collision resolved); header explains it is not a doctype |
| `docs/decisions/proposals/2026-06-03-windows-parity-coreutils.md` | ADR filename references updated |
| `docs/intro.md` | Decisions-directory statement re-pointed (EN) |
| `docs/contributors/README.md` | Same, plus the AIDEC location (EN) |
| `docs/i18n/es/intro.md`, `docs/i18n/es/contributors/README.md` | Same (es) |
| `docs/i18n/zh-CN/intro.md`, `docs/i18n/zh-CN/contributors/README.md` | Same (zh-CN) |
| `CLAUDE.md` | Decision link + two stale self-adoption statements corrected |
| `CHANGELOG.md` | Historical decision links re-pointed |
| `experiment-loom/{README.md,CHARTER-01-loom-server.md,CHARTER-02-code-weave.md}` | ADR references updated |
| `experiment-loom/specs/{001,002,003}/**` | ADR references updated (6 files) |
| `website/blog/*.md` (5) + `website/i18n/{es,zh-CN}/docusaurus-plugin-content-docs-blog/current/*.md` (10) | GitHub blob URLs re-pointed |
| `.straymark/07-ai-audit/agent-logs/AILOG-2026-07-25-002-migrate-historical-decisions.md` | New — this log |

## Follow-ups

- None. The migration is complete for the seven records that existed; `proposals/` was deliberately left in place with the reasoning recorded above, so there is no residual "phase 2".
