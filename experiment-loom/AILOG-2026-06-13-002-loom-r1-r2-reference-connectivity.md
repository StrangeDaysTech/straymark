---
id: AILOG-2026-06-13-002
title: Loom connectivity — reference normalization (R1) + charter/plan/audit nodes (R2)
status: accepted
created: 2026-06-13
agent: claude-fable-5
confidence: high
review_required: true
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 900
files_modified: [CHANGELOG.md, Cargo.lock, README.md, cli/Cargo.toml, cli/src/audit_engine.rs, core/Cargo.toml, core/src/entities.rs, core/src/graph.rs, core/src/lib.rs, docs/adopters/CLI-REFERENCE.md, docs/i18n/es/README.md, docs/i18n/es/adopters/CLI-REFERENCE.md, docs/i18n/zh-CN/README.md, docs/i18n/zh-CN/adopters/CLI-REFERENCE.md, experimento/CHANGELOG.md, experimento/Cargo.toml, experimento/src/snapshot.rs, experimento/web/package.json, experimento/web/src/main.ts]
observability_scope: none
tags: [loom, connectivity, references, charters, plans, audits, knowledge-graph]
related: [AILOG-2026-06-13-001, AILOG-2026-06-12-002, ADR-2026-06-02-001]
---

# AILOG: Loom connectivity — R1 (reference normalization) + R2 (entity nodes)

## Summary

Dogfooding Loom against the Sentinel corpus surfaced that **330 of 395 references were
dangling**. Two follow-ups close the gap, both in the shared `straymark-core` graph builder
so `straymark audit` gains the same connectivity:

- **R1 — reference normalization** (shipped earlier as PR #245, `straymark-core`): resolve an
  edge target by exact id and, failing that, by unique file basename, leading dated id prefix
  — never an ambiguous match.
- **R2 — charter/plan/audit nodes** (this change): discover the corpus's non-document
  entities and inject them as graph nodes so references to them resolve, plus relative-path
  and `CHARTER-NN`-prefix resolution.

Result on Sentinel: dangling references **330 → 87**, nodes **131 → 193**, orphans **2 → 0**.
Released together as `loom-0.4.0` (`straymark-core` → 0.4.0).

## Context

R1 (already merged) recovered 146 references written by filename instead of id (330 → 184).
The remaining 184 were references to entities StrayMark cross-links but does not model as
graph nodes: charters (`.straymark/charters/NN-slug.md`), plan telemetry
(`plans/PLAN-NN.telemetry.yaml`) and audit reviews (`audits/<charter>/review.md`). These are
not `DocType`s and are not picked up by `discover_documents` (which only matches
`TYPE-YYYY-MM-DD-NNN-*.md`). The remaining ~87 references are to files outside the governance
corpus (`.specify/memory/…`, `constitution.md`) and correctly stay dangling.

## Actions Performed

1. **`straymark-core::entities`** (new module): `discover_entities(straymark_dir)` finds
   charters (frontmatter `charter_id`, status, `originating_ailogs`, H1 title), audit reviews
   (frontmatter `charter_id`; id = relative path; link to its charter), and plans (telemetry
   `plan_id`/`plan_title`/`closed_at`). Missing subdirectories yield no entities.
2. **`Graph::build_with_entities`**: `Graph::build` now delegates to it with no entities (all
   existing callers/tests unchanged). It appends entity nodes after the documents and adds
   their declared links as edges.
3. **Resolution fallbacks** (`resolve_target`): exact id → unique basename (R1) → unique
   **relative-path suffix** (R2, for `…/audits/CHARTER-13/review.md` where `review.md` alone
   is ambiguous) → **`CHARTER-NN` prefix** (R2, boundary-safe) → leading dated id prefix (R1).
   Resolved targets are canonicalized to the node id.
4. **Callers** (consistency / NFR1): Loom `snapshot.rs::build_cached` and the CLI
   `audit_engine::generate_audit` (which already has `straymark_dir`) both discover entities
   and call `build_with_entities`, so `/api/graph` ≡ `straymark audit`.
5. **Frontend**: `CHARTER` / `PLAN` / `AUDIT` colors in `TYPE_COLORS`; the stats panel surfaces
   their counts automatically.
6. **Release**: `straymark-loom` + web → 0.4.0, `straymark-core` → 0.4.0 (additive API; CLI
   dependency bumped to match), changelogs (R1 + R2), current-version tables.

## Decisions Made

- **Entities live in the shared graph, not as `DocType`s.** A `DocType::Charter` would ripple
  through `validate`/`compliance`/`metrics` and the CI prefix sync; entities are a graph-only
  concept (`Node.doc_type` is already a free string).
- **Shared builder, so `audit` benefits too.** The user accepted this scope; it also seeds the
  Architecture (A1) track, which needs charters in the model. CLI fixtures have no
  charter/plan/audit dirs, so `discover_entities` is empty and the suite is unchanged.
- **Ambiguous references never resolve.** Every fallback index maps a shared key to `None`, so
  a reference is dropped to dangling rather than wired to the wrong node.
- **External references stay dangling.** `.specify/memory/…`, `constitution.md` and other
  non-corpus files are not modeled — dangling is the correct signal for them.
- **Plans without telemetry (PLAN-04/07) and yaml-only audit refs stay dangling** — there is no
  artifact to model.

## Impact

- **Functionality:** additive. The graph densifies (+62 nodes, +47 edges on Sentinel);
  existing endpoints/panels/filters/deltas are unchanged (they are `doc_type`-agnostic).
- **`straymark audit`:** traceability now includes charter/plan/audit nodes (a correctness
  improvement — they are part of the audit trail).
- **Performance:** entities are re-discovered per rebuild (few files); the R1 parse cache is
  untouched.
- **Security:** unchanged (loopback-only, read-only).

## Verification

- [x] `cargo test` — `straymark-core` 42/42 (incl. entity discovery + resolution tests),
      `straymark-loom` 9/9.
- [x] `cargo test -p straymark-cli` — all suites pass **unchanged** (M0 regression oracle;
      fixtures have no entity dirs).
- [x] `cargo clippy -p straymark-core -p straymark-loom -- -D warnings` — clean.
- [x] `npm run build` — pass.
- [x] Sentinel smoke test: dangling **330 → 87**, nodes **131 → 193** (41 CHARTER, 5 PLAN, 16
      AUDIT), orphans **2 → 0**; forged `Host` → 403.

## Follow-ups

- The remaining ~87 dangling are external references — candidates for an "external/context"
  visual treatment later, but not nodes.
- **R3** — visual density at 100+ nodes (deferred polish).
- **Architecture track A1/A2** (Spec 002) — the next major Loom frontier; charters now in the
  shared model are its groundwork.
- Pre-existing CLI clippy debt (`manual_checked_ops` etc. in untouched files) remains outside
  scope.

## Additional Notes

- `straymark-core` 0.4.0 is not published to crates.io by this Loom release; it ships on the
  next CLI release, where `release-cli.yml` publishes core before the CLI.

---

<!-- Template: StrayMark | https://strangedays.tech -->
