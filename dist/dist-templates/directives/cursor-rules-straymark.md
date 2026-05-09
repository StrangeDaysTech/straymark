# StrayMark - Cursor Rules

<!-- straymark:begin -->
<!-- straymark:end -->

## StrayMark Documentation Rules

Identity: Use `cursor-v{version}` in the `agent:` field.

Document when: complex code — run `straymark analyze <files> --output json`, AILOG if `above_threshold > 0` (fallback: >20 lines), alternatives (AIDEC), auth/PII (AILOG+ETH), API/DB changes (AILOG+ADR), ML/prompts (AILOG+review).

Review required: ETH, ADR, SEC, MCARD, DPIA → always. risk_level high/critical → always.

Never document: credentials, tokens, API keys, PII.

Regulatory fields (when relevant): eu_ai_act_risk, nist_genai_risks, iso_42001_clause.

Observability: No PII in OTel attributes. Tag instrumentation changes with `observabilidad`.

Naming: [TYPE]-[YYYY-MM-DD]-[NNN]-[description].md
