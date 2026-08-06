---
id: AILOG-2026-08-05-003
title: Qwen Code agent surface (QWEN.md + .qwen/skills) and the update path that would have withheld it
status: accepted
created: 2026-08-05
agent: claude-opus-5-v1.0
confidence: high
review_required: false
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 445
files_modified:
  - dist/dist-manifest.yml
  - dist/dist-templates/directives/QWEN.md
  - dist/dist-templates/directives/AGENTS.md
  - dist/.qwen/skills/
  - dist/STRAYMARK.md
  - dist/.straymark/00-governance/AGENT-RULES.md
  - cli/src/main.rs
  - cli/src/commands/install_skills.rs
  - cli/src/commands/update_framework.rs
  - cli/src/commands/remove.rs
  - cli/src/commands/validate.rs
  - cli/tests/qwen_skills_test.rs
  - cli/tests/inject_test.rs
  - cli/tests/architecture_skill_test.rs
  - .github/workflows/ci.yml
  - README.md
  - docs/adopters/CLI-REFERENCE.md
  - docs/adopters/ADOPTION-GUIDE.md
  - CHANGELOG.md
observability_scope: none
tags: [multi-agent, qwen-code, qoder, directive-injection, update-path, ci, i18n]
related:
  - AIDEC-2026-08-05-001
  - AILOG-2026-08-04-002
---

# AILOG: Qwen Code agent surface

## Summary

Qwen Code went from zero integration to a first-class surface — `QWEN.md`
directive injection, 15 skills at `.qwen/skills/`, `install-skills --agent
qwen`, `validate --agent qwen`, `remove` cleanup. Along the way, two defects
that the review surfaced were fixed: `update-framework` never created
newly-declared injection targets (so the new surface would have reached fresh
installations only), and the Qoder documentation asserted the opposite of what
the Qoder runtime does.

## Context

The question asked was narrow — *do the Qwen and Qoder CLIs get the same rules
and configuration Claude Code gets, at `init` and at later `update`?* — and the
answer split cleanly:

- **Qoder**: yes for skills (`dist/.qoder/skills/`, #399) and yes for rules,
  via `AGENTS.md`. Verified in the shipped application: it file-searches
  `**/AGENTS.md` and adds each match as an instruction source.
- **Qwen Code**: no, on both counts. Nothing referenced it anywhere in the
  product; the only occurrences in the repo were prose inside the audit skills
  warning auditors not to write `qwen-code` in the `auditor:` field.

The severity of the Qwen gap depended on one empirical question — does Qwen
Code read `AGENTS.md`? — so it was answered against the installed runtime
rather than from memory. Its context-filename resolver ends in
`["QWEN.md"] : Array.isArray(configured) ? configured : [configured]`: the
default is `QWEN.md` alone. `AGENTS.md` appears exactly once in the bundle, in
a system-prompt string enumerating files the agent must not modify without
permission — recognized, never loaded. So Qwen Code users were receiving no
governance at all, which is a strictly worse position than Codex or Qoder.

Two further runtime facts settled the design (see [AIDEC-2026-08-05-001] for
the alternatives): `Storage.getGlobalQwenDir()` resolves `$QWEN_HOME` →
`$HOME/.qwen`, and skills resolve at **both** project and user scope with
`allowed-tools` supported. The channel is therefore a byte-for-byte mirror of
`.claude/`, exactly as Qoder's is.

## Changes

**Framework (`dist/`)**

- `dist/.qwen/skills/` — 15 skills, byte-identical to `dist/.claude/skills/`.
- `dist/dist-templates/directives/QWEN.md` — cloned from the `GEMINI.md`
  template (Qwen Code is a Gemini CLI fork) with the `qwen-code-v{version}`
  identity.
- `dist-manifest.yml` — `.qwen/skills/` in `files:`, `QWEN.md` in `injections:`.
- `AGENT-RULES.md` (EN/es/zh-CN) and the `AGENTS.md` template gained
  `qwen-code-v1.0` **and** `qoder-v1.0`; the latter had been missing since #399.
- `STRAYMARK.md` § "Directive Injection Markers" lists `QWEN.md`.

**CLI**

- `install_skills.rs`: `--agent qwen` branch + `resolve_qwen_home()` mirroring
  the runtime's own resolution. The doc-comment now states which agents
  *require* the user-level install (only Codex) and which merely benefit.
- `validate.rs`: `validate_codex_skills()` generalized to
  `validate_agent_skills(agent)` behind an `AgentSkillsSpec`. The `claude-only-key`
  check is now conditional — Qoder and Qwen legitimately carry `allowed-tools`,
  so flagging it there would have been a false positive by construction.
- `remove.rs`: `.qwen/skills`, the `.qwen` parent, and `QWEN.md` in
  `LEGACY_DIRECTIVE_TARGETS`.
- `update_framework.rs`: dropped the `if !target_path.exists() { continue; }`
  guard — see §Risk R1.

**CI** — new `skill-mirror-parity` job. The `.qoder` mirror had a parity test
since #399, but the pipeline runs no `cargo test`, so the only gate in CI was
`.codex`'s generator check. The job runs exactly the two mirror suites, which
keeps it narrow enough not to reopen the deferred "no full test suite in CI"
decision.

**Docs (EN/es/zh-CN)** — Qwen added to README (agent list, architecture tree,
directory table, platform table), CLI-REFERENCE (`install-skills`,
`validate --agent`, "5 parallel forms" → 6) and ADOPTION-GUIDE. The i18n
READMEs also gained Qoder, which #399 had updated only in English.

## Verification

- `cargo test`: 949 passed, 0 failed.
- New tests: `qwen_skills_test.rs` (tree parity, `install-skills` e2e under
  `QWEN_HOME`, manifest surface, `validate --agent` acceptance);
  `update_framework::tests::update_creates_injection_targets_that_are_missing_on_disk`;
  `inject_test::test_manifest_declares_every_directive_injection` (was
  `AGENTS.md`-only, now all seven targets plus template-on-disk).
- Manual e2e against a simulated post-`init` tree: `install-skills --agent qwen`
  installed 15 skills into a temp `$QWEN_HOME`; `validate --agent qwen` reported
  all 15 passing with no `allowed-tools` false positives; `validate --agent
  qoder` degraded to its install hint on an empty directory.

`straymark init` downloads the published release ZIP, so it cannot exercise a
local `dist/`. The `init`-level check of `QWEN.md` is therefore deferred to
after the `fw-4.41.0` tag lands.

## Risk

| Id | Risk | Handling |
|----|------|----------|
| R1 | `update-framework` creating missing targets means a directive file the operator deleted on purpose is now restored on the next update. | Accepted: `STRAYMARK.md` § "Directive Injection Markers" already documents exactly this ("no opt-out per target short of editing the manifest"), and `repair` has always behaved this way. The code was the outlier, not the doc. If adopters turn out to delete these deliberately, the fix is a per-target opt-out in `config.yml`. |
| R2 (new, not in Charter) | The manual e2e caught that `validate --agent`'s clap `value_parser` is declared in `main.rs`, separate from the dispatch in `validate.rs` — widening only the latter left the flag rejected at parse time. | Fixed, and pinned by `validate_accepts_every_user_level_agent`. Worth remembering as a shape: this CLI keeps its accepted-value lists at a distance from the code that consumes them, so `install-skills` and `validate` each need both ends touched. |
| R3 (new, not in Charter) | Six directive files now land in an adopter's project root at `init`, and the count grows with every supported CLI. | Not addressed here. Noted as a real ergonomics cost of the per-CLI directive model; the alternative (AGENTS.md only) is not available while runtimes like Qwen Code default to their own filename. |

## Follow-ups

- Verify `QWEN.md` and `.qwen/skills/` land through a real `straymark init` and
  a real `straymark update` once `fw-4.41.0` is published, and confirm in the
  live `qwen` and `qoder` CLIs that the context file and skills are picked up.
