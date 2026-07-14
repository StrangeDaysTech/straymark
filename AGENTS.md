# StrayMark - Universal Agent Configuration

<!-- straymark:begin -->
> **Read and follow the rules in [STRAYMARK.md](STRAYMARK.md).**
> That file contains all StrayMark documentation governance rules for this project.
<!-- straymark:end -->

---

> **About this file.** `AGENTS.md` is the open standard for guiding AI coding agents,
> donated to the Agentic AI Foundation (Linux Foundation, 2025) and read by Claude Code,
> OpenAI Codex CLI, Cursor, Aider, Devin, Sourcegraph Amp, Google Jules, Zed AI, Continue,
> Roo Code, Factory Droids, GitHub Copilot, Gemini CLI, Windsurf, Amazon Q and others.
> StrayMark keeps this file in sync with `STRAYMARK.md` via `straymark update`.

## Autonomous Rules (minimum viable — works without STRAYMARK.md)

### Identity
- Identify yourself with your platform and version in the `agent:` field
  (e.g. `claude-code-v1.0`, `gemini-cli-v1.0`, `copilot-v1.0`, `cursor-v1.0`, `codex-cli-v1.0`)
- Declare confidence in decisions: `high | medium | low`

### Review Requirements
- ETH, ADR, SEC, MCARD, DPIA → always `review_required: true`
- `risk_level: high | critical` → always `review_required: true`

### Prohibited
- Never document credentials, tokens, API keys, or PII in document content

### Pre-commit Checklist

Before committing, check:
- [ ] Changed auth/PII/security code? → Create AILOG (`risk_level: high`) + ETH draft
- [ ] Complex code change? → Run `straymark analyze <changed-files> --output json`; if `above_threshold > 0` create AILOG (fallback if CLI unavailable: >20 lines)
- [ ] Chose between 2+ alternatives? → Create AIDEC
- [ ] Changed public API or DB schema? → Create AILOG + consider ADR
- [ ] Changed ML model/prompts? → Create AILOG + human review
- [ ] Changed OTel instrumentation? → Create AILOG + tag `observabilidad`
- [ ] Starting a multi-session implementation block (>1 day, >5 tasks, multi-phase)? → Declare a Charter via `straymark charter new` (see STRAYMARK.md §15)

### Regulatory Frontmatter Snippet

When creating documents for AI-related changes, include applicable fields:

```yaml
eu_ai_act_risk: not_applicable  # unacceptable | high | limited | minimal | not_applicable
nist_genai_risks: []            # privacy | bias | confabulation | cbrn | dangerous_content | environmental | human_ai_config | information_integrity | information_security | intellectual_property | obscene_content | value_chain
iso_42001_clause: []            # 4 | 5 | 6 | 7 | 8 | 9 | 10
```

### NIST AI 600-1 Risk Categories (quick reference)

1. CBRN Information — 2. Confabulation — 3. Dangerous Content — 4. Data Privacy — 5. Environmental — 6. Harmful Bias — 7. Human-AI Config — 8. Information Integrity — 9. Information Security — 10. Intellectual Property — 11. Obscene Content — 12. Value Chain

### Observability Rule

Do not capture PII, tokens, or secrets in OTel attributes or logs. Record instrumentation pipeline changes (new spans, changed attributes, Collector configuration) in AILOG with tag `observabilidad`.

### File Naming Convention

```
[TYPE]-[YYYY-MM-DD]-[NNN]-[description].md
```

Example: `AILOG-2026-05-14-001-add-feature.md`

---

*StrayMark | [Strange Days Tech](https://strangedays.tech) — Because every change tells a story.*
