# OKF vs StrayMark — strategic analysis

*Prepared 2026-06-17. Sources: Google Cloud OKF announcement (2026-06-12), OKF
`SPEC.md` v0.1, the Karpathy gist (see [`02`](02-karpathy-llm-wiki-genesis.md)).*

## 1. One-line verdict

OKF and StrayMark share the **same mechanics** — the "LLM-wiki pattern":
markdown + YAML frontmatter + a graph of links + an agent that maintains the
corpus — which StrayMark reached **independently** (OKF credits Karpathy; we
didn't get here from him). But they obey **opposite paradigms of AI use**: OKF
builds cognition *for the agent* — lifting the burden of knowing the system off
people, in line with the dominant long-running-autonomous-agent trend — while
StrayMark builds cognition *for the agent and the human engineer together*,
keeping a person oriented and in command. The domains differ too (OKF describes
*the data and systems agents consume*; StrayMark records *the governance of what
agents do*), but that's downstream of the deeper split: **who the knowledge is
for.** Not a frontal competitor; a strong validation of the design bet and a real
interoperability opportunity. The risk is **mindshare capture**, not
replacement.

## 2. What OKF is

OKF v0.1 (Google Cloud, Apache 2.0) is *"an open specification that formalizes
the LLM-wiki pattern into a portable, interoperable format … a vendor-neutral,
agent- and human-friendly standard for representing the metadata, context, and
curated knowledge that modern AI systems need."*

Concretely: a **directory of markdown files with YAML frontmatter**, one file
per *concept* (a table, a metric, an API, a playbook), exactly **one required
field** (`type`), cross-links as plain markdown links, optional `index.md`
(progressive disclosure) and `log.md` (history). The full spec fits on a page.
Reference implementations: an enrichment agent that walks BigQuery and drafts
OKF docs, a static HTML graph visualizer, sample bundles, and Google Cloud's
Knowledge Catalog updated to ingest OKF.

## 3. The convergence is real and deep

Google arrived — independently — at almost every design decision StrayMark made:

| Design decision | OKF v0.1 | StrayMark |
|---|---|---|
| Knowledge unit | `.md` file + YAML frontmatter | AILOG / ADR / Charter / TDE = `.md` + frontmatter |
| One required field | `type` | `DocType` |
| Relationship graph | markdown links between concepts | `related` / `supersedes` / `originating_ailogs` → `core::graph` |
| Graph visualizer | static client-side HTML | Loom (Sigma.js / maxGraph / Three.js) |
| "Seed" generator | enrichment agent: BigQuery → docs | `architecture generate`: code → model |
| Maintainer | AI agent updates the docs | the whole agent-directive framework |
| Distribution | "just files": tarball / git / vendor-neutral | same; `dist/` + crates.io |
| Spec | versioned, backward-compatible, one page | versioned (`fw-`/`cli-`), independent spec |
| Reserved files | `index.md`, `log.md` | equivalent conventions; `audit` timeline ≈ log |
| Tolerant consumption | broken links are "not-yet-written knowledge", unknown keys preserved | dangling refs kept as first-class `resolved:false` edges |

That Google converged on markdown+frontmatter+graph+agent is the strongest
possible evidence the bet was right — *because the two projects started from
different places.* OKF cites Karpathy and the "LLM-wiki pattern" as its genesis;
StrayMark reached the same intuition on its own, without that lineage. Independent
arrival at the same primitives is what makes the convergence evidence rather than
imitation.

## 4. The decisive difference: who the knowledge is for

This is where they stop touching.

The split isn't only domain; it's **who each format's knowledge is ultimately
for.** OKF encodes a system so an *agent* can understand and operate it with as
little human involvement as possible — the autonomous-agent direction the industry
is racing toward. StrayMark encodes how a system was built so a *human engineer*
stays oriented and in command alongside the agent — the "AI-augmented engineering"
position, where the human is repositioned into judgment and direction, not
displaced. One faces the agent; the other faces the agent **and** the human at
once. The domain difference below is a consequence of that, not the root of it:

- **OKF = a semantic / context layer over data.** It encodes *what this table
  means, what this metric is, the join paths, the runbook* so an AI agent can
  reason over an organization's data estate (BigQuery, Knowledge Catalog). It is
  in effect a **portable data-catalog / context format for RAG and agents**.
  Deliberately *minimally opinionated*: it requires only `type` and states that
  *"everything else is left to the producer."*

- **StrayMark = governance and traceability of the AI-assisted development
  process.** It encodes *what decision was made and why (ADR), what the agent
  did (AILOG), what was planned (Charter), what debt remains (TDE), what each
  model audited, what the EU AI Act / ISO 42001 / NIST require.* It is **highly
  opinionated**: lifecycle, policies, drift, compliance.

Put differently: **OKF describes the system so AI can understand it; StrayMark
describes how the system was built so a human (or auditor) can trust it.** A
single team could run both at once with no overlap — OKF for its data semantic
layer, StrayMark to govern its development.

Critically: the OKF announcement and `SPEC.md` never mention governance,
compliance, audit, or decision traceability. **That axis — StrayMark's core —
is clear.** And StrayMark fills precisely the space OKF leaves open: governance
*is* the "everything else … left to the producer."

## 5. Competitor? — nuanced

- **Not frontal.** OKF does not do what StrayMark does, and StrayMark does not
  aim to be a BigQuery data catalog.
- **Yes, it competes for mindshare** of the LLM-wiki pattern. With Google's
  distribution and BigQuery integration, OKF can become the generic *lingua
  franca*, and someone may ask "why not just OKF?". The answer is the governance
  domain — but it must be **articulated well** (see §7).
- **Convergence risk:** OKF is deliberately minimal. If someone builds a
  governance layer *on top of* OKF, that would graze StrayMark. Unlikely
  near-term (Google is focused on data), but worth watching.
- **Resource asymmetry:** Google has mass distribution; StrayMark is
  independent. Don't fight to be the generic standard — **be the domain
  specialist OKF doesn't touch.**

## 6. SWOT

**Strengths (StrayMark)** — mature, opinionated governance domain; typed,
bidirectional graph (richer than OKF's untyped links); full lifecycle (drift,
audit, compliance, follow-ups); Loom is a richer visualizer than OKF's static
HTML; already shipping (`fw-4.28` / `cli-3.28` / `loom-0.6.2`).

**Weaknesses** — independent project vs. Google's reach; no standards-body
imprimatur; smaller community; "governance" is a harder sell than "make your
data agent-ready."

**Opportunities** — emit/consume OKF (small distance, see [`03`](03-okf-technical-report.md));
Loom as an OKF viewer; ride the OKF news cycle as design validation; sharpen
positioning against the generic pattern.

**Threats** — mindshare capture; "why not just OKF?"; a future governance layer
built over OKF; Google bundling more of the workflow.

## 7. How to exploit it

1. **Interoperability — export/import OKF.** Both are markdown+frontmatter+graph,
   so the distance is small. `straymark-core` already has the document model,
   typed graph, and — thanks to Loom's R1/R2 work — reference→path resolution,
   which is exactly what OKF bundle-relative links need. Position StrayMark as
   *OKF-compatible*: hook into Google's standard for distribution/visualization
   without losing identity. See [`05`](05-okf-exporter-design.md).
2. **Loom as an OKF viewer.** Loom already renders markdown+frontmatter graphs;
   an OKF bundle is nearly the same input, and Loom is richer than OKF's static
   HTML (status overlays, 3D BIM, graph analytics). "Loom: also an OKF viewer"
   is a real adoption hook.
3. **Strategic validation for the blog.** A strong post: *Google just published
   our design thesis.* Convergence as validation, then crisp differentiation. See
   [`04`](04-blog-post-draft.md).
4. **Positioning line:** *"OKF is for the knowledge agents consume; StrayMark is
   the governance record of what agents do."* Inoculates against "why not OKF?".
5. **Adopt convergent conventions** where sensible (the `type` field, `index.md`
   / `log.md`) to minimize future interop friction.

## 8. Recommendation

Treat OKF as **tailwind, not threat.** Immediate, low-cost/high-return: a blog
post that (a) celebrates the convergence as validation and (b) nails the domain
differentiation. Medium-term, optional but strategic: a `straymark export
--format okf` prototype over `straymark-core` plus Loom-as-OKF-viewer — making
StrayMark interoperable with Google's ecosystem while keeping its governance
edge. The one thing to watch actively: anyone building governance/compliance on
top of OKF. While Google keeps it a data layer, the domains don't collide.
