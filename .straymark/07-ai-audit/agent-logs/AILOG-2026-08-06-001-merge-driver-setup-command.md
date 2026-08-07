---
id: AILOG-2026-08-06-001
title: install-merge-driver — close FU-001 by making the #391 fix reachable, and correct what skipping the setup actually costs
status: accepted
created: 2026-08-06
agent: claude-opus-5-v1.0
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 420
files_modified:
  - cli/src/commands/followups/install_merge_driver.rs
  - cli/src/commands/followups/mod.rs
  - cli/src/commands/init.rs
  - cli/src/main.rs
  - cli/tests/install_merge_driver_test.rs
  - docs/adopters/CLI-REFERENCE.md
  - CONTRIBUTING.md
  - .gitattributes
  - CHANGELOG.md
observability_scope: none
tags: [follow-ups, merge-driver, git, adopter-feedback, cli, dogfood]
related:
  - AILOG-2026-08-04-003
---

# AILOG: `install-merge-driver`

## Summary

FU-001 asked whether the merge-driver setup should be wired into `straymark
init` behind a prompt. Implemented — plus a standalone command, because `init`
alone would have missed every installation that already exists, which is all of
them. Verification of the change also corrected a claim I had written into the
code an hour earlier.

## Context

The registry merge driver shipped in cli-3.41.0 to close [#391], reported from
Sentinel with concrete evidence: three closures (`FU-055-003/004/017`) silently
reverted while merging three parallel PRs of one Charter.

Checking the premise before building found the fix **inert everywhere**:

| Installation | Registry entries | `.gitattributes` | `git config` |
|---|---|---|---|
| Sentinel (filed #391) | 296 | absent | absent |
| LNXDrive | 19 | absent | absent |
| straymark (self-adoption) | 4 | absent | absent |

The global CLI was already 3.41.0, so the driver was available in all three.
It had, however, only been published ~16 hours earlier — so non-adoption was
**not** evidence of setup friction, and I said so. The operator decided to build
anyway. What the data did support was a different framing: the fix was not
reaching the adopter who asked for it, for the mundane reason that a two-command
setup step had not been run.

That reframing is what made a standalone command necessary. `init --merge-driver`
helps nobody who is already initialized — and `straymark init` refuses to run on
an existing installation.

## Changes

- **`straymark followups install-merge-driver [--path .]`** — writes both halves,
  idempotently. An existing binding (even to a *different* driver) and an
  existing `merge.straymark-followups.driver` pointing elsewhere are both left
  untouched and reported: overwriting a deliberate adopter override silently is
  not the tool's call.
- **`straymark init --merge-driver` / `--no-merge-driver`**, plus an interactive
  prompt when neither flag is given. The prompt is gated on `stdin.is_terminal()`
  and on the target being a git repo — `init` runs in CI and provisioning
  scripts, where a blocking prompt is a hang, not a nicety. This mirrors the
  `--hooks` precedent (principle #6, friction with consent).
- **Dogfood**: wired in this repo, and documented as a required clone step in
  `CONTRIBUTING.md` (see §Risk R1).

## The correction

I wrote, in the module doc and in the `.gitattributes` block the command
generates, that without the git-config half the attribute line is "inert" and
git "silently falls back to a normal conflict".

**Both are wrong.** An A/B merge on a seeded registry — same two branches, same
conflicting frontmatter counters, driver configured in one clone and unset in
the other — gives:

- with the driver: merge succeeds, **all three closures survive**, counters consistent;
- without it: `fatal: custom merge driver straymark-followups lacks command line`
  and the merge aborts.

So the committable half is not harmless on its own; it *breaks merges* for
anyone who has not run the setup. Corrected in the module doc, in the generated
`.gitattributes` comment, in the command's output, and in CLI-REFERENCE
(EN/es/zh-CN). It also strengthens the case FU-001 made: "run two commands per
clone" is not a documentation problem when forgetting them is a hard stop.

## Verification

- `cargo test`: **958 passed, 0 failed**.
- `cli/tests/install_merge_driver_test.rs` — 5 tests: both halves written;
  idempotent (a second run adds no duplicate attribute line, which matters
  because the `init` prompt can be accepted more than once); an existing
  `.gitattributes` survives including its last line when the file lacks a
  trailing newline; a foreign driver is preserved and reported; refuses outside
  a git repo; `init` exposes both flags.
- **End-to-end in anger**, which is the only test that proves the wiring does
  anything: two branches each closing different entries so the CLI-owned
  counters diverge and conflict. Merged clean with all closures intact — and
  the same scenario without the driver aborts, as above.

## Risk

| Id | Risk | Handling |
|----|------|----------|
| R1 | Committing `.gitattributes` here means any contributor who merges a branch touching the registry hits `fatal: … lacks command line` until they run the command. | Accepted, and mitigated where a contributor will actually look: a numbered step in `CONTRIBUTING.md` § Setup Steps, and the failure mode spelled out in a comment inside `.gitattributes` itself. The alternative — not committing the binding — means the driver never works for anyone, which is the state that produced this AILOG. |
| R2 (new, not in Charter) | The premise behind FU-001 ("manual setup causes friction") was **never verified**; 16 hours of non-adoption proves nothing. The feature was built on the operator's call, over a stated recommendation to dogfood first. | Recorded plainly rather than retro-justified. The real signal to watch is now cheap to collect: this repo is wired, so the next parallel-PR merge here either exercises the driver or does not. |

## Follow-ups

- Wire the driver in Sentinel and LNXDrive — Sentinel is the adopter that filed
  #391 and the only installation with enough registry volume (296 entries) to
  hit the conflict routinely. Until then the fix remains unexercised where it
  was needed.
