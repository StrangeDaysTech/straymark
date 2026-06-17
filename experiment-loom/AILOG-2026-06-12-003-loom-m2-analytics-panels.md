---
id: AILOG-2026-06-12-003
title: Loom M2 — analytics, panels, and server-side graph filters
status: accepted
created: 2026-06-12
agent: codex-gpt-5
confidence: high
review_required: true
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 791
files_modified: [CHANGELOG.md, Cargo.lock, README.md, docs/adopters/CLI-REFERENCE.md, docs/i18n/es/README.md, docs/i18n/es/adopters/CLI-REFERENCE.md, docs/i18n/zh-CN/README.md, docs/i18n/zh-CN/adopters/CLI-REFERENCE.md, experimento/CHANGELOG.md, experimento/Cargo.toml, experimento/specs/001-loom-server/tasks.md, experimento/src/server.rs, experimento/src/snapshot.rs, experimento/web/index.html, experimento/web/package-lock.json, experimento/web/package.json, experimento/web/src/main.ts]
observability_scope: none
tags: [loom, analytics, louvain, filters, panels, knowledge-graph]
related: [AILOG-2026-06-12-002, ADR-2026-06-02-001]
---

# AILOG: Loom M2 — analytics, panels, and server-side graph filters

## Summary

Executed the implementation portion of milestone **M2** of
`CHARTER-01-loom-server` (T2.1–T2.4): Loom now colors the graph by Louvain
community, exposes navigable node and corpus-stat panels, and supports
server-side graph filters for type, status, risk, tag, and inclusive created
date bounds. The component and web package were advanced to `0.2.0`; T2.5
remains open only for the post-merge release tag.

## Context

M1 delivered the live walking skeleton and already exposed basic node detail
plus corpus statistics through the API. M2 turns those foundations into the
analytical dashboard defined by Spec 001 FR8–FR10 without changing Loom's
loopback-only, read-only security posture or the shared `straymark-core`
parser/graph contract.

## Actions Performed

1. **T2.1 — Louvain communities.** Added
   `graphology-communities-louvain`; builds an undirected projection from
   resolved graph edges, computes communities client-side, colors nodes by
   cluster, and renders a live cluster legend. A seeded RNG keeps cluster
   identities and colors stable across unchanged rebuilds.
2. **T2.2 — node summary panel.** Expanded the existing M1 detail panel with
   metadata, tags, body excerpt, and clickable incoming/outgoing relationship
   endpoints. Dangling endpoints remain visible but are not clickable.
3. **T2.3 — corpus stats panel.** Added counts by type/status/risk plus
   navigable orphan and dangling-reference lists. Selecting an item focuses
   the document; if an active filter excludes it, Loom clears the filter and
   reloads the full graph first.
4. **T2.4 — server-side filters.** `/api/graph` now accepts combinable
   `type`, `status`, `risk`, `tag`, `from`, and `to` query parameters.
   Metadata filters are case-insensitive, date bounds are inclusive, and all
   supplied filters combine with AND.
5. **Filtered graph semantics.** Responses contain matching nodes, resolved
   edges only when both endpoints survive, and dangling references whose
   source survives. Counts and lists are recalculated for the filtered view.
6. **Live filtered views.** WebSocket rebuilds continue to apply immediately;
   when filters are active the client refetches `/api/graph` with the active
   query rather than replacing the view with the unfiltered WS snapshot.
7. **Release preparation.** Bumped `straymark-loom` and the web package to
   `0.2.0`, updated `Cargo.lock`, release notes, current-version tables, and
   marked T2.1–T2.4 complete. T2.5 remains pending until the post-merge
   `loom-0.2.0` tag is created.
8. **M2 operator-review polish.** Replaced the exhaustive numeric cluster
   legend with a compact interactive view of the 8 largest non-singleton
   communities. Each entry uses its highest-degree document title as the
   human-facing label; clicking focuses/dims the cluster. The summary reports
   total communities, isolated nodes, and hidden smaller communities.
9. **Excerpt clarity.** `/api/node/:id` now returns `excerpt_truncated`; the
   detail panel labels the body as an excerpt, shows the source path, and
   explicitly signals when more content exists. Full-document reading remains
   deferred to M3.
10. **Operator acceptance on Sentinel.** Reviewed the M2 UI against the
    reference corpus (130 docs / 389 links). Operator feedback identified the
    exhaustive numeric legend and ambiguous truncated body as the remaining
    M2 legibility gaps; both were corrected in this milestone.
11. **Left-column layout polish.** Expanded orphan/dangling lists scroll
    inside a bounded statistics panel and can no longer render underneath the
    community legend. An initial shared transparent rail caused inconsistent
    hit-testing over Sigma's canvas; it was removed in favor of independent
    interactive panels with reserved vertical space.
12. **Panel interaction + live-state fix.** Dynamic actions bind directly to
    their rendered buttons instead of relying on document-level delegation.
    More importantly, community focus and open stats sections now survive
    subsequent WS rebuild renders; previously clicks executed but their state
    was erased in under 500ms, making them appear non-responsive. Panel
    stacking is explicit above Sigma and interactive elements declare
    `pointer-events: auto`.

## Modified Files

| File | Change Description |
|------|--------------------|
| `experimento/src/snapshot.rs` | Filter model, induced filtered graph, recalculated stats, and 2 tests |
| `experimento/src/server.rs` | `/api/graph` query extraction, filtered response, and excerpt truncation signal |
| `experimento/web/src/main.ts` | Louvain analytics, interactive community focus, panels, filters, live filtered rebuilds |
| `experimento/web/index.html` | M2 dashboard layout and styles |
| `experimento/web/package*.json` | Version 0.2.0 and Louvain dependency |
| `experimento/Cargo.toml`, `Cargo.lock` | Loom version 0.2.0 |
| `experimento/specs/001-loom-server/tasks.md` | T2.1–T2.4 completion status |
| changelogs and adopter docs | M2 release notes and current Loom version |

## Decisions Made

- **Louvain remains client-side**, as recommended in the implementation plan,
  keeping the server thin and avoiding analytics dependencies in Rust.
- **Filtering returns an induced graph**, not merely a filtered node list, so
  clients never receive resolved edges whose endpoints are absent.
- **Dangling references remain first-class signals** when their matching
  source survives a filter; this preserves FR2/FR8 behavior.
- **Dates use canonical string comparison** because StrayMark `created`
  values are normalized ISO `YYYY-MM-DD`; documents without a date do not
  match a supplied date bound.
- **T2.5 is not marked complete before merge/tag.** The implementation and
  acceptance work are complete, but the release tag belongs after merge.
- **Community labels use representative document titles**, not generated
  topic names. This makes the legend immediately useful without introducing
  semantic/topic inference before M3 or post-M3 polish.
- **The node panel remains a summary surface.** It signals truncation and
  path clearly; full document reading/open-in-editor remains M3 scope.

## Impact

- **Functionality:** additive analytical UI and API filtering; existing
  unfiltered `/api/graph`, node detail, thread highlighting, and WS rebuilds
  remain available.
- **Performance:** filtered corpora are reduced server-side before transfer;
  Louvain runs client-side over resolved edges. The production JS bundle is
  approximately 207 KB / 53 KB gzip.
- **Security:** unchanged. Loom remains read-only, binds loopback only, and
  rejects forged non-loopback `Host` headers.
- **Dependencies:** added `graphology-communities-louvain` and its transitive
  production dependencies; production dependency audit found no known
  vulnerabilities.

## Verification

- [x] `npm run build` — TypeScript and Vite production build pass.
- [x] `cargo test -p straymark-loom` — 6/6 tests pass, including filter
      semantics and excerpt-truncation coverage.
- [x] `cargo test -p straymark-core` — 21/21 tests pass.
- [x] `cargo test` — complete workspace suite passes.
- [x] `cargo clippy -p straymark-loom --no-deps -- -D warnings` passes.
- [x] `npm audit --omit=dev` reports 0 vulnerabilities.
- [x] `git diff --check` passes.
- [x] Real-server smoke test: embedded/dev asset UI returns HTTP 200;
      `/api/graph?type=ADR` returns the expected filtered graph and
      recalculated stats; forged `Host: evil.example.com` returns HTTP 403.
- [x] Final Sentinel acceptance: compact title-labeled community legend served
      against 130 docs / 389 links; long-document detail returns and displays
      `excerpt_truncated: true`.
- [x] Expanded Sentinel dangling-reference list remains bounded above the
      community legend in the left column.
- [x] Left-panel links, details toggles, and community focus buttons remain
      directly clickable after removing the transparent rail wrapper.
- [x] Physical Chrome WebGL test at 1920x1080: community focus remains active,
      stats details remain open, and a visible node link opens the selection
      panel, all after live rebuilds.
- [x] Full `cargo clippy -p straymark-loom -- -D warnings` was attempted; it
      stops on the pre-existing `clippy::unnecessary_map_or` warning at
      `core/src/graph.rs:251`, outside this milestone's changes.

## Follow-ups

- **T2.5:** merge, run final acceptance in the release revision, then create
  and push tag `loom-0.2.0`.
- **M3 analytics:** cycle/SCC reporting, centrality-based sizing, incremental
  WS deltas, search, pinned subgraphs, and open-in-editor remain deferred.
- **Post-M3 polish candidate:** infer durable human topic names for
  communities if operator use shows representative document titles are not
  sufficient.
- **Reference normalization and Charter nodes:** M1 follow-ups remain open;
  high dangling-reference counts in adopter corpora still reduce graph
  connectivity and therefore community quality.

## Additional Notes

- The development-only Vite/esbuild audit findings documented in M1 remain
  outside the shipped static production bundle; the M2 production dependency
  audit is clean.
- The current UI string table/i18n work remains deferred to M3 per NFR5.

---

<!-- Template: StrayMark | https://strangedays.tech -->
