---
id: AILOG-2026-08-05-004
title: Antigravity CLI replaces Gemini CLI — two dead channels retired, GEMINI.md kept, and a retirement mechanism built to make it land
status: accepted
created: 2026-08-05
agent: claude-opus-5-v1.0
confidence: high
review_required: false
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 695
files_modified:
  - dist/dist-manifest.yml
  - dist/.agent/skills/
  - dist/dist-templates/directives/GEMINI.md
  - dist/.straymark/00-governance/AGENT-RULES.md
  - dist/.claude/skills/
  - cli/src/manifest.rs
  - cli/src/commands/update_framework.rs
  - cli/src/commands/repair.rs
  - cli/src/commands/install_skills.rs
  - cli/src/commands/charter/audit.rs
  - cli/src/main.rs
  - cli/src/bin/gen_minimal_skills.rs
  - cli/tests/retired_paths_test.rs
  - cli/tests/architecture_skill_test.rs
  - cli/tests/audit_skill_test.rs
  - .github/workflows/ci.yml
  - README.md
  - docs/adopters/
  - website/
  - CHANGELOG.md
observability_scope: none
tags: [multi-agent, antigravity, gemini-cli, deprecation, update-path, ci, i18n]
related:
  - AIDEC-2026-08-05-002
  - AILOG-2026-08-05-003
---

# AILOG: Antigravity CLI replaces Gemini CLI

## Summary

Gemini CLI was retired by Google in favour of Antigravity CLI (`agy`). Two of
StrayMark's Gemini-era surfaces turned out to be dead and are retired;
`GEMINI.md` turned out to be alive and is kept, re-framed. Along the way the
framework gained the piece that makes retiring a distributed path possible at
all — without it, "discontinuing" a channel would have meant new installations
stop getting it while every existing one keeps it forever.

## Context

The task looked like a straightforward deprecation. Checking the runtime first
changed the shape of it.

`agy` 1.1.10 is installed on this machine, and Google ships the authoritative
spec of its customization system *inside the binary* as a builtin skill
(`agy-customizations`). Reading it:

- **`GEMINI.md` is a first-class Rules file**, listed alongside `AGENTS.md` and
  `.agents/rules/*.md`. `~/.gemini/` is Antigravity's own global config
  directory (the binary carries a `resolve_gemini_dir` symbol). The product was
  retired; the on-disk contract was kept. **Deleting `GEMINI.md` would have
  removed a rules source from the very CLI we were adding support for.**
- **`<project>/.gemini/skills/` is not a discovery location.** Workspace roots
  are `.agents/`, `.agent/`, `_agents/`, `_agent/`; the global one is
  `~/.gemini/config/`.
- **`<project>/.agent/workflows/` is read by nothing.** The root is valid but
  the subdirectory Antigravity looks for is `skills/<name>/SKILL.md`; the binary
  contains no `agent/workflows` string. That channel had been advertised in the
  README as the Antigravity surface since before this release.

Which left the retirement problem: `update_files` only copies. See §Changes.

## Changes

**Framework**

- `dist/.agent/skills/` — 15 skills, `skills/<name>/SKILL.md`, minimal
  frontmatter. Discovered from the project tree; no install step.
- `dist/.gemini/skills/` and `dist/.agent/workflows/` removed from `files:` and
  declared under the new `retired:` key.
- `GEMINI.md` directive retitled to the Antigravity lineage, identity
  `antigravity-v{version}`, plus a note explaining why the filename stays —
  without it the next reader deletes the file as a leftover.
- Auditor-CLI prose moved to `agy` across the skills, governance docs and
  templates. **Gemini model ids were deliberately left alone** (see §Decisions).

**CLI**

- `manifest.rs`: `retired: Vec<String>` with `#[serde(default)]` — load-bearing,
  since every pre-4.42.0 `.straymark/dist-manifest.yml` lacks the key and
  `repair`/`remove` re-read those copies.
- `update_framework.rs`: `prune_retired()`, provenance-gated. A file is deleted
  only when its hash still matches `.checksums.json`. Operator-edited files and
  files StrayMark never installed are kept and reported.
- `repair.rs`: same sweep, plus retired leftovers now count toward
  `total_issues` — otherwise the early return on a healthy installation would
  have skipped the sweep exactly where it was needed.
- `install-skills --agent gemini` → `--agent agy`.
- `gen_codex_skills` → `gen_minimal_skills`, emitting `.codex/skills/` **and**
  `.agent/skills/` under the existing CI gate.

**Docs/website** — README, CLI-REFERENCE, ADOPTION-GUIDE, WORKFLOWS,
TRANSLATION-GUIDE, React components, features and `code.json` (EN/es/zh-CN).
"6 parallel forms" → 5. The zh-CN CLI-REFERENCE, two releases behind, resynced.

## Decisions

Recorded in full in [AIDEC-2026-08-05-002]. Two worth restating here because
they are the ones a reader will second-guess:

**Why `GEMINI.md` survives a "discontinue Gemini CLI" task.** Because the
runtime reads it. The name is stale; the file is not.

**Why Gemini *model* ids were not renamed.** The CLI product was retired; the
models were not, and `agy` serves them. The audit skills already state that
`auditor:` must name the backend model rather than the CLI, so `gemini-3-pro`
stays valid while `gemini-cli` as "a CLI to open" does not. Historical records —
Sentinel telemetry, the Charter-template rule justified by an observed
Gemini-auditor behavior across 2 cycles — were left exactly as written.
Rewriting the actor in a piece of evidence would falsify the evidence.

## Verification

- `cargo test`: **953 passed, 0 failed**.
- New: `retired_paths_test.rs` (the shipped manifest actually retires the dead
  channels and does not distribute them simultaneously) plus four unit tests in
  `update_framework::tests` covering the three provenance outcomes, full-subtree
  cleanup, the no-op case, and the pre-4.42.0 manifest parse.
- Manual e2e on a simulated fw-4.41.0 installation: two pristine files pruned,
  one operator-edited file kept and flagged as modified, one operator-authored
  file kept and flagged as *not installed by StrayMark*, parent directories
  cleaned only when empty.

The e2e caught two real defects before they shipped: `repair`'s early return
skipped the sweep on a healthy installation, and the first version of the report
told the operator they had "modified" files they had authored themselves. A unit
test also caught that only the retired root was being cleaned, leaving emptied
skill subdirectories behind.

## Risk

| Id | Risk | Handling |
|----|------|----------|
| R1 | ~~The `.agent/` vs `.agents/` root is not empirically confirmed.~~ **Closed 2026-08-06.** | Three headless `agy -p` probes could not settle it — workspace customizations did not surface in print mode at all, even in a trusted directory with a git root, so the probe could not distinguish the roots rather than showing one failing. The operator ran an interactive `agy` session against a probe project carrying one skill under each root: it listed **both** `straymark-probe-alias` and `straymark-probe-canonical`. `.agent/` is a real alias; the shipped channel needs no change. Durable lesson: `agy -p` does not exercise workspace customization discovery — use an interactive session for that class of question. |
| R2 | `prune_retired` is the first code path that deletes adopter files outside `straymark remove`. | Provenance-gated on the checksum store, conservative by default (when in doubt, keep), and every kept file is named in the report. |
| R3 (new, not in Charter) | `.gemini/skills/` had drifted behind `.claude/skills/` in 7 of 15 skills, undetected, because nothing regenerated or gated it — while the generated `.codex/` channel was byte-perfect. | Fixed by construction: the replacement channel is generated, not hand-mirrored, and CI gates it. The general lesson — a hand-maintained mirror without a gate *will* drift — is why `.qoder/` and `.qwen/` got their own CI job last release. |
| R4 (new, not in Charter) | Attempting the probe required touching `~/.gemini/trustedFolders.json`; the write was blocked and I asked instead of forcing it. | Correct outcome, noted because it will recur: verifying agent-runtime behavior often needs config the agent should not silently grant itself. |

## Follow-ups

- After the tag: verify on a real `straymark update` from fw-4.41.0 that both
  retired directories disappear and the summary names anything kept — chained
  with the `QWEN.md` init/update verification still pending from
  [AILOG-2026-08-05-003].
