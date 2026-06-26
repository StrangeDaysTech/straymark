---
id: AILOG-2026-06-26-001
title: Baton — per-type → endpoint keying for generated type files (call-site binding)
status: accepted
created: 2026-06-26
agent: claude-code-opus-4.8
confidence: high
review_required: true
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 280
files_modified: [experiment-baton/src/codescan.rs, experiment-baton/src/scan.rs, experiment-baton/tests/generated_keying.rs, experiment-baton/tests/fixtures/generated-types-project]
observability_scope: none
tags: [baton, coherence, keying, codescan, speckit, generated-types]
related: [AILOG-2026-06-25-005, CHARTER-01-coherence-bridge]
---

# AILOG: Baton — per-type → endpoint keying for generated type files

## Summary

Closes #313. The coherence engine keyed a contract's *shape* (fields, enum
variants) by the **nearest `/api/...` anchor at or above a declaration**. On a
generated types file (`web/src/api/types.gen.ts`) that holds all API types in one
file with sparse endpoint comments, every interface collapsed onto a few coarse
`ContractId`s (on Sentinel: all of them onto `services`), so the #304 field/enum
mismatch (C2/C3) could never isolate on the right contract. That is why the
Sentinel dogfood reported **0 blocking by design**.

This adds a second, higher-priority keying source — **call-site binding** — that
keys a *type* to its *route* from the usage site, exactly where the association
actually lives:

```ts
api.get<HealthSnapshot>(`/services/${serviceId}/health`)   // HealthSnapshot → services.health
api.get<ListIncidentsResponse>('/incidents')               // ListIncidentsResponse → incidents
```

This is the issue's second proposed option ("associate a TS type to a route via
the API client call site"). It is more general than parsing OpenAPI — Sentinel
does not even commit an OpenAPI file (it is gated; see its `FU-005-001`).

## What changed

1. **Call-site binding pre-pass** (`codescan::collect_bindings`). A repo-wide scan
   of TS files mines `IDENT<Type>('/route')` into a `Type → ContractId` map.
   Conservative (R6): a type bound to two *different* routes is dropped as
   ambiguous; only a quoted first argument that starts with `/` counts (so
   `useState<T>('loading')`, `useQuery<T>({…})` and JSX `<Table<Row>` never bind).

2. **Binding-first keying** (`codescan::extract_file`). Each declaration resolves
   its contract as `binding(type) ?? nearest_anchor(line)`. An anchorless
   declaration with no binding still yields nothing (unchanged conservatism), but
   a *bound* declaration now keys even when its file has no anchor at all.

3. **Enum-by-reference resolution.** A generated file binds the *interface* to a
   route, but the `type X = 'A' | 'B'` it uses sits in a sibling decl with no
   anchor/binding of its own. An interface keyed to a contract now attributes the
   union its fields reference to the same contract — needed for C3 to see the
   consumer's enum variants.

4. **Import-specifier false anchors** (`scan::scan_endpoints`). `import … from
   '@/api/types.gen'` was scanned as an endpoint and produced a bogus `types.gen`
   contract. A `/api/` glued to an identifier/alias/relative path (preceded by an
   alphanumeric, `_`, `@`, or `.`) is now skipped — it is a module specifier, not
   an HTTP endpoint. This also tightens spec-prose scanning.

5. **`normalize_endpoint` robustness.** Strips a query string (`?…`) and drops
   template params (`${id}`) in addition to `{id}` / `:id`, so call-site relative
   paths normalize to the same `ContractId` a Go handler comment yields.

## Verification

- `cargo test -p straymark-baton` ✓ — 47 tests (6 new: 3 unit in `codescan`, 3
  integration in `generated_keying.rs`), no regressions; `cargo clippy` clean.
- New fixture `tests/fixtures/generated-types-project/`: an anchorless
  `types.gen.ts` whose two interfaces bind to two routes via call sites. C2 + C3
  fire on `services.health` (orphan `status`/`cpu`, enum `OPERATIONAL…` vs
  `GREEN…`); the matching `services` sibling stays silent. Proves per-contract
  isolation — the issue's "Done when".
- **Sentinel dogfood** (read-only, `git status` unchanged): the generated
  consumers now spread across the correct contracts instead of collapsing onto
  `services` — `HealthSnapshot → services.health`, `SearchRecordsResponse →
  audit.records`, `IncidentDetailResponse → incidents`, `ReopenResponse →
  incidents.reopen` — and the bogus `types.gen` contract is gone. The run stays
  **0 blocking**, correctly: Sentinel's `types.gen.ts` was remediated to match the
  Go backend, so there is no drift to flag (de-collapsing introduced no false
  positives — R6 holds). The #304 C4 cross-spec finding still fires.

## Scope boundary — producer-side keying gap (filed separately)

Keying the *consumer* (generated type) was #313's chartered scope and is done. The
dogfood surfaced the **symmetric** gap on the producer side: Sentinel's Go backend
uses the `huma` framework, which registers every route in one block
(`huma.Get(api, "/api/v1/services/{id}/health", h.getServiceHealth)`) while the
response struct `getServiceHealthOutput` is defined far below. Nearest-anchor
mis-keys it away from `services.health` (the contract shows `producer=None`). The
same binding idea applies — bind the route in the registration call to the
handler, then the handler's output struct — but it is framework-shaped and beyond
the #313 scope. It does **not** change the Sentinel result (the repo is remediated →
0 blocking either way), and the capability is already demonstrated on the fixture
(whose Go producer carries a normal comment anchor). Filed as **#319** so C2/C3
can fire end-to-end on an un-remediated `huma`-style repo.

## EU AI Act Considerations

Not applicable — local developer tooling; no automated decision-making, no
personal data, no model inference. Read-only over the target tree (NFR1).
