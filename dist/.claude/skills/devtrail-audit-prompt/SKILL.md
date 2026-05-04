---
name: devtrail-audit-prompt
description: Prepare external multi-model audit for a Charter. Generates auditor prompts inline so the operator can paste them into 2 LLM auditors of different families.
allowed-tools: Read, Bash(devtrail charter audit *, devtrail charter status *, ls *)
---

# DevTrail Audit Prompt Skill

Generate prompts for external multi-model audit of a Charter, surfaced inline in the conversation so the operator can paste them into auditors of two different model families without leaving the chat.

## When to invoke

Use this skill when the developer agreed to run an external audit at the Charter checkpoint (see `.devtrail/00-governance/AGENT-RULES.md` § Audit checkpoint, available from `fw-4.8.0`).

The Charter should be in `in-progress` or `declared` status — auditing closed Charters is allowed but atypical (warn the operator and proceed only on confirmation).

## Instructions

### 1. Resolve the Charter

Argument: a Charter identifier (`CHARTER-04`, `04`, or the full id with slug).

```bash
devtrail charter status <CHARTER-ID>
```

Verify the Charter exists and capture its `status`. If `status: closed`, surface a one-line warning to the operator and ask whether to proceed.

### 2. Generate the auditor prompts (PREPARE step)

```bash
devtrail charter audit <CHARTER-ID>
```

The CLI writes the resolved prompts to disk:

- `audit/charters/<CHARTER-ID>/prompts/auditor-primary.prompt.md`
- `audit/charters/<CHARTER-ID>/prompts/auditor-secondary.prompt.md`

This is `devtrail charter audit` step 1 (PREPARE). The CLI does NOT invoke any LLM — it only resolves placeholders against the Charter content, the git diff, and the originating AILOGs.

### 3. Surface the prompts inline

Read both files and print their contents in the conversation, with clear separators:

```
═══════════════════ AUDITOR PRIMARY PROMPT ═══════════════════
[full contents of auditor-primary.prompt.md]
══════════════════════════════════════════════════════════════

═══════════════════ AUDITOR SECONDARY PROMPT ═════════════════
[full contents of auditor-secondary.prompt.md]
══════════════════════════════════════════════════════════════
```

Do not summarise or truncate. The operator needs the full prompts to paste into the external auditors as-is.

### 4. Provide next-steps guidance

After surfacing the prompts, print this guidance verbatim (substituting `<CHARTER-ID>`):

```
Next steps:

  1. Run AUDITOR PRIMARY PROMPT in a model of family A
     (e.g., Anthropic — claude-sonnet-4-6 or claude-opus-4-7).

  2. Run AUDITOR SECONDARY PROMPT in a model of family B
     (e.g., Google — gemini-2.5-pro, or OpenAI — gpt-4o / Copilot).
     DO NOT use the same family for both. Heterogeneity inter-family
     is what makes convergent findings high-signal — same-family
     auditors share blind spots.

  3. Save the auditor responses to canonical paths:
       audit/charters/<CHARTER-ID>/auditor-primary.md
       audit/charters/<CHARTER-ID>/auditor-secondary.md

  4. Return with: /devtrail-audit-review <CHARTER-ID>
     I will calibrate the responses, generate the calibrator analysis
     locally, and merge the findings into the Charter telemetry.
```

## Output schema for auditor responses

The `auditor-primary.md` and `auditor-secondary.md` files the operator saves must follow `audit-output.schema.v0.json` (validated by `devtrail charter audit --calibrate`). The required frontmatter is documented inside each prompt — the operator should preserve it when pasting the LLM response.

## Notes

- This skill is **orchestration-only**. It does NOT invoke LLM APIs, decide which models the operator uses, or wait for responses. It surfaces prompts and exits.
- Re-running the skill on the same Charter regenerates the prompts (idempotent at the prompt level). It does NOT overwrite operator-saved responses in `auditor-primary.md` / `auditor-secondary.md`.
- Heterogeneity inter-family is recommended but not enforced in v0. The operator decides the model pairing; the skill surfaces the recommendation in the next-steps guidance.
- For the rationale on why dual auditors of different families produce calibration that mono-auditor cannot, see `dist/.devtrail/audit-prompts/auditor-primary.md` § heterogeneity (or `Propuesta/devtrail-cli-roadmap.md` §5.2 in the upstream repo).
