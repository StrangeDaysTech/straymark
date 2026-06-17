# experiment-okf

An exploration triggered by Google Cloud's announcement of the **Open Knowledge
Format (OKF)** v0.1 (2026-06-12) and the Andrej Karpathy gist that seeded it.
The question: how related is OKF to StrayMark, is it a competitor, and is it
something we can exploit?

**Short answer:** OKF and StrayMark share the same *mechanics* (the "LLM-wiki
pattern": markdown + YAML frontmatter + a link graph + an agent that maintains
the docs) but live in **orthogonal domains** — OKF describes *the data agents
consume*; StrayMark records *the governance of what agents do*. OKF is not a
frontal competitor. It is a strong validation of StrayMark's design bet and a
concrete interoperability opportunity. The real risk is mindshare capture, not
replacement.

## Contents

| Doc | What it is |
|---|---|
| [`01-okf-vs-straymark-analysis.md`](01-okf-vs-straymark-analysis.md) | Strategic analysis: relationship, competitor/opportunity, SWOT, positioning. |
| [`02-karpathy-llm-wiki-genesis.md`](02-karpathy-llm-wiki-genesis.md) | Analysis of the Karpathy gist + community replies, and how each maps to StrayMark. |
| [`03-okf-technical-report.md`](03-okf-technical-report.md) | Read of the real OKF v0.1 `SPEC.md`; field-by-field OKF↔StrayMark mapping; export/import distance; conformance assessment. |
| [`04-blog-post-draft.md`](04-blog-post-draft.md) | Draft dev-blog post (series voice): convergence-as-validation + clean domain differentiation. |
| [`05-okf-exporter-design.md`](05-okf-exporter-design.md) | Design sketch for `straymark export --format okf` over `straymark-core`, plus Loom-as-OKF-viewer. |

## Sources

- Google Cloud blog — *How the Open Knowledge Format can improve data sharing*
  (2026-06-12): <https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing>
- OKF repo (Apache 2.0): <https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf> — `SPEC.md` v0.1 (Draft).
- Karpathy gist (LLM-wiki pattern): <https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f>

## Status

EXPLORATORY. Nothing here ships yet. `01`–`03` are reports; `04` is a draft for
review; `05` is a design proposal, not a committed feature. Prepared 2026-06-17.
