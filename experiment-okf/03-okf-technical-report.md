# OKF v0.1 technical report — spec read, field mapping, export/import distance

*Prepared 2026-06-17. Source: OKF `SPEC.md` v0.1 (Draft), Apache 2.0,
`GoogleCloudPlatform/knowledge-catalog/okf`. This is plan (a): a precise read of
the real spec and a measurement of how far StrayMark is from emitting/consuming
OKF.*

## 1. The spec, precisely

OKF v0.1 normative rules (the only hard constraints — everything else is SHOULD):

- A **bundle** is a directory tree of UTF-8 markdown files. Distributable as a
  git repo (recommended), tarball/zip, or a subdirectory of a larger repo.
- A **concept** is one `.md` file = YAML frontmatter (delimited by `---`) + a
  markdown body. **Concept ID = file path minus `.md`** (e.g. `tables/users.md`
  → `tables/users`).
- **Frontmatter:** exactly one **REQUIRED** field, `type` (free string, not
  centrally registered, consumers must tolerate unknown values). Recommended
  optional fields: `title`, `description`, `resource` (canonical URI of the
  underlying asset), `tags` (list), `timestamp` (ISO 8601). **Producers MAY add
  any keys; consumers SHOULD preserve unknown keys and MUST NOT reject unknown
  ones.**
- **Body:** free-form markdown; conventional (not required) headings `# Schema`,
  `# Examples`, `# Citations`.
- **Cross-links:** standard markdown links. Two forms — **absolute
  (bundle-relative)** `/tables/customers.md` (recommended, move-stable) and
  relative `./other.md`. **Links are untyped**: "the specific kind of
  relationship … is conveyed by the surrounding prose, not by the link itself."
  Consumers MUST tolerate broken links ("not-yet-written knowledge").
- **Reserved files:** `index.md` (directory listing for progressive disclosure;
  *no frontmatter* except an optional bundle-root `okf_version`), `log.md`
  (date-grouped change history, newest first, ISO dates).
- **Citations:** `# Citations` heading, numbered, may point to URLs,
  bundle-relative paths, or a `references/` subtree.
- **Conformance (§9):** a bundle conforms if every non-reserved `.md` has
  parseable frontmatter, every frontmatter has a non-empty `type`, and reserved
  files follow their shape. Everything else is soft. Consumers must not reject
  on missing optional fields, unknown `type`, unknown keys, broken links, or
  missing `index.md`.
- **Versioning (§11):** `<major>.<minor>`; minor = backward-compatible
  additions. Bundle may declare `okf_version: "0.1"` in root `index.md`.

## 2. Field-by-field mapping: StrayMark → OKF

| OKF field | StrayMark source | Notes |
|---|---|---|
| `type` (REQUIRED) | `DocType` (ADR, AILOG, Charter, TDE, AIDEC, MCARD, SEC, ETH, …) | Direct. Emit a descriptive string, e.g. `Architecture Decision Record`, `AI Work Log`. Trivial map table. |
| `title` | frontmatter title / first `# H1` | Present on nearly all docs. |
| `description` | summary/description field or derived first line | Many docs carry an explicit summary; else derive. |
| `resource` | usually **absent** (governance docs describe decisions, not assets) | OKF allows omission for "abstract ideas." For Charters/AILOGs that name source files, `resource` could point at the repo path or the PR/issue URL. Optional. |
| `tags` | `tags` frontmatter | Already a list. Direct. |
| `timestamp` | `created` / `updated` date | Normalize to ISO 8601 datetime. |
| *(extensions)* | `status`, `related`, `supersedes`, `originating_ailogs`, `alternatives_documented`, `api_changes`, `risk_level`, `eu_ai_act_risk`, `iso_42001_clause`, `confidence`, `affects`, `charter_id`, `plan_id`, … | OKF §4.1 says consumers preserve unknown keys → **emit the full StrayMark frontmatter verbatim**. Our richer governance metadata rides along losslessly as producer extensions. |

**Concept ID / bundle layout.** `.straymark/` is already a hierarchical tree, so
it maps to a bundle directly. Cleanest target layout groups by type:

```
straymark-bundle/
├── index.md                 # generated; okf_version: "0.1"
├── adr/        <ADR-…>.md
├── ailogs/     <AILOG-…>.md
├── charters/   <CHARTER-…>.md
├── tde/        <TDE-…>.md
└── audits/     <…>/review.md
```

## 3. Cross-link translation — where Loom's work pays off

This is the technically interesting part. OKF wants **bundle-relative markdown
links** (`/charters/CHARTER-02.md`). StrayMark expresses relationships as
**typed frontmatter IDs** (`related: [AILOG-2026-…-003]`, `supersedes:
ADR-…-001`). To export, we must resolve each ID to its file path and emit a
markdown link.

**`straymark-core` already does the hard half.** The Loom arc's reference
normalization (R1/R2, see the dev blog) resolves an edge target by exact id,
unique basename, path suffix, `CHARTER-NN` prefix, or dated id prefix, and
canonicalizes to a node id — and entity discovery already knows each node's file
path. So the exporter's link rewriting is *"ask the graph for the path, write a
markdown link."* The expensive machinery exists.

**Fidelity note (the one real loss).** OKF links are **untyped**; StrayMark
edges are **typed** (`SUPERSEDES` ≠ `RELATED_TO` ≠ `ORIGINATES_FROM`). On export
we either (a) flatten to plain links and let prose carry the type (OKF-idiomatic,
lossy for machine consumers), or (b) keep the typed relation in a producer
extension key (e.g. retain `supersedes:` in frontmatter) **and** emit a
convenience link. Recommended: **both** — emit the link for OKF graph viewers,
preserve the typed frontmatter for round-trip fidelity. Because OKF preserves
unknown keys, a StrayMark→OKF→StrayMark round-trip is **lossless** if we keep our
frontmatter intact.

## 4. Conformance assessment — how close is StrayMark today?

Measured against §9:

| Requirement | StrayMark today | Gap |
|---|---|---|
| Every non-reserved `.md` has parseable YAML frontmatter | ✅ all docs do | none |
| Every frontmatter has non-empty `type` | ⚠️ `DocType` is *inferred* (by path/heading), not always a literal `type:` key | add a `type:` mapping on export |
| `index.md` shape (if present) | ➖ not produced today | generate on export |
| `log.md` shape (if present) | ➖ `straymark audit` produces a timeline, not `log.md` | optional: render audit → `log.md` |
| Tolerant consumption (import side) | ✅ our parser already keeps dangling refs as `resolved:false` and tolerates unknown fields | aligns with OKF's permissive model |

**Conclusion:** StrayMark is **~80% OKF-conformant out of the box.** The corpus
is already markdown+frontmatter+graph with tolerant parsing. The export gap is
small and mechanical: (1) a `DocType → type` string map, (2) ID→path link
rewriting (graph already resolves it), (3) generated `index.md` files, (4)
optional `log.md` from the audit timeline, (5) packaging. See
[`05`](05-okf-exporter-design.md).

## 5. Import (OKF → StrayMark) — feasible, lower priority

A generic OKF bundle is *less* structured than StrayMark (untyped links, free
`type` values, no lifecycle). Importing means: read concepts as generic nodes,
map links to `RELATED_TO` edges, and surface them in Loom / `core::graph`. Useful
for **Loom as an OKF viewer** (§6) but it cannot synthesize governance semantics
that aren't in the source. Recommend export-first; import only to power Loom
viewing.

## 6. Loom as an OKF viewer

Loom ingests markdown+frontmatter and builds a typed graph; an OKF bundle is the
same input minus the typing. Pointing Loom at a bundle directory (treating links
as untyped `RELATED_TO` edges, `type` as the node type for coloring) is a small
adapter over existing code, and Loom's overlays/analytics/3D are strictly richer
than OKF's reference static-HTML visualizer. Low-effort, high-visibility:
*"Loom also renders any OKF bundle."*

## 7. Distance summary

| Capability | Effort | Reuses |
|---|---|---|
| `straymark export --format okf` (export) | **Small–medium** | `discover_documents`, `Frontmatter`, `core::graph` resolution, entity paths |
| Generated `index.md` per dir | Small | descriptions already in frontmatter |
| `log.md` from audit timeline | Small (optional) | `audit` engine |
| Loom reads OKF bundles | Small | Loom's existing graph builder + an untyped adapter |
| OKF import into StrayMark | Medium (optional) | parser tolerance already present |

Nothing here requires new infrastructure — it's serialization and a type map
over machinery the Loom arc already built.
