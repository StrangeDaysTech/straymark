---
id: AIDEC-2026-08-05-001
title: Qwen Code as a first-class agent surface — mirror the Claude channel, and make update-framework create new injection targets
status: accepted
created: 2026-08-05
agent: claude-opus-5-v1.0
confidence: high
review_required: false
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
tags: [multi-agent, qwen-code, qoder, directive-injection, dist-manifest, update-path]
related:
  - AILOG-2026-08-05-003
  - AILOG-2026-08-04-002
---

# AIDEC: Qwen Code as a first-class agent surface

## Context

A review of StrayMark's integration with the Qwen Code and Qoder CLIs — asked
for both at `init` time and at later `update` time — surfaced one silent hole
and one false claim.

**Qwen Code had zero integration.** No `dist/.qwen/`, no manifest entry, no
`install-skills` branch, no row in any compatibility table. That matters more
than the absence suggests: verified against the installed runtime
(`@qwen-code/qwen-code`), its context-file resolver falls back to `["QWEN.md"]`
when nothing is configured, and `AGENTS.md` appears only in a safety string
listing files the agent must not modify without permission. A Qwen Code user
therefore received **no StrayMark governance at all** — not even the AGENTS.md
fallback that covers Codex and Qoder.

Two runtime facts shaped the design, both read directly from the shipped code
rather than assumed:

- `Storage.getGlobalQwenDir()` returns `$QWEN_HOME` when set, `$HOME/.qwen`
  otherwise; user-level skills live at `<dir>/skills/`.
- Skills resolve at **both** scopes — `<projectRoot>/.qwen/skills/` and the
  global dir — and the frontmatter parser accepts `allowed-tools`.

**Qoder's documentation was wrong.** README and CLI-REFERENCE (three languages)
stated Qoder reads skills from `~/.qoder/skills/` "not from the project tree".
The application bundle resolves a `Project` scope
(`<projectRoot>/.qoder/skills/`) and watches it, so `install-skills --agent
qoder` is a convenience, not a prerequisite. It also searches `**/AGENTS.md`
and injects the matches as instructions, which is why Qoder was in fact
governed while Qwen Code was not.

## Problem

Two decisions had to be made together, because the second determines whether
the first reaches anyone who already installed StrayMark.

1. What shape should the Qwen Code channel take — which skill frontmatter, and
   which directive file?
2. `update_framework.rs` skipped every injection target absent from disk
   (`if !target_path.exists() { continue; }`). A new agent surface added to the
   manifest would therefore land only on fresh `init`s. Meanwhile
   `STRAYMARK.md` § "Directive Injection Markers" states that `init`,
   `update-framework` **and** `repair` all "create any missing target file".
   Code and documentation contradicted each other; one of them had to move.

## Alternatives Considered

### Alternative 1: Mirror `.claude/` byte-for-byte, pinned by a parity test

**Description**: `dist/.qwen/skills/` is a straight copy of
`dist/.claude/skills/`, guarded by the same shape of test that guards Qoder
(#399), plus a CI job. `QWEN.md` clones the `GEMINI.md` template with a
`qwen-code-v{version}` identity.

**Pros**:
- Qwen Code parses `allowed-tools`, so the richer frontmatter is usable, not
  merely tolerated. Stripping it would lose real tool-scoping information.
- Identical to the precedent already set for Qoder — one pattern, not two.
- A parity test states the invariant in one line and fails loudly on drift.

**Cons**:
- Duplicates 15 files in the repo (~2,300 lines) that a generator could emit.

### Alternative 2: Clone `.gemini/` (minimal frontmatter)

**Description**: Qwen Code is a Gemini CLI fork, so ship the reduced
`name` + `description` frontmatter.

**Pros**:
- Conservative if the parser turned out to be strict about unknown keys.

**Cons**:
- Contradicted by the runtime: 94 `allowedTools` references in the bundle, and
  `allowed-tools` present in the skill frontmatter path. The conservatism buys
  nothing and discards tool scoping.

### Alternative 3: Generate `.qwen/` from `.claude/` like `.codex/`

**Description**: Add a `gen_qwen_skills` binary and a `--check` CI gate.

**Pros**:
- No duplicated content in the repo.

**Cons**:
- A generator earns its keep when it *transforms* (Codex strips frontmatter
  keys). Here the transformation is the identity function, so it would be a
  build step whose only output is a copy — more moving parts guarding less.

### Alternative 4 (problem 2): Fix the documentation instead of the code

**Description**: Leave `update-framework` skipping missing targets and amend
`STRAYMARK.md` to say `straymark repair` is required after a new agent lands.

**Pros**:
- Smallest diff; no behavior change for existing installations.

**Cons**:
- Makes every future agent surface depend on adopters reading a release note
  and running a second command. The failure is silent: the adopter's Qwen Code
  keeps starting with no governance and nothing reports it.
- The documented contract is the better one. Choosing the worse behavior to
  match a stale sentence inverts the relationship between intent and code.

## Decision

**Chosen**: Alternative 1 for the channel shape, and *fix the code* (rejecting
Alternative 4) for the update path.

**Justification**: The mirror matches both the runtime's actual capabilities
and the precedent set for Qoder, and the parity test plus the new CI job make
the duplication self-correcting rather than a maintenance liability. On the
update path, `STRAYMARK.md` already described the behavior adopters need; the
code was the outlier. Creating missing targets during `update-framework` is
also what makes this release meaningful for existing installations rather than
for new ones only — the exact question that prompted the review.

## Consequences

### Positive
- Qwen Code users receive the governance pointer (`QWEN.md`) and all 15 skills.
- Any *future* agent surface now reaches existing installations through the
  ordinary `straymark update`, with no second command and no release note to
  read.
- The Qoder and Qwen mirrors are gated in CI; previously only `.codex` was, and
  the Qoder parity test lived in a suite the pipeline never ran.
- `validate --agent` stopped being Codex-shaped and is now per-agent.

### Negative
- Six directive files are now written into an adopter's project root at `init`
  (`AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `QWEN.md`, plus the two Cursor files).
  Root clutter grows with each supported CLI, and there is still no opt-out
  short of editing the manifest.
- `dist/` carries a third byte-identical copy of the skill corpus.

### Risks
- **A future release could now overwrite a file the operator had deleted on
  purpose.** Deleting `QWEN.md` no longer survives `straymark update` — it is
  restored. Mitigation: this is the documented contract (`STRAYMARK.md` § 
  "Directive Injection Markers": "There is no opt-out per target short of
  editing the manifest"), and it matches what `repair` has always done. If
  adopter feedback shows people deleting directive files deliberately, the
  right fix is a per-target opt-out in `config.yml`, not re-introducing the
  skip.
- **Qwen Code's default context filename could change.** It is a fallback in
  the resolver, not a constant an adopter can rely on forever. Mitigation:
  `AGENTS.md` is also shipped, so a runtime that adopts the open standard
  degrades to covered rather than uncovered.
