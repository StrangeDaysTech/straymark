---
id: AILOG-2026-08-04-002
title: Lote 2 adopter fixes — Qoder CLI skills (#399) and update-framework user-config preservation (#388)
status: accepted
created: 2026-08-04
agent: qodercli-v1.0
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 0
files_modified: []
observability_scope: none
tags: [qoder-cli, skills, install-skills, update-framework, checksums, adopter-feedback, cli]
related: []
---

# AILOG: Lote 2 adopter fixes — Qoder CLI skills (#399) and update-framework user-config preservation (#388)

## Summary

Second remediation batch from the 2026-08-04 open-issue triage. #399 adds
Qoder CLI to the agents StrayMark ships skills for (`dist/.qoder/skills/` +
`straymark install-skills --agent qoder`). #388 fixes an inverted invariant
in `straymark update-framework`: files the operator chose to keep were
stamped into `.straymark/.checksums.json` with the on-disk (user) hash
instead of the release hash, so the very next update treated them as
unmodified releases and re-prompted or re-overwrote them.

## Context

Qoder CLI discovers skills at `<project>/.qoder/skills/<name>/SKILL.md` and
at user level in `$QODER_CONFIG_DIR/skills/` (default `$HOME/.qoder/skills/`),
accepting the same full SKILL.md frontmatter Claude Code uses. The #388
reporter's hypothesis — that the checksum store recorded the wrong baseline
for kept files — was confirmed in code before the fix.

## Actions Performed

1. **#399 — Qoder skills.**
   - `dist/.qoder/skills/`: byte-for-byte mirror of `dist/.claude/skills/`
     (15 skills); Qoder consumes the full Claude-format frontmatter, so no
     generator (like `gen_codex_skills`) is needed.
   - `dist/dist-manifest.yml`: `.qoder/skills/` added so `init` /
     `update-framework` materialize it in adopter projects.
   - `cli/src/commands/install_skills.rs`: the former Codex-only installer
     was generalized to `install_user_level(agent, project_subdir,
     resolve_home, …)`; `--agent qoder` installs from `<path>/.qoder/skills/`
     into `$QODER_CONFIG_DIR/skills/` (fallback `$HOME/.qoder/skills/`).
   - `cli/src/main.rs`: `qoder` added to the `--agent` value parser and doc
     comments. `cli/src/commands/remove.rs` now removes `.qoder/skills` —
     and `.codex/skills`, which had been missing since fw-4.19.0.
   - `.gitignore` (self-adoption), README (tree, directory table, platform
     table), CLI-REFERENCE in en/es/zh-CN updated.
   - New `cli/tests/qoder_skills_test.rs`: parity test (dist/.qoder/skills
     mirrors dist/.claude/skills file-for-file and byte-for-byte) plus an
     end-to-end `install-skills --agent qoder` test against a temp
     `QODER_CONFIG_DIR`.
2. **#388 — checksum baseline for kept files.** `update_files()` now records
   the *distributed* (release) hash for every file that reaches disk logic
   (new files, identical files, and all Keep/Use/Backup selections), and
   `save_checksums()` applies those hashes as overrides after the disk walk.
   A kept `config.yml` therefore stores the release hash, so subsequent
   updates see it as "modified by the user" and keep honoring the keep
   decision instead of silently converging. The final report also names
   each kept file (`- <path> (kept your version)`), because a bare count is
   what let the original overwrite go unnoticed.

## Modified Files

| File | Change Description |
|------|--------------------|
| `cli/src/commands/install_skills.rs` | generalized installer; `--agent qoder` + `resolve_qoder_home()` |
| `cli/src/main.rs` | `qoder` in value parser + doc comments |
| `cli/src/commands/remove.rs` | remove `.qoder/skills` (and the previously missing `.codex/skills`) |
| `cli/src/commands/update_framework.rs` | `UpdateStats.skipped_files` + `distributed_hashes`; `save_checksums` overrides; named kept-files report + unit test |
| `cli/tests/qoder_skills_test.rs` | parity + end-to-end install tests (new) |
| `dist/.qoder/skills/` | 15 skills, mirror of `dist/.claude/skills/` (new) |
| `dist/dist-manifest.yml` | `- .qoder/skills/` |
| `.gitignore` | `/.qoder/skills/` |
| `README.md`, `docs/adopters/CLI-REFERENCE.md` (+ i18n es/zh-CN) | Qoder documented as supported agent |

## Decisions Made

- **#399 — literal mirror, not a generator.** Codex needs a generator
  because its skills strip frontmatter down to `name`+`description`. Qoder
  accepts the full Claude format, so a byte-identical copy is simpler and
  the parity test pins it against drift.
- **#388 — release hash wins in the store.** The checksum store's contract
  is "what the release shipped"; user deviations are detected *against* it.
  Storing the on-disk hash inverted the contract. Keep/Backup/Use all map
  to the same stored hash; only the on-disk content differs.
- **Scope: `.codex/skills` removal fix folded in.** `straymark remove`
  leaked `.codex/skills/` since fw-4.19.0 — same line list, same defect
  class as the `.qoder/skills` addition.

## Impact

- **Functionality**: Qoder users get skills via `init` + one
  `install-skills --agent qoder`; `update-framework` stops clobbering
  user-kept files across successive updates.
- **Performance**: N/A
- **Security**: N/A
- **Privacy**: N/A
- **Environmental**: N/A

## Verification

- [x] Code compiles without errors
- [x] Tests pass — `cargo test --no-fail-fast` in `cli/`: all suites green
  except the pre-existing `audit_template_test::unified_template_has_seven_universal_sections`
  failure documented in AILOG-2026-08-04-001 (R1). New tests: 2 parity/install
  tests + 1 checksum-override test, all passing.
- [x] Manual review performed
- [ ] Security scan passed (if risk_level: high/critical) — N/A (low)
- [ ] Privacy review completed (if handling PII) — N/A

## Risk

- R1 (known, accepted): pre-existing failing test on main — unchanged from
  AILOG-2026-08-04-001.
- R2 (new): `straymark validate --agent` still only supports `codex`;
  a `qoder` variant (checking `~/.qoder/skills/straymark-*`) is a natural
  follow-up if Qoder adoption grows.
- R3 (new): `.straymark/` installed copies are gitignored in this repo;
  `dist/` is the source of truth for this PR.

## Follow-ups

- (new) Consider `straymark validate --agent qoder` (see R2).
- (new) Evaluate a `gen_qoder_skills --check`-style CI guard if the manual
  mirror ever drifts; the parity test covers it inside `cargo test` today.

---

<!-- AILOG generated by qodercli-v1.0 | StrayMark | https://strangedays.tech -->
