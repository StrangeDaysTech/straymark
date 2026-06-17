---
id: AILOG-2026-06-17-001
title: Rename experimento/ → experiment-loom/ — directory rename + hardcoded-path refactor
status: accepted
created: 2026-06-17
agent: claude-code-opus-4.8
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 154
files_modified: [Cargo.toml, .gitignore, .github/workflows/release-loom.yml, CLAUDE.md, cli/src/commands/loom/mod.rs, core/src/architecture/mod.rs, experiment-loom/CHARTER-01-loom-server.md, experiment-loom/README.md, experiment-loom/architecture/model.yml, experiment-loom/architecture/plan.drawio, experiment-loom/specs/001-loom-server/spec.md, experiment-loom/specs/001-loom-server/plan.md, experiment-loom/specs/001-loom-server/tasks.md, experiment-loom/specs/002-architecture-plan/spec.md, experiment-loom/specs/002-architecture-plan/tasks.md, experiment-loom/src/architecture.rs, experiment-loom/src/main.rs, website/blog/2026-06-12-what-the-second-reader-demanded.md, website/blog/2026-06-14-what-the-graph-couldnt-draw-yet.md, website/blog/2026-06-16-where-the-debt-actually-was.md, website/i18n/es/docusaurus-plugin-content-blog/2026-06-12-what-the-second-reader-demanded.md, website/i18n/es/docusaurus-plugin-content-blog/2026-06-14-what-the-graph-couldnt-draw-yet.md, website/i18n/es/docusaurus-plugin-content-blog/2026-06-16-where-the-debt-actually-was.md, website/i18n/zh-CN/docusaurus-plugin-content-blog/2026-06-12-what-the-second-reader-demanded.md, website/i18n/zh-CN/docusaurus-plugin-content-blog/2026-06-14-what-the-graph-couldnt-draw-yet.md, website/i18n/zh-CN/docusaurus-plugin-content-blog/2026-06-16-where-the-debt-actually-was.md]
observability_scope: none
tags: [loom, refactor, rename, paths, hygiene]
related: [CHARTER-01-loom-server, ADR-2026-06-02-001, AILOG-2026-06-15-001]
---

# AILOG: Rename experimento/ → experiment-loom/

## Summary

Renamed the Loom experimental component's directory `experimento/` →
`experiment-loom/` and updated every **hardcoded path** across the repo so the
Cargo workspace, the release CI, the architecture status overlay, and the docs
remain correct after the move. The rename was performed with `git mv` (history
preserved); content edits were **77 one-to-one path substitutions** plus the
whole-directory move (38 files relocated). `cargo check --workspace` passes;
`straymark-loom 0.6.2` compiles from the new path. Shipped on
`refactor/experiment-loom-rename` (PR #289).

The new name was chosen for **symmetry with `experiment-okf/`** (the OKF analysis
folder added in PR #288): both experimental work-trees now share the
`experiment-<topic>` convention, and `experiment-loom` says *what* the experiment
is, where the bare Spanish word `experimento` did not.

## Context

`experimento/` has been the Loom component's home since the M0 core extraction
(`ADR-2026-06-02-001`, `cli-3.23.1`). The name was a placeholder; with a second
experiment folder now in the repo (`experiment-okf/`), the operator asked to
normalize the Loom folder to `experiment-loom/` and to chase down every path that
would break.

The hazard in a rename like this is **partial coverage**: a directory is a string
that appears in build config, CI scripts, source doc-comments, the architecture
model *and* its DrawIO twin, governance docs, and published blog links. Miss one
and the workspace won't resolve, CI won't build, the Loom overlay silently
mismatches, or a published link 404s. The work was therefore driven by an
exhaustive `git grep` inventory, classifying every hit as **functional path**
(must change), **immutable history** (must NOT change), or **false positive** (the
Spanish word "experimento", which must NOT change).

## Actions Performed

1. **Inventory.** `git grep -n 'experimento'` across the tracked tree, then
   refined with `git grep 'experimento/'` (the trailing slash isolates real paths
   from the Spanish noun). Classified every hit (see §Decisions, §Left Intact).
2. **Directory rename.** `git mv experimento experiment-loom` — git recorded the
   moves as renames, preserving file history.
3. **Slash-path substitution.** `experimento/` → `experiment-loom/` across the
   24-file functional change-set (build/CI/code-comments/CLAUDE.md + the live
   design docs inside the dir + the three Loom-arc blog posts in EN/es/zh-CN).
4. **Quoted-bare substitution.** `"experimento"` → `"experiment-loom"` in
   `Cargo.toml` (workspace member), `model.yml` (component `id`), and `CHARTER-01`
   (the members snippet).
5. **Component-id substitution (the subtle one).** A second pass for the bare
   `experimento` token that the slash pattern missed: the `plan.drawio`
   `straymark_component_id` / cell ids, and a few `members = [… "experimento"]` /
   `experimento→core` references inside the specs. This is what keeps the BIM join
   key consistent (see §Decisions).
6. **Build verification.** `cargo check --workspace` — clean; `straymark-loom`
   compiles from `experiment-loom/`. Final `git grep` confirmed no `experimento/`
   path remains outside the immutable AILOGs.

## Modified Files

| File | Change Description |
|------|--------------------|
| *(whole directory)* | `git mv experimento → experiment-loom` (38 files relocated, history preserved) |
| `Cargo.toml` | workspace member `"experimento"` → `"experiment-loom"` |
| `.gitignore` | Loom web `node_modules`/`dist` ignore paths + comment |
| `.github/workflows/release-loom.yml` | `cache-dependency-path`, `--manifest-path`, `working-directory`, version-check grep, comment |
| `CLAUDE.md` | project-structure tree + Loom component bullet + Loom release-workflow section |
| `cli/src/commands/loom/mod.rs`, `core/src/architecture/mod.rs` | doc-comment path references |
| `experiment-loom/CHARTER-01-loom-server.md` | `originating_spec` + body path refs + workspace-members snippet |
| `experiment-loom/README.md` | directory-tree path |
| `experiment-loom/architecture/model.yml` | component `id` + `globs` → `experiment-loom` |
| `experiment-loom/architecture/plan.drawio` | `straymark_component_id` + cell/edge ids → `experiment-loom` (BIM join key) |
| `experiment-loom/specs/001-loom-server/{spec,plan,tasks}.md` | path refs + workspace-member snippet |
| `experiment-loom/specs/002-architecture-plan/{spec,tasks}.md` | path refs + component-id references |
| `experiment-loom/src/{architecture,main}.rs` | doc-comment path references |
| `website/blog/2026-06-1{2,4,6}-*.md` (EN) | GitHub `/blob/main/experimento/…` links + one prose path |
| `website/i18n/{es,zh-CN}/…/2026-06-1{2,4,6}-*.md` | same links/prose in the translated posts |

## Decisions Made

- **New name `experiment-loom` (not e.g. `loom/` or keeping `experimento/`).**
  Chosen for symmetry with `experiment-okf/` (PR #288) so the repo has a single
  `experiment-<topic>` convention for exploratory work-trees, and because the
  component is still **experimental (v0/N=1)** — graduating it out of an
  `experiment-*` name is a separate, later decision (tracked as G.2 in the specs),
  not this hygiene rename.
- **`git mv`, not delete+add.** Preserves per-file history through the rename, so
  `git log --follow` and blame still work across the boundary.
- **The trailing-slash inventory (`experimento/`) as the primary filter.** The
  string `experimento` is also the Spanish word for "experiment", which appears
  throughout the es-locale docs and older blog posts ("el experimento Sentinel").
  Filtering on `experimento/` (with the slash) isolated real filesystem paths from
  the noun, preventing a destructive blanket replace. The few bare-token
  exceptions (workspace member, component id) were handled explicitly.
- **`model.yml` id and `plan.drawio` `straymark_component_id` changed in
  lockstep.** These two are the **BIM join key** — the architecture projection
  matches a model component to its DrawIO cell by this id. Changing one without
  the other would silently break the Loom "you are here" overlay and `status
  --where` for the Loom component (no test guards this; it's a runtime match), so
  both moved together to `experiment-loom`. This was the one hit the slash-pattern
  missed (the id is a bare token, no slash) and the reason for the second pass.
- **`core` dependency pins NOT bumped.** This is a path/name change only — no crate
  version moved, so `cli`/`experiment-loom` `straymark-core` pins are untouched and
  `Cargo.lock` was not rewritten (the lockfile keys on crate name, not directory).
- **Blog GitHub links repaired despite the posts being "published."** The footers
  of the three Loom-arc posts link to `github.com/.../blob/main/experimento/…`
  (the CHARTER and the specs). After the rename those targets 404. Repairing a
  link is fixing a defect, not rewriting the narrative, so the links — and the one
  prose path ("the server lives entirely in `experimento/`") — were updated to
  `experiment-loom`. Distinct from the immutable-history rule below: blog posts
  carry no formal immutability guarantee; AILOGs and ADRs do.

## What Was Deliberately Left Intact

This is the half the operator explicitly asked to preserve. Each was a conscious
call, not an oversight:

- **The 9 AILOGs inside the directory** (`experiment-loom/AILOG-2026-06-1*.md`).
  AILOGs are **append-only, immutable records** of work as it happened. When those
  logs were written, the directory *was* called `experimento/`; that is a true
  historical fact. Rewriting them to say `experiment-loom/` would falsify the
  record — the precise failure mode the AILOG convention exists to prevent. They
  keep their original `experimento/` references, and `git mv` means they now
  physically live under `experiment-loom/` while their bodies still narrate the
  name of their era. This is correct, not inconsistent.
- **`docs/decisions/` ADRs and proposals.** `ADR-2026-06-02-001` (loom stack) and
  `-002` (architecture plan) record decisions taken when the folder was
  `experimento/`, including paths in their context/consequences. ADRs are decision
  records; even in `draft` they document a point-in-time rationale. Treated as
  immutable history, same as the AILOGs — consistency over a marginal
  navigability gain. (A reader following an ADR path will land on a moved file,
  but the decision text stays faithful to its moment.)
- **Root `CHANGELOG.md`.** The `cli-3.23.1` entry describing the M0 extraction
  mentions `experimento/` as the component's home at release time — a historical
  release note, left verbatim.
- **The Spanish word "experimento"** in es-locale docs (`docs/i18n/es/…`,
  `contributors/*`, `adopters/*`), the charter template, `blog-excerpts.json`, and
  the April/May blog posts ("el experimento Sentinel", "primer experimento
  sistemático"). These are the common noun, never the directory. Untouched.
- **The `kebab()` unit test** in `cli/src/commands/architecture/common.rs:400`
  (`assert_eq!(kebab("  experimento  "), "experimento")`). Here `"experimento"` is
  an arbitrary example string exercising whitespace-trim + kebab-casing — not a
  path. Renaming it would break the assertion's intent for no reason. Untouched.

## Impact

- **Functionality**: none. Pure rename + path-reference update; no behavior,
  command, API, or output changes. `straymark-loom` is byte-for-byte the same
  binary, built from a new directory.
- **Build/CI**: `release-loom.yml` now references `experiment-loom/`; a future
  `loom-*` tag build will resolve correctly. Workspace resolves; `Cargo.lock`
  unchanged.
- **Architecture overlay**: `model.yml` ↔ `plan.drawio` join key preserved, so the
  Loom dogfood overlay and `status --where` for the Loom component still match.
- **Docs/site**: the three Loom-arc blog posts' GitHub links resolve post-merge;
  no MDX/structure change (the deploy pipeline rebuilds on merge).
- **Performance / Security / Privacy**: N/A.

## Verification

- [x] `cargo check --workspace` clean — `straymark-loom 0.6.2` compiles from `experiment-loom/`
- [x] `git grep 'experimento/'` returns **no path** outside the immutable AILOGs
- [x] `plan.drawio` `straymark_component_id="experiment-loom"` matches `model.yml` `id: "experiment-loom"` (overlay integrity)
- [x] English word "experimental" (e.g. "experimental dashboard" label) untouched by the substitutions
- [x] `git` recorded the directory move as renames (history preserved)
- [x] Manual review performed (human review pending at PR — `review_required: true`)
- [ ] Security scan — N/A (risk_level: low)
- [ ] Privacy review — N/A (no PII)

## Additional Notes

- **Why an AILOG for a rename.** A directory rename is low-risk but high-surface;
  the value of this record is the **classification rationale** — which references
  are functional paths, which are immutable history, which are false positives —
  so a future reader (or a future rename, e.g. graduating Loom out of an
  `experiment-*` name) inherits the reasoning instead of re-deriving it.
- **Follow-up (not done here).** If Loom ever graduates from experimental, a
  second rename (`experiment-loom/` → `loom/`) will face the same surface; this
  AILOG is the template for that pass. The specs' G.2 task tracks the graduation
  decision itself.

---

<!-- Template: StrayMark | https://strangedays.tech -->
