---
id: AILOG-2026-06-12-001
title: Loom M0 — straymark-core extraction (workspace + shared document model + typed graph)
status: accepted
created: 2026-06-12
agent: claude-code-opus-4.8
confidence: high
review_required: true
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 650
files_modified: [Cargo.toml, core/Cargo.toml, core/README.md, core/src/lib.rs, core/src/document.rs, core/src/graph.rs, cli/Cargo.toml, cli/src/main.rs, cli/src/audit_engine.rs, cli/tests/validate_test.rs, .github/workflows/release-cli.yml, .gitignore, CHANGELOG.md]
observability_scope: none
tags: [loom, refactor, workspace, straymark-core]
related: [ADR-2026-06-02-001]
---

# AILOG: Loom M0 — straymark-core extraction

## Summary

Executed milestone **M0** of `CHARTER-01-loom-server` (tasks T0.1–T0.7 of
`specs/001-loom-server/tasks.md`): converted the repo root into a Cargo
workspace, moved the document model out of the CLI into a new shared
`straymark-core` crate, and generalized the audit engine's traceability
adjacency into a typed, bidirectional, orphan-preserving `core::graph`
builder. **Zero behavior change**, verified against the declared regression
oracle (full test suite + byte-for-byte `straymark audit` output).

## Context

Loom (the experimental visualization server, Spec 001) must parse StrayMark
documents with *exactly* the same code as the CLI so the rendered graph can
never drift from `straymark audit`'s truth (`ADR-2026-06-02-001`, Charter risk
R5). M0 isolates the workspace refactor blast radius (risk R1) in its own
reviewable, bisectable increment before any server code exists.

This PR also lands the previously untracked Loom intention documents
(`experimento/README.md`, SpecKit sets 001/002, the Charter, the component
CHANGELOG, the market-research note) and the two dogfood ADRs
(`ADR-2026-06-02-001` stack, `ADR-2026-06-02-002` plan format), and flips the
Charter to `in-progress`.

## Actions Performed

1. **T0.1** — Root `Cargo.toml` virtual workspace (`members = ["core", "cli"]`,
   `resolver = "2"`); `[profile.release]` (opt-level=z, lto, strip) moved to the
   root (cargo only honors root profiles); `Cargo.lock` moved `cli/` → root;
   `.gitignore` gains `/target/`.
2. **T0.2** — New `straymark-core` crate with crates.io metadata; `document.rs`
   moved verbatim from `cli/src/` (git detects the rename).
3. **T0.3** — CLI import churn: `crate::document` → `straymark_core::document`
   (10 files, 15 occurrences, mechanical sed); `mod document;` removed from
   `main.rs`; `straymark-core = { version = "0.1.0", path = "../core" }` added.
4. **T0.4** — New `core::graph`: one node per document (id falls back to the
   filename stem, `has_explicit_id` records the difference), typed edges per
   Spec 001 §3.2 (`RELATED_TO`, `SUPERSEDES`, `DOCUMENTS_ALTERNATIVE`,
   `CHANGES_API`, `ORIGINATES_FROM`), `resolved: false` dangling references
   kept as first-class signals, bidirectional adjacency, orphan preservation,
   deterministic (input/declaration) ordering. 7 unit tests.
   Additive `Frontmatter` fields parsed for the new edge types: `supersedes`,
   `alternatives_documented`, `originating_ailogs`.
5. **T0.5** — `audit_engine::build_traceability` reimplemented as a projection
   over the shared graph: resolved `RELATED_TO` edges between explicit-id
   documents reproduce the legacy adjacency; root-finding and BFS unchanged.
6. **CI** — `release-cli.yml`: binary paths `cli/target/` → workspace `target/`
   (the build would have shipped stale/missing binaries otherwise), plus an
   idempotent "publish `straymark-core` if this version is not on crates.io"
   step *before* the `straymark-cli` publish (the CLI's versioned dep makes
   core a publish prerequisite).
7. **T0.7** — CLI bump 3.23.0 → **3.23.1** (patch: no user-facing change);
   versioning tables updated (README ×3, CLI-REFERENCE ×3); root CHANGELOG
   entry `## CLI 3.23.1`; `Cargo.lock` regenerated.

## Batch Ledger

### Batch 1 — M0: straymark-core extraction (T0.1–T0.7)

Completed 2026-06-12 — this AILOG's PR. Regression oracle green (see
§Verification). T0.8 (merge + tag `cli-3.23.1`) happens at PR merge.

### Batch 2 — M1: walking skeleton (`loom-0.1.0`)

(pending)

### Batch 3 — M2: analytics + panels (`loom-0.2.0`)

(pending)

### Batch 4 — M3: rich UI (`loom-0.3.0`)

(pending)

## Modified Files

| File | Lines Changed (+/-) | Change Description |
|------|--------------------|--------------------|
| `Cargo.toml` (root) | +10/-0 | New — workspace members + shared release profile |
| `core/src/document.rs` | +6/-0 (rename) | Moved from `cli/src/`; 3 additive frontmatter fields |
| `core/src/graph.rs` | +396/-0 | New — typed bidirectional graph + 7 tests |
| `core/{Cargo.toml,README.md,src/lib.rs}` | +48/-0 | New crate scaffolding + crates.io metadata |
| `cli/src/audit_engine.rs` | +41/-45 | `build_traceability` projected over `core::graph` |
| `cli/src/{main,commands/*,compliance,metrics_engine,validation}.rs` | ±15 | Mechanical import churn |
| `cli/Cargo.toml` | +5/-6 | core dep; profile moved out; version 3.23.1 |
| `cli/tests/validate_test.rs` | +2/-1 | Source-path of `document.rs` updated |
| `Cargo.lock` | moved | `cli/` → workspace root |
| `.github/workflows/release-cli.yml` | +19/-3 | Workspace `target/` paths; publish core before cli |
| `CHANGELOG.md`, README/CLI-REFERENCE ×6 | +22/-6 | 3.23.1 entry + versioning tables |
| `experimento/**`, `docs/decisions/ADR-2026-06-02-*` | +6397/-0 | Intention docs (authored 2026-06-02, previously untracked) |

## Decisions Made

- **Publish `straymark-core` to crates.io** (operator decision, 2026-06-12;
  plan.md §2 recommendation confirmed): the CLI depends on a versioned core,
  and `release-cli.yml` publishes core idempotently before the CLI.
- **Node id collision semantics** in `core::graph`: first document wins
  (documented in code). The legacy code's behavior under duplicate frontmatter
  ids was arbitrary (last-wins HashMap); duplicates are a corpus defect that
  `straymark validate` owns. Microscopic, pathological-only divergence,
  declared here for transparency.
- **No Louvain/communities in M0**: `community` is a Spec 001 §3.1 *computed*
  property delivered in M2 (plan.md recommends client-side first); the core
  graph carries the structure it needs without new dependencies.

## Impact

- **Functionality**: none user-visible. `straymark audit` output byte-for-byte
  identical; all CLI commands unchanged.
- **Performance**: N/A (same algorithms; one extra graph build per audit is
  O(nodes+edges), negligible).
- **Security**: N/A (no new I/O, no network surface; `straymark-core` is
  parse-only).
- **Privacy**: N/A.
- **Environmental**: N/A.

## Verification

- [x] Code compiles without errors (`cargo build` workspace, clean)
- [x] Tests pass — **635 passed, 0 failed** (T0.6 gate; includes 7 new
      `core::graph` tests)
- [x] `straymark audit` regression oracle: JSON, JSON-with-cycle, and text
      outputs diffed byte-for-byte against pre-refactor baselines on two
      fixture corpora (chains+branches+orphan+dangling; full cycle) — identical
- [x] Manual review performed (human review pending at PR — `review_required: true`)
- [ ] Security scan passed — N/A (risk_level: medium)
- [ ] Privacy review completed — N/A (no PII)

## Additional Notes

- The fixture corpora and baselines live in `/tmp` (ephemeral by design); the
  NFR1 consistency invariant gets a *permanent* automated check in M1 (T1.11,
  `/api/graph` ≡ `straymark audit`).
- CLAUDE.md's release instructions still say `git add cli/Cargo.toml
  cli/Cargo.lock` — the lockfile path changed to the root. Updated in this PR.
- `straymark charter drift` was not run against this Charter: this repo does
  not install a root `.straymark/` (it is not an adopter of itself), so the
  charter tooling has no registry here. Drift is covered by this AILOG's
  Modified Files vs. the Charter's `## Files to modify` (one declared row not
  yet touched: `cli/src/commands/loom/*` and `experimento/Cargo.toml` belong
  to M1; `core/src/architecture.rs` to A1 — expected, milestone-scoped).

---

<!-- Template: StrayMark | https://strangedays.tech -->
