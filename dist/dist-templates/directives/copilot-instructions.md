# StrayMark - GitHub Copilot Configuration

<!-- straymark:begin -->
> **Read and follow the rules in [../STRAYMARK.md](../STRAYMARK.md).**
> That file contains all StrayMark documentation governance rules for this project.
<!-- straymark:end -->

---

## StrayMark Rules for Copilot

When assisting with code changes in this project, follow these documentation rules:

**Document when:**
- Complex code change → suggest running `straymark analyze`; if `above_threshold > 0`, suggest AILOG (fallback: >20 lines)
- Choosing between alternatives → suggest creating AIDEC
- Changing auth/PII/security → suggest AILOG (risk_level: high) + ETH draft
- Changing public API or DB schema → suggest AILOG + consider ADR
- Changing ML models or prompts → suggest AILOG + human review
- Starting a multi-session implementation block (>1 day, >5 tasks, multi-phase) → suggest declaring a Charter (`straymark charter new`)

**Always:**
- Identify as `copilot-v{version}` in the `agent:` field
- Set `review_required: true` for ETH, ADR, SEC, MCARD, DPIA documents
- Set `review_required: true` when `risk_level: high | critical`
- Never include credentials, tokens, or PII in document content

**Regulatory fields** (include when relevant to AI changes):
- `eu_ai_act_risk`: unacceptable | high | limited | minimal | not_applicable
- `nist_genai_risks`: [privacy, bias, confabulation, ...]
- `iso_42001_clause`: [4-10]

**Observability:** Do not capture PII in OTel attributes. Tag instrumentation changes with `observabilidad`.

---

*StrayMark | [Strange Days Tech](https://strangedays.tech) — Because every change tells a story.*
