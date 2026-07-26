---
id: AILOG-2026-07-25-001
title: charter audit --prepare excludes audit artifacts from the embedded diff (#372) + AuditArgs refactor (#356)
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
tags: [audit, isolation, cli, adopter-feedback, refactor, self-adoption]
related: []
---

# AILOG: Auditor isolation at prompt-build time (#372) + `AuditArgs` refactor (#356)

## Summary

`straymark charter audit --prepare` embedded the full git diff of the audited range in the prompt. Since the audit flow tells reports and reviews to live under `.straymark/audits/`, a commit that lands them put **round N-1's reports and consolidated review inside the very diff round N was asked to audit**. The embedded diff now excludes `.straymark/audits/**`, with `--include-audit-artifacts` as the opt-back-in. Bundled with the deferred `audit::run` argument-struct cleanup (#356), which touches the same file. Ships as **cli-3.38.0** (framework unchanged).

## Context

Reported by the Sentinel adopter (#372) from a 4-round audit cycle on CHARTER-55: **1,092 of 1,581 embedded diff lines were prior-round audit prose**. Two observed contaminations — one auditor declared `prior_round: report-<self>.md` in its own frontmatter and inherited that round's *scope* (auditing the whole Charter instead of the one commit); another resubmitted its round-2 report byte-for-byte, three of whose four findings the audited commit had already fixed.

The failure is epistemic, not cosmetic. Cross-model convergence is only signal when each auditor reached it independently; N reports that inherited one framing are one data point wearing N hats, and the inflation lands hardest on exactly the findings that most need independent confirmation.

Sentinel first tried the obvious mitigation — moving prior reports into `ronda-N/` subdirs with a README saying "do not read these". **Two of four auditors read them anyway.** A rule a model can rationalize as "useful context" is not isolation.

## Actions Performed

1. **Exclusion at the diff.** `run_git_diff` takes `include_audit_artifacts` and, when false, appends `-- :(exclude).straymark/audits`. Negative-only pathspec, so scope stays "everything else" — including when the project root is a subdirectory of the git repo (no positive `.` pathspec that would silently narrow the diff).
2. **Escape hatch.** `--include-audit-artifacts` on `charter audit`, off by default, for the rare case where auditing the audit trail itself is the point.
3. **Visibility either way.** `report_audit_artifacts_in_range` runs `git diff --name-only` and reports what the range carried: excluded → an info line listing the dropped reports/reviews; included → a warning that this round's convergence is not independent evidence. Advisory only; a git failure here is silent.
4. **#356 refactor.** `audit::run` now takes an `AuditArgs<'_>` struct (was 9 positional args, over clippy's limit since before `--round` was threaded in #341); `run_prepare` takes a `PrepareArgs<'_>` so the new flag didn't push it over the same limit. Also fixed the standing `useless_asref` in the same function.
5. **Docs** — CLI-REFERENCE (EN + es + zh-CN): the new flag in the argument table plus an "Auditor isolation" subsection carrying the prevention-beats-detection rationale. Version tables ×3 locales, README ×3, `CHANGELOG.md`.

## Decisions Made

- **Prevent at `--prepare`, don't detect at `--review`.** The review skill's contamination guard stays, but it can only *flag* a contaminated report, never un-contaminate it — and a flagged report is a wasted audit. The cheapest place to enforce independence is where the prompt is built.
- **Exclude the whole `.straymark/audits/` subtree, not just `report-*.md`.** Audit artifacts are governance byproduct that never constitutes the object of an audit; a narrower filter would leak whatever new artifact shape the flow grows next.
- **The contamination *warning* is narrower than the exclusion** — only `report-*.md` and `review.md` count as contaminating. The resolved `audit-prompt.md` is the auditor's own instruction sheet, so listing it would be noise.
- **Kept the deprecated `--calibrate` / `--finalize` flags** while restructuring into `AuditArgs` (#356 named dropping them as an open judgment call). They already `bail!`/alias; removing a hidden flag is a separate, breaking decision.

## Verification

- Full workspace `cargo test` green; 5 new tests. Two unit tests pin the artifact classification (reports/reviews across flat and `ronda-N/` layouts; `audit-prompt.md` and report-shaped files outside `.straymark/audits/` correctly ignored).
- Three integration tests build a real git repo reproducing #372 — a commit carrying both a code change and the prior round's `report-*.md` + `review.md` — and assert: the resolved prompt contains **neither** report's content while **keeping** the code change (the exclusion is surgical); `--include-audit-artifacts` restores it and warns; a range with no audit artifacts prints nothing.
- `cargo clippy -p straymark-cli`: `audit.rs` is now warning-free (both pre-existing warnings cleared; the remaining `too_many_arguments` in `approve.rs` is untouched and out of scope).

## Risk

Low, and the one real consideration is **under-coverage**: the exclusion silently narrows what auditors see. Mitigated by the fact that the excluded subtree is by construction governance prose, never code, plus the printed list of what was dropped on every run that touches it. The escape hatch covers the audit-the-audit-trail case.

## Modified Files

| File | Change |
|------|--------|
| `cli/src/commands/charter/audit.rs` | Pathspec exclusion, artifact detection/reporting, `AuditArgs`/`PrepareArgs`, unit tests |
| `cli/src/main.rs` | `--include-audit-artifacts` flag + struct-based dispatch |
| `cli/tests/charter_audit_test.rs` | 3 integration tests reproducing the #372 condition |
| `cli/Cargo.toml` | Version 3.38.0 |
| `docs/adopters/CLI-REFERENCE.md` | Flag + "Auditor isolation" section (EN + es + zh-CN) |
| `README.md` | Version table (EN + es + zh-CN) |
| `CHANGELOG.md` | CLI 3.38.0 entry |

## Follow-ups

- None. The issue's optional secondary ask (warn when the range contains audit artifacts) shipped as part of the main fix rather than as a separate nice-to-have.
