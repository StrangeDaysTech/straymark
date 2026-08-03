---
id: AILOG-2026-08-02-002
title: Baton — close Track A item A4 (CLI-REFERENCE docs) + fw 4.39.0 bump
status: accepted
created: 2026-08-02
agent: qoder
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 30
files_modified: [docs/adopters/CLI-REFERENCE.md, docs/i18n/es/adopters/CLI-REFERENCE.md, docs/i18n/zh-CN/adopters/CLI-REFERENCE.md, README.md, docs/i18n/es/README.md, docs/i18n/zh-CN/README.md, dist/dist-manifest.yml, CHANGELOG.md]
observability_scope: none
work_verb: operate
design_provenance: upstream
tags: [baton, track-a, work-verb, documentation, version-bump]
related: [AILOG-2026-08-02-001, 06-work-verb-schema-ratification, PLAN-avance-post-calibracion]
---

# AILOG: Baton — Track A item A4 closed; Framework 4.39.0

## Summary

Closes the one loose end left by the 4.38.0 graduation: Track A item **A4**
asked for `docs/adopters/CLI-REFERENCE.md` to document the new fields, and the
shipped release never carried that documentation. This change adds it (EN/es/
zh-CN) and bumps the Framework to **4.39.0**, packaging the Track C adopter kit
(AILOG-2026-08-02-001) in the same release note.

## What changed

- **`docs/adopters/CLI-REFERENCE.md` ×3 locales** — two insertions each:
  (1) `straymark validate` gains the advisory work-classification vocabulary
  check (charter frontmatter + follow-up entries; absent → silent, invalid →
  non-blocking warning); (2) `straymark charter new` documents the optional
  `work_verb` / `design_provenance` frontmatter fields with the two
  load-bearing determination rules (foundational contract = `implement`;
  `implement`+`upstream` degrades to mechanical).
- **Version bump fw 4.38.1 → 4.39.0** — `dist/dist-manifest.yml`, version
  tables in README ×3 locales and CLI-REFERENCE ×3 locales.
- **CHANGELOG.md** — Framework 4.39.0 entry; explicit that this is a
  documentation-only release (no `dist/` content beyond the version).

## Why a bump at all

The repo policy (AILOG-2026-07-25-002) is "no bump when nothing adopter-visible
changed". A4's documentation IS adopter-visible (the reference adopters consult
for the fields 4.38.0 added to their templates), and the Track C kit is the
adopter handoff that makes those fields actionable. The CHANGELOG entry states
plainly that no `dist/` content changed, so the release record stays honest.

## Verification

Docs-only change; no code touched. Insertions cross-checked against the ratified
schema (06-work-verb-schema-ratification.md), the graduated dist templates, and
the validate checks in `cli/src/validation.rs` (`check_charter_work_verb`,
`check_followups_work_verb`). Version references grepped repo-wide: all
`fw-4.38.1` occurrences updated (CHANGELOG history untouched, as intended).

## EU AI Act Considerations

Not applicable — documentation and version bookkeeping; no model inference, no
personal data.
