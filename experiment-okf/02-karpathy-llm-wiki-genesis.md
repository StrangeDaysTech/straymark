# The Karpathy gist — genesis of the LLM-wiki pattern, and what it means for StrayMark

*Prepared 2026-06-17. Source: Andrej Karpathy, gist `442a6bf555914893e9891c11519de94f`
(the pattern OKF's announcement explicitly credits as its origin), plus the
community replies on that gist.*

This is the "extra analysis step" requested: the gist is the intellectual
headwater both OKF and StrayMark drink from, and its comment thread already
mapped out the failure modes — several of which StrayMark answers by design.

## 1. The proposal

Karpathy proposes that instead of classic RAG (retrieve from raw docs on
demand), an **LLM incrementally maintains a persistent wiki** — *"a persistent,
compounding artifact"* that grows richer with each source ingested and each
query asked. Three layers:

1. **Raw sources** — immutable, curated, read but never modified.
2. **The wiki** — LLM-generated markdown: summaries, entity pages, concept
   pages, cross-references, maintained by the AI.
3. **The schema (`CLAUDE.md`)** — configuration: wiki structure, conventions,
   workflows.

The human curates sources and asks questions; the LLM does *"all the grunt
work — the summarizing, cross-referencing, filing, and bookkeeping."* Three
operations: **Ingest** (a new source updates 10–15 pages at once, flagging
contradictions), **Query** (answers can themselves become new wiki pages, so
*"explorations compound"*), and **Lint** (periodic health-check for
contradictions, stale claims, orphans, gaps — the LLM *suggests* investigations
rather than auto-resolving). The rationale: humans find wiki upkeep tedious;
*"LLMs don't get bored, don't forget to update a cross-reference."* He grounds
it in Vannevar Bush's 1945 Memex — associative trails that finally have a tireless
maintainer.

## 2. Why this is StrayMark's DNA too

StrayMark is, structurally, an instance of this pattern aimed at one domain —
software governance:

| Karpathy's pattern | StrayMark |
|---|---|
| Raw sources (immutable) | the codebase + PRs + commits (the ground truth AILOGs reference) |
| The wiki (LLM-maintained markdown) | `.straymark/` corpus: AILOGs, ADRs, Charters, TDEs, audits |
| The schema (`CLAUDE.md`) | `STRAYMARK.md` + `AGENT-RULES.md` + governance policies |
| Ingest (one change → many pages) | an agent closing work writes the AILOG, updates Charter rows, extracts follow-ups |
| Query → new pages | audits and analyses become first-class documents in the corpus |
| Lint (contradictions/stale/orphans/gaps) | `validate`, `charter drift`, follow-ups drift, dangling-ref classification |
| "LLMs don't forget cross-references" | typed frontmatter links resolved by `core::graph` |

So OKF and StrayMark are **siblings, not rivals** — two specializations of the
same parent idea. OKF specializes it for *data context*; StrayMark for
*development governance*.

## 3. The community replies already mapped the failure modes — and StrayMark answers several

The gist's thread is unusually substantive. The critiques matter because they
are the known weak points of the raw pattern, and they double as a checklist of
where StrayMark's opinionation earns its keep.

- **pursultani — "convergent epistemology" / typed edges.** Warned the pattern
  defaults to *reconciling* contradictions, and that in many domains a
  contradiction *carries information* and should be preserved, not resolved.
  Proposed **typed edges in YAML frontmatter** to hold tensions.
  → **StrayMark already does this.** `supersedes`, `alternatives_documented`,
  and the whole **TDE** (Transversal Debt Entry) concept exist precisely to
  *record* an unresolved tension rather than flatten it. ADRs are immutable once
  accepted and *superseded*, never edited — contradiction is preserved as
  history. This is StrayMark's single strongest differentiator against both the
  raw pattern *and* OKF, whose links are explicitly **untyped** (relationship
  conveyed by prose, not the link). **Headline for the blog.**

- **Archimondstat — hallucination accumulation.** Worried LLM-written wikis
  compound errors; proposed a "refine before promoting" gate.
  → StrayMark's analogue is the **human-gated lifecycle**: ADRs require human
  review (`review_required`), follow-ups are *captured* but promotion to a TDE
  is operator-gated, and the **multi-model independent audit** cycle exists to
  catch exactly this. Knowledge doesn't auto-promote; it's reviewed.

- **watsonrm — multi-writer scaling / idempotent, commutative writes.** Argued
  branch-and-merge alone doesn't stop semantic duplicates.
  → StrayMark's CLI-owned counters, content-hash dedup in follow-ups drift, and
  surgical (non-reserializing) writes are early moves toward idempotent,
  conflict-resistant updates. Not solved, but the problem is named and the
  architecture leans the right way.

- **witwaycorp — "why not just a database?"** Once indexed, why markdown?
  → The same answer StrayMark and OKF both give: **diffability, human
  readability without tooling, portability, and version control as the
  substrate.** Governance artifacts must be auditable by humans and survive tool
  churn — a database row can't be reviewed in a PR.

- **Implementations** (Dense-Mem, AutoSci, Synthadoc, **memwiki**). The most
  on-point is **memwiki** — explicitly built for coding projects to fight
  *"Agent Amnesia,"* where agents forget architectural decisions. That is
  **StrayMark's exact founding problem**, independently re-derived by a
  community member. Strong external corroboration of the thesis.

## 4. Takeaways

1. **Convergence at three levels.** The same idea produced (a) Karpathy's
   pattern, (b) Google's OKF spec, (c) community implementations (memwiki), and
   (d) StrayMark — independently. The pattern is no longer a differentiator; the
   **domain and the opinionation** are.
2. **StrayMark answers the gist's hardest critique.** Typed edges that preserve
   contradiction (pursultani) are designed-in via `supersedes`/`alternatives`/TDE
   — exactly where OKF chose untyped links. This is the sharpest line to draw.
3. **"Agent Amnesia" is the marketable framing.** memwiki names StrayMark's
   problem in three words. Worth borrowing for positioning.
4. **The lint operation = StrayMark's validate/drift suite.** Karpathy's "lint"
   is conceptually our `validate` + `charter drift` + follow-ups drift +
   dangling-ref classification. We've productized what he sketched.

These threads feed the blog post ([`04`](04-blog-post-draft.md)) and reinforce
the positioning in [`01`](01-okf-vs-straymark-analysis.md).
