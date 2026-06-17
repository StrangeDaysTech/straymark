# experiment-okf

An exploration triggered by Google Cloud's announcement of the **Open Knowledge
Format (OKF)** v0.1 (2026-06-12) and the Andrej Karpathy gist OKF credits as its
origin. The question: how related is OKF to StrayMark, is it a competitor, and is
it something we can exploit?

**Short answer:** OKF and StrayMark share the same *mechanics* (the "LLM-wiki
pattern": markdown + YAML frontmatter + a link graph + an agent that maintains
the docs) — StrayMark reached them **independently**; OKF credits Karpathy. But
they obey **opposite paradigms of AI use**: OKF builds cognition *for the agent*
(lifting the burden of knowing the system off people, in line with the
long-running-autonomous-agent trend); StrayMark builds cognition *for the agent
and the human engineer together*, keeping a person oriented and in command. The
domain differs too — OKF describes *the data agents consume*, StrayMark records
*the governance of what agents do* — but the deeper split is **who the knowledge
is for**. OKF is not a frontal competitor; it is a strong validation of
StrayMark's design bet and a concrete interoperability opportunity. The real risk
is mindshare capture, not replacement.

## Contents

| Doc | What it is |
|---|---|
| [`01-okf-vs-straymark-analysis.md`](01-okf-vs-straymark-analysis.md) | Strategic analysis: relationship, competitor/opportunity, SWOT, positioning. |
| [`02-karpathy-llm-wiki-genesis.md`](02-karpathy-llm-wiki-genesis.md) | Analysis of the Karpathy gist (OKF's lineage) + community replies, and how StrayMark independently answers the failure modes they name. |
| [`03-okf-technical-report.md`](03-okf-technical-report.md) | Read of the real OKF v0.1 `SPEC.md`; field-by-field OKF↔StrayMark mapping; export/import distance; conformance assessment. |
| [`04-blog-post-draft.md`](04-blog-post-draft.md) | Draft dev-blog post (series voice): convergence-as-validation + clean domain differentiation. |
| [`05-okf-exporter-design.md`](05-okf-exporter-design.md) | Design sketch for `straymark export --format okf` over `straymark-core`, plus Loom-as-OKF-viewer. |

## Sources

- Google Cloud blog — *How the Open Knowledge Format can improve data sharing*
  (2026-06-12): <https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing>
- OKF repo (Apache 2.0): <https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf> — `SPEC.md` v0.1 (Draft).
- Karpathy gist (LLM-wiki pattern): <https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f>

## Status

EXPLORATORY. `01`–`03` are reports; `04` **shipped** as the dev-blog post
*"What the open format left to the producer"* (EN/ES/zh-CN, published 2026-06-17);
`05` is a design proposal, not a committed feature. Prepared 2026-06-17.
