---
charter_id: CHARTER-02-mechanical-verifiers-for-the-markdown-layer-419
status: in-progress
effort_estimate: M
trigger: "Adopter issue #419 (Sentinel, CHARTER-61 external audit rounds 1-2): governance artifacts drift from the tree because nothing mechanically rejects them — three evidenced defect classes with upstream-shaped remediations."
# Exactly one of the following two should be set when the Charter has a known origin.
# Both absent is valid for a Charter scaffolded without an explicit origin (must be
# filled before status moves to in-progress).
# originating_ailogs: [AILOG-YYYY-MM-DD-NNN]
# originating_spec: specs/001-feature/spec.md
# A spec-originated Charter that accrues execution AILOGs records them here at close
# (NOT in originating_ailogs — that stays the single origin). Counterpart for the
# spec-as-context case: context_spec. Neither is subject to the exactly-one rule.
# execution_ailogs: [AILOG-YYYY-MM-DD-NNN]
# context_spec: specs/001-feature/spec.md
# Declared work-classification (Baton #332, optional, declared at authoring — cost ≈ 0).
# work_verb: design | implement | audit | operate. Maps to a routing tier. "Defining a bounded
# foundational contract" is implement, NOT design (design = open-ended architecture/spec).
# design_provenance: new | upstream — only meaningful for implement (upstream degrades to operator).
# An out-of-vocabulary value is an advisory `straymark validate` warning, never blocking.
work_verb: implement
design_provenance: new
---

# Charter: Mechanical verifiers for the markdown layer (#419)

> **Status (mirrored from frontmatter — source of truth is above):** in-progress. Effort: M.
>
> **Origin:** adopter issue [#419](https://github.com/StrangeDaysTech/straymark/issues/419) — Sentinel field report (Sentinel's charter 61, external audit rounds 1–2). No originating AILOG/spec: the origin is the upstream issue itself.

<!-- Charter template — 6 format conventions distilled from the Sentinel /plan-audit
     experiment (6 cycles, 2026-04-28). See the comment block at the end of this file
     for each convention with its empirical justification, and straymark-cli-roadmap.md §3
     plus straymark-thesis-validation.md §3-§5 for the source evidence. -->

## Context

A two-round external audit of a Sentinel Charter (issue #419) diagnosed a generalizable gap: **the code has mechanical verifiers; the governance markdown does not.** Compilation, tests, SAST and DI wiring reject bad code before it lands, but AILOGs, the follow-ups registry, Charters and commit messages are only checked by prose checklists. Three evidenced defect classes: (1) a phantom AILOG citation in a remediation commit message — the id format right, the date plausible, the sequence plausible, and the file absent from the tree; nothing resolves id-shaped references; (2) open follow-ups asserting "no production caller" while the entrypoint wires the job — the registry tracks what AILOGs say, not what the code does; (3) a root-cause lesson that stayed prose in an AILOG and recurred in a new form the next audit round.

The framework owns the id namespaces (AILOG-*, FU-*, CHARTER-*), so referential integrity over them can only be enforced generically upstream. The CLI already has the surfaces: `validate --include-charters` does referential integrity scoped to Charter frontmatter, `validate --staged` is hook-shaped, `analyze declared-vs-wired` proves engine-upstream/rules-downstream, and `followups` owns the drifting artifact. This Charter implements the issue's three proposals in three PRs.

## Scope

**In scope:**

PR 1 — generalized reference resolution (cli minor bump):

1. `cli/src/validation.rs` gains a generalized id tokenizer (`scan_straymark_ids`) covering dated ids (`PREFIX-YYYY-MM-DD-NNN` over `DocType::ALL_PREFIXES`), `FU-NNN(-NNN)` and `CHARTER-NN`, plus an `IdIndex` resolving against discovered documents, charters and the follow-ups registry.
2. REF-001 (`related:` must resolve) is promoted Warning → Error. Defect class: phantom references in frontmatter.
3. New rule REF-003 (Warning): bodies of dated documents and charters are scanned for id-shaped tokens that do not resolve. Defect class: phantom references in prose. Warn-first per design constraint 1 (legacy/shipped content may trip it); flip to Error deferred.
4. `straymark validate --commit-msg <file>`: new mode (blocking from day one) extracting id-shaped tokens from a commit message and failing on any that do not resolve — designed for `commit-msg` hooks the way `--staged` is designed for pre-commit.
5. Unit + integration tests for 1–4; CHANGELOG; AILOG.

PR 2 — `followups verify --claims` (cli minor bump):

6. `regex` + `glob` promoted from optional (`analyze` feature) to required dependencies; the tree walker behind `collect_symbols` is extracted into a shared module reused by both `analyze declared-vs-wired` and the new mode.
7. `followups verify` gains `--claims` (fu_id becomes optional): batch re-derivation of code claims in `open`/`in-progress` entries — backticked paths that no longer exist, backticked symbols absent from the tree, and "no caller / not wired / unused" claims whose symbol now has callers. Warn-first (exit 0). Defect class: registry drift vs the tree (issue case 2).
8. Tests; CHANGELOG; AILOG.

PR 3 — guard-closure in remediation AILOGs (fw + cli):

9. `core/src/document.rs` `Frontmatter` gains optional `guard_closure` (additive); new rule GUARD-001 (Warning): an AILOG with `trigger:` must carry a non-empty `guard_closure`, each item with exactly one of `guard:` / `unguardable:`, and `unguardable:` rationales must be non-generic. Defect class: lesson-as-prose that recurs (issue case 3).
10. `charter amend`'s AILOG template renders the field; STRAYMARK.md §8/§13/§15.B and `dist/STRAYMARK.md` document it; `straymark-audit-review` SKILL.md requires it when consolidating remediation rounds (all platform variants regenerated).
11. Tests; CHANGELOG; AILOG.

**Out of scope:**

- Project-semantics checks (e.g. "integration tests must execute the generated query, not a hand-copied copy") — adopter-side guards or declared-vs-wired profiles, per the issue's own division principle.
- Flipping REF-003 or GUARD-001 from Warning to Error — deferred to a later release once the warn-first baseline is measured (design constraint 1).
- Installing a `commit-msg` hook via `straymark init` — this Charter ships the `--commit-msg` mode and documents the hook snippet; auto-install is a separate decision (hooks are adopter territory).
- JSON output for `followups verify --claims` — the followups namespace is plain-text today; a `--json` flag is a separate enhancement if adopters ask.

## Files to modify

<!-- Reconnaissance first (#210): READ every file before you list it here —
     confirm the path exists in the tree. Charters authored against assumed,
     un-read code drift before execution even begins. `straymark validate
     --include-charters` flags any declared path that does not exist
     (CHARTER-FILES-EXIST). For a file this Charter CREATES, start its Change
     column with "New" (the validator skips existence-checking those).

     Cross-component APIs (#209): if this Charter modifies a contract that other
     components consume — a D-Bus/gRPC/REST interface, a shared trait, an IPC
     method — list ALL consumers of that API as separate rows, not just the
     producer. A mitigation that updates the producer but leaves a consumer
     calling the old contract is the "shipped-mitigation regression" anti-pattern
     (POLISH-CHARTER-PATTERN.md sub-class 5). -->

| File | Change |
|---|---|
| `cli/src/validation.rs` | Generalized id tokenizer + IdIndex; REF-001 → Error; new REF-003 (Warning); new GUARD-001 (Warning) |
| `cli/src/commands/validate.rs` | New `--commit-msg` mode branch mirroring `run_staged` |
| `cli/src/main.rs` | `Validate` gains `--commit-msg`; `FollowupsCommands::Verify` gains `--claims`, fu_id → optional |
| `cli/src/commands/followups/verify.rs` | Dispatch `--claims` to the new batch mode; keep per-entry behavior intact |
| `cli/src/commands/followups/verify_claims.rs` | New — batch claim re-derivation (warn-first) |
| `cli/src/tree_grep.rs` | New — shared tree walker/symbol grep extracted from `collect_symbols` |
| `cli/src/commands/analyze_declared_vs_wired.rs` | Reuse the extracted shared walker instead of its private `collect_symbols` |
| `core/src/document.rs` | `Frontmatter` gains optional `guard_closure` (additive) |
| `cli/src/commands/charter/amend.rs` | Remediation-AILOG template renders `guard_closure` |
| `cli/Cargo.toml` | `regex`+`glob` → required deps; version bumps (3.46.0, 3.47.0, patch for PR 3) |
| `core/Cargo.toml` | Minor bump for the additive Frontmatter field |
| `cli/tests/validate_test.rs` | `--commit-msg` pass/fail, REF-003 warning, REF-001 error |
| `cli/tests/followups_test.rs` | `--claims` batch cases over a fixture tree |
| `cli/tests/charter_amend_test.rs` | Template renders `guard_closure` |
| `STRAYMARK.md` | §8 metadata, §13 quick reference, §15.B amendment convention |
| `dist/STRAYMARK.md` | Synced copy of the above |
| `dist/.claude/skills/straymark-audit-review/SKILL.md` | Require guard-closure when consolidating remediation rounds |
| `dist/.qoder/skills/straymark-audit-review/SKILL.md` | Regenerated/copied variant |
| `dist/.codex/skills/straymark-audit-review/SKILL.md` | Regenerated minimal variant (via `gen_minimal_skills`) |
| `dist/.qwen/skills/straymark-audit-review/SKILL.md` | Copied variant |
| `dist/.agent/skills/straymark-audit-review/SKILL.md` | Copied variant |
| `CHANGELOG.md` | One entry per PR |
| `docs/adopters/CLI-REFERENCE.md` | `validate`: `--commit-msg` flag, REF-001 → Error note, REF-003 bullet (added during PR 1 — scope expansion recorded per closure protocol) |
| `docs/i18n/es/adopters/CLI-REFERENCE.md` | Spanish mirror of the above |
| `docs/i18n/zh-CN/adopters/CLI-REFERENCE.md` | Chinese mirror of the above |
| `.straymark/07-ai-audit/agent-logs/AILOG-2026-08-13-NNN-*.md` | New, one per PR, `risk_level: medium`, `review_required: false` |
| `.straymark/charters/02-mechanical-verifiers-for-the-markdown-layer-419.md` | This Charter — status flips + atomic updates per closure protocol |
| `.straymark/charters/README.md` | Move the CHARTER-02 row to `## Closed` at closure |

## Verification

### Local checks

Commands executable literal in a clean shell — include explicit setup of dependencies.
Any failure of these commands indicates real debt.

```bash
# Build & test (workspace: core + cli)
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets   # warning-free for changed files (the workspace carries pre-existing warnings — no -D gate)

# Dogfooding — run the new gates against this repository itself
cargo run -p straymark-cli -- validate .straymark --include-charters
printf 'fix: close finding, see AILOG-1999-01-01-001\n' > /tmp/msg.txt && \
  cargo run -p straymark-cli -- validate --commit-msg /tmp/msg.txt   # must FAIL (phantom id)
printf 'fix: close finding, see AILOG-2026-08-13-001\n' > /tmp/msg.txt && \
  cargo run -p straymark-cli -- validate --commit-msg /tmp/msg.txt   # must PASS once that AILOG exists
cargo run -p straymark-cli -- followups verify --claims              # warns on drifted claims, exit 0
```

### Production smoke (after deploy)

Not applicable — this Charter ships CLI/framework artifacts, not a deployed service.
Post-merge verification is the release workflow (tagged per-component releases, CI
green) plus the adopter-facing behavior checks already listed under Local checks.

## Risks

- **R1 — REF-001 → Error breaks adopters with dangling `related:` refs**: medium probability / medium severity (validate starts failing on existing repos).
  Mitigation: (a) called out in the CHANGELOG entry as a behavior break; (b) the issue explicitly requests blocking id resolution; (c) the fix for a broken adopter is mechanical (remove or correct the dangling id). If the mitigation fails (adopter backlash), the severity flip is one line to revert — captured as a follow-up.
- **R2 — REF-003 false positives on shipped governance docs and legacy content**: high probability / low severity (warning noise).
  Mitigation: warn-first severity; rule scans only dated instance documents and charters, not all markdown; the warn-first baseline on this repo's own `.straymark/` is measured in the PR 1 AILOG before any future flip to Error.
- **R3 — `--claims` heuristics misfire on semi-structured claim phrasing**: medium / low.
  Mitigation: warn-first with exit 0 by design; claims are only read from backticked tokens (the convention the issue itself proposes); each warning names the entry and the claim so the operator can reword.
- **R4 — promoting `regex`+`glob` to required deps grows the minimal build**: low / low.
  Mitigation: both crates are already in the default build via the `analyze` feature; `--no-default-features` builds gain two small, ubiquitous crates. If this blocks a consumer, the claim-grep can be re-gated — captured as a follow-up.
- **R5 — GUARD-001 keyed on `trigger:` misses remediation AILOGs written without `charter amend`**: medium / low.
  Mitigation: the rule also fires when `amends:` or `findings_closed:` is present (same remediation signature); documented in STRAYMARK.md §15.B so hand-authored remediation AILOGs learn the convention.

## Tasks

1. Sync main, branch `charter-02/pr1-reference-resolution`.
2. PR 1: tokenizer + IdIndex, REF-001 → Error, REF-003 (Warning), `validate --commit-msg`, tests, CHANGELOG, cli 3.46.0. AILOG (`risk_level: medium`, `review_required: false`). Squash-merge (admin).
3. Branch `charter-02/pr2-followups-verify-claims` from updated main.
4. PR 2: deps promoted, shared walker, `followups verify --claims`, tests, CHANGELOG, cli 3.47.0. AILOG. Squash-merge (admin).
5. Branch `charter-02/pr3-guard-closure` from updated main.
6. PR 3: `guard_closure` in core Frontmatter, GUARD-001, amend template, STRAYMARK.md + dist sync, audit-review skill variants, tests, CHANGELOG, fw/cli/core bumps. AILOG. Squash-merge (admin).
7. Local verification passes clean (all commands in §Verification).
8. **Auto-checklist drift** per PR: `straymark charter drift CHARTER-02-mechanical-verifiers-for-the-markdown-layer-419 --range <pr-range>` before each merge; remediate atomically or document as `R<N+1> (new, not in Charter)` in the PR's AILOG.
9. Close: post-merge drift check over the three PR ranges, move the README row, flip status to `closed`.

## Charter Closure

When closing this Charter:

1. **Atomic update (format v4)**: if the drift check (Tasks #7) reported any drift
   not already captured in the AILOG, edit `## Files to modify` and/or add a
   `## Closing notes` block in **this same commit/PR**, before submitting. Do not
   defer to a post-merge housekeeping PR. The atomic-update pattern is the canonical
   way to keep the Charter coherent with execution; deferring it leaves the Charter
   stale and confuses future readers (PLAN-07 of Sentinel demonstrated the failure
   mode that this step prevents).

2. **Post-merge drift check**:
   - Run `straymark charter drift CHARTER-NN --range origin/main...HEAD`, and
     validate the output is clean or that all drifts are documented in the AILOG.
   - This catches the rare case where drift is introduced post-merge (squash
     mangling, admin amendments, etc.) and the atomic step in #1 could not apply.

3. **Move the row** in `.straymark/charters/README.md` to `## Closed` and reference the PR.

4. **Status frontmatter** moves from `in-progress` to `closed` (and optionally
   `closed_at: YYYY-MM-DD` is added — the schema allows arbitrary additional fields).

5. **Do not delete** this file — the planning history matters as much as the AILOG
   of execution.

## Closing notes

> Add this section ONLY when Tasks #7 drift check reported drift that the
> implementer chose to remediate atomically (rather than redoing the implementation
> to match `## Files to modify` exactly). Each bullet: what changed vs declaration,
> why, reference to the AILOG that documented the decision. Omit the section
> entirely if no drift was detected — empty `## Closing notes` is noise.
>
> Historical examples in Sentinel: PLAN-05 (`docs/plans/05-per-service-anomaly-thresholds.md`)
> §Notas de cierre — files removed because the implementation chose a different
> injection point; PLAN-07 (`docs/plans/07-fix-distribution-aligner.md`) §Notas de
> cierre — file removed because the live test was agnostic to the change. Both
> demonstrate the pattern in production usage.

- `[path/file-from-declaration.ext]` [removed | relocated to X | repurposed]:
  [1-2 lines explaining what the implementation did instead and why the original
  declaration is no longer accurate]. Reference: AILOG-YYYY-MM-DD-NNN §[section].

---

<!--
Format conventions — 6 patterns embedded in this template, distilled from the
6-cycle Sentinel /plan-audit experiment (2026-04-28). The provenance is part of the
historical record (in StrayMark terms these are simply "the conventions", not "v2 +
v3 addition" — the partition was Sentinel's iteration log, not structural).

1. Verification splits into `### Local checks` (executable literal in clean shell)
   and `### Production smoke (after deploy)` (not executable without infrastructure).
   Reason: external auditors classified prod-only command failures as `real_debt` —
   avoidable noise. Validated 5/5 cycles after the convention was named.

2. Effort is measured in TIME (XS/S/M/L), not in `~N lines`. Reason: time met the
   estimate (1.0x) in 4/5 cycles; line count drifted 1.0x → 3.1x → 8.1x due to
   AILOG/tests/mocks. Lines are not predictive of cognitive effort.

3. Modifiers like `(optional)` or `(after deploy)` live as structured sub-sections,
   never as inline parenthetical comments. Reason: the Gemini auditor consistently
   ignored parenthetical modifiers and classified marked-optional commands as
   `real_debt`. Validated 2/2 cycles where the pattern applied.

4. R<N> risks are enumerated in the Charter; new risks emergent during execution are
   documented in the AILOG as `R<N+1> (new, not in Charter)`. Reason: cross-validable
   signal by external auditors — they triangulate Charter declarations against AILOG
   emergence. Validated 4/4 cycles where new risks emerged.

5. The `## Charter Closure` section requires the implementer to update the Charter
   doc atomically (same PR as the fix) when drift is detected by Tasks #7, not in
   a separate post-merge housekeeping PR. The `## Closing notes` block is the
   canonical place to document each atomic edit (what changed vs `## Files to
   modify`, why, AILOG reference). Reason: PLAN-07 of Sentinel demonstrated that
   without an explicit atomic-update step, drift remediation can lag the main PR
   by days, leaving the Charter stale and confusing future readers — AIDEC of
   Sentinel 2026-05-02-001 formalized the gap and proposed format v4 (this template
   embodies it).

6. Auto-checklist drift (`straymark charter drift`; Sentinel originally had
   `scripts/check-plan-drift.sh`) runs in pre-commit (Tasks #7) and at
   Charter closure. Detects OMISSION drifts (file declared, not touched) and SCOPE
   EXPANSION drifts (file touched, not declared). Reason: external auditors caught
   implementation-gap and hallucination drifts that the implementer did not document
   in their AILOG. The script catches the same drifts BEFORE commit, separating
   "known and documented" from "forgotten". Zero false positives on 2/2 empirical
   tests against the canonical Sentinel Plans.

7. When a Charter closes an Etapa or SpecKit `Polish` Phase, the polish Charter
   doubles as a debt-detection mechanism — its load-bearing job is to exercise the
   documented operator runbook end-to-end against the real binary (not a test
   harness with mock adapters). See
   `.straymark/00-governance/POLISH-CHARTER-PATTERN.md` for the named anti-pattern
   ("Surface declaration without wiring") it surfaces and the four mechanical
   sub-class checks that cover the common cases. Empirical signal from the
   reference implementation: budget the polish Charter as L (not XS/S/M) and
   expect emergent follow-on Charters, not residual cleanup scope creep.

8. `## Files to modify` is authored from READ code, not assumed code (StrayMark
   findings #209/#210, LNXDrive N=2). Two disciplines: (a) every declared path
   exists in the tree, or is marked "New" in its Change column — the
   `CHARTER-FILES-EXIST` validate rule (cli-3.17.0+) flags violations, separating
   "Charter mis-declared" (authoring bug) from `charter drift`'s "declared but
   not modified" (implementation drift); (b) a change to a cross-component API
   lists ALL consumers, not just the producer — see
   `.straymark/00-governance/POLISH-CHARTER-PATTERN.md` sub-class 5
   ("shipped-mitigation regression via an un-updated downstream consumer").
-->
