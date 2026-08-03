---
id: AILOG-2026-08-02-003
title: Baton — release channel (release-baton.yml) + version 0.2.0
status: accepted
created: 2026-08-02
agent: qoder
confidence: high
review_required: false
risk_level: low
eu_ai_act_risk: not_applicable
nist_genai_risks: []
iso_42001_clause: []
lines_changed: 185
files_modified: [.github/workflows/release-baton.yml, experiment-baton/Cargo.toml, Cargo.lock, experiment-baton/07-track-c-adopter-kit.md]
observability_scope: none
work_verb: operate
design_provenance: upstream
tags: [baton, release, ci, distribution, version-bump]
related: [AILOG-2026-08-02-001, 07-track-c-adopter-kit]
---

# AILOG: Baton — GitHub-release channel landed; straymark-baton 0.2.0

## Summary

Closes the "binary handoff" gap in Track C. `experiment-baton/Cargo.toml`
already declared the intent ("Experimental: GitHub-release-only, mirroring
Loom"; `publish = false`) but the machinery never landed: no workflow, no
release, and the adopter kit's only answers were "we hand you the binary"
or "compile it yourself". This change lands the channel.

## What changed

- **`.github/workflows/release-baton.yml` (new)** — mirror of
  `release-loom.yml` (simplified: no Node/frontend build). Trigger: `baton-*`
  tag or `workflow_dispatch`. 4-target matrix (linux-x64, macos-x64,
  macos-arm, win-x64); guards that the tag is `baton-*` and that
  `experiment-baton/Cargo.toml` version matches the tag; assets named
  `straymark-baton-v{version}-{target}.{tar.gz|zip}`; release titled
  "StrayMark Baton {version} (EXPERIMENTAL)" with `--latest=false` (the CLI
  release stays the repo's "latest" so update flows are unaffected);
  **previous `baton-*` releases are deleted** — same only-latest posture as
  Loom, decided explicitly with the operator.
- **`experiment-baton/Cargo.toml`** — version 0.1.0 → **0.2.0**. 0.1.0
  predates the declared-work-verb turn (#332); the shipped classifier is a
  different mechanism, so the first real release starts at 0.2.0. `Cargo.lock`
  moved mechanically with it.
- **`07-track-c-adopter-kit.md` §1** — preconditions now point adopters at the
  `baton-*` GitHub release assets (with the four target names) instead of
  "te lo pasamos"; compile-from-source kept as the alternative.

## Release procedure (for the operator)

Once this commit is on main: `git tag baton-0.2.0 && git push origin
baton-0.2.0` — the workflow builds the four assets and creates the release.

## Verification

No Rust code changed; the workflow is a structural mirror of
`release-loom.yml` (same action versions, same job topology) with the
Loom-specific Node/frontend steps removed. Binary name (`straymark-baton`)
and manifest path (`experiment-baton/Cargo.toml`) verified against the
crate's `[[bin]]` section. The version-match guard in the workflow makes a
tag/Cargo.toml skew fail loudly instead of shipping a mislabeled release.

## EU AI Act Considerations

Not applicable — CI release tooling; no model inference, no personal data.
