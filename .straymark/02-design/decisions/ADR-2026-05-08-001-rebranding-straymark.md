---
id: ADR-2026-05-08-001
title: Rebranding from "DevTrail" to "StrayMark"
status: accepted
created: 2026-05-08
updated: 2026-05-08
agent: claude-opus-4-7-1m
confidence: high
review_required: true
reviewed_by: pepemontfort@gmail.com
reviewed_at: 2026-05-08
review_outcome: approved
risk_level: medium
eu_ai_act_risk: not_applicable
iso_42001_clause: []
alternatives_documented: []
api_changes: []
tags: [branding, naming, trademark, governance]
related: []
supersedes: []
---

# ADR: Rebranding from "DevTrail" to "StrayMark"

## Status

**accepted** (2026-05-08)

> **Immutability Rule**: Once an ADR reaches `accepted` status, it MUST NOT be modified. If the decision changes, create a new ADR with `supersedes: ADR-2026-05-08-001` in its frontmatter. The original ADR's status changes to `superseded`.

## Context

The project was originally named "DevTrail" — a documentation governance framework + Rust CLI for AI-assisted development, published at `github.com/StrangeDaysTech/devtrail` and as `devtrail-cli` on crates.io (active at version 3.10.0 with 286 downloads at the time of this decision). The framework distributes templates, governance policies, and agent directives to adopter projects via a `.devtrail/` directory and a CLI binary named `devtrail`.

Prior to broader adoption, the operator (project owner) commissioned trademark conflict research via a Claude.ai web session. The research surfaced material risks around the "DevTrail" mark — sufficient that continued use under that name would carry **legal uncertainty about trademark ownership** as the project gained adopters.

The current state of adoption favors action now over later:

- Only one adopter is known: `StrangeDaysTech/sentinel`, which is itself owned by the same operator. There are no third-party adopters that would be impacted by a name change.
- The `devtrail-cli` crate has 286 downloads — modest, with no known production dependents.
- 2 issues are open in the project (#93, #113); 0 PRs.
- The framework reached `fw-4.10.0` and CLI `cli-3.10.0` in the days prior to this decision, having just shipped the audit-skills v1 flow (Issue #102) and the follow-ups backlog pattern (Issue #111). The product is materially complete enough to deserve a stable identity going forward.

The decision is motivated by **legal certainty over the trademark** rather than by product strategy or user feedback. The name "StrayMark" was selected by the operator from a shortlist (etymological rationale to be elaborated in `README.md` in a separate, future task — not part of the rebranding execution).

Verified pre-flight on 2026-05-08:

- `github.com/StrangeDaysTech/straymark` — 404, name available.
- `crates.io/crates/straymark-cli` — 404, name available.
- `crates.io/crates/straymark` — 404, name available (potential reservation as umbrella).

## Decision

We will rebrand the project, end-to-end, from **"DevTrail" to "StrayMark"**, with the following scope and boundaries:

**In scope (the "live state" of the project):**

- All identifiers in source code: Rust struct names (`DevTrailConfig`→`StraymarkConfig`, `DevTrailDocument`→`StraymarkDocument`), the `GITHUB_REPO` constant, error/help/about strings, hardcoded path literals (`.devtrail/`→`.straymark/`).
- Cargo metadata: package name (`devtrail-cli`→`straymark-cli`), binary name (`devtrail`→`straymark`), description, repository URL, keywords.
- Source-of-truth path: `dist/.devtrail/`→`dist/.straymark/` (preserving per-file git history with `git mv`), `dist/DEVTRAIL.md`→`dist/STRAYMARK.md`.
- 30 skill/workflow files across `.claude/skills/`, `.gemini/skills/`, `.agent/workflows/` (filenames + bodies).
- Public documentation: `README.md`, `CLAUDE.md`, governance docs (3 langs), adopter docs (3 langs), `CONTRIBUTING.md` references. Footer version banners.
- CI workflows: `release-cli.yml` and `release-framework.yml` titles, asset filename prefixes (`devtrail-fw-*` → `straymark-fw-*`, `devtrail-cli-*` → `straymark-cli-*`), binary path references.
- `dist/dist-manifest.yml`: description, repository URL, file list.
- Private local artifacts: `Propuesta/devtrail-*.md` filenames + bodies, `china/investigacion.md` references, agent persistent memory.
- GitHub repository name: `StrangeDaysTech/devtrail` → `StrangeDaysTech/straymark` via `gh api -X PATCH`.
- Next release: `fw-4.11.0` and `cli-3.11.0`, published as "StrayMark Framework 4.11.0" and "StrayMark CLI 3.11.0", with assets named `straymark-fw-4.11.0.zip` and `straymark-cli-v3.11.0-{linux,darwin-x86_64,darwin-aarch64,windows}.{tar.gz,zip}`. The `straymark-cli` crate published fresh on crates.io at version `3.11.0`.

**Out of scope (immutable history, preserved verbatim):**

- All commits, commit messages, and git history.
- All tags published before this ADR (`fw-4.10.0`, `cli-3.10.0`, and earlier).
- All release titles and bodies of releases published before the rebranding (they retain "DevTrail Framework X.Y.Z").
- All previously merged PRs, closed issues, and their bodies/comments — preserved as historical record.
- All prior `CHANGELOG.md` sections — preserved literally with the "DevTrail" name where present.
- The `devtrail-cli@3.10.0` crate on crates.io — **not yanked**. Yanking has the semantic of "do not use this version", which is incorrect: the version was valid in its time. Leaving it accessible means historical adopters (improbable) can still build.
- Sentinel adopter's AILOGs and Charters that reference "DevTrail" — those are Sentinel's history, owned by the adopter, not by this repo.

## Alternatives Considered

### 1. Maintain "DevTrail" and accept trademark risk

- **Description**: Keep the current name, continue publishing and growing adoption under "DevTrail".
- **Pros**:
  - Zero migration cost.
  - Brand continuity for the existing 286 downloads of `devtrail-cli`.
  - No risk of a third-party adopter (improbable but possible) being broken by the rename.
- **Cons**:
  - **Material legal uncertainty about trademark ownership** as adoption grows. Future cease-and-desist or forced rename mid-adoption is materially worse than rename now.
  - The rename window narrows over time — every new adopter increases the cost of changing later.
- **Why not**: The legal risk dominates. Acting now, with one self-owned adopter, is materially cheaper than acting later under pressure.

### 2. Hybrid branding — keep "DevTrail" as legacy, introduce "StrayMark" as parallel name

- **Description**: Publish a new `straymark-cli` crate alongside the existing `devtrail-cli`, dual-maintain documentation under both names, gradually deprecate "DevTrail".
- **Pros**:
  - Smoother transition for hypothetical existing users.
  - Hedges against the chance the new name is rejected by users.
- **Cons**:
  - Maintenance burden doubles (two CLIs, two doc trees, two crates).
  - Adopter confusion: which is the "real" name?
  - Ambiguous trademark position — using both names doesn't resolve the legal concern.
  - Framework is small enough that hard rename is cheaper than dual-maintenance overhead.
- **Why not**: Cost outweighs benefit. With one self-owned adopter, the migration is contained; dual-name confusion would be the dominant cost.

### 3. Reset to v0.1.0 under StrayMark, signal "new project"

- **Description**: Publish `straymark-cli@0.1.0` and `straymark-fw-0.1.0` as a "fresh start", marking discontinuity from DevTrail's 4.x.x and 3.x.x history.
- **Pros**:
  - Symbolic clean break.
  - Signals "we learned and rebooted" to any onlookers.
- **Cons**:
  - The product trajectory is continuous — only the name changes. Resetting versions misrepresents the maturity of the codebase.
  - CHANGELOG and release history become harder to follow ("Framework 4.10.0 → Framework 0.1.0" makes no narrative sense).
  - Operator-internal continuity (Sentinel adopter, memory in `.devtrail/` migrating to `.straymark/`) is naturally preserved by version continuity.
- **Why not**: The product is the same; only the name is different. Continuity is the more honest signal.

## Consequences

### Positive

- **Legal certainty over the trademark** going forward — the primary motivation, achieved at modest engineering cost (~12-15 focused hours).
- The migration is captured in a single coherent release (`fw-4.11.0` / `cli-3.11.0`) rather than dribbled across versions.
- Pre-flight verification confirmed `straymark-cli` and `straymark` are both available on crates.io, leaving room for future expansion.
- GitHub provides automatic redirects from the old repo URL for at least a year, protecting any silent adopters who may exist.

### Negative

- 9 sequential PRs touch ~260 files; the largest (PR 6 — public docs in 3 languages) is unavoidable churn in git blame.
- The Sentinel adopter (operator-owned) must perform a manual one-time migration (`mv .devtrail .straymark`, update CLAUDE.md/AGENT.md) — a small but real one-time cost.
- The `devtrail-cli@3.10.0` crate is left in place; users searching crates.io may still discover the old name and require a redirect via documentation rather than a tooling-level mechanism.

### Neutral

- The project's tag prefixes (`fw-X.Y.Z`, `cli-X.Y.Z`) are agnostic to the project name and continue unchanged.
- The org name `StrangeDaysTech` is unaffected — only the repo and product names change.

### Quality Impact Assessment

| Quality Characteristic (ISO 25010:2023) | Impact | Description |
|-----------------------------------------|--------|-------------|
| Functional Suitability | ~ | No functional change. Identifiers rename without altering behavior. |
| Maintainability | + | Identifier consistency under one brand reduces long-term confusion. |
| Compatibility | - | Breaking change for any adopter of `.devtrail/` paths or `devtrail` binary; mitigated by GitHub redirects (URLs) and the small known adopter set (1, self-owned). |
| Security | + | Legal certainty over the trademark reduces a non-technical attack surface (forced rename under hostile cease-and-desist). |
| Flexibility | + | Both `straymark-cli` and `straymark` are reserved on crates.io — room for future product expansion under a single brand. |

## Affected Components

| Component | Type of Change | Impact |
|-----------|----------------|--------|
| Rust source (`cli/src/`) | Modified | High (38 files) |
| Test suite (`cli/tests/`) | Modified | High (18 files, fixtures + assertions) |
| `cli/Cargo.toml` | Modified | High (package name, binary name, metadata) |
| `dist/.devtrail/` directory | Renamed (`git mv`) | High (122 files, history preserved per-file) |
| `dist/DEVTRAIL.md` | Renamed (`git mv`) | High |
| `dist/dist-manifest.yml` | Modified | Medium |
| Skills under `.claude/skills/`, `.gemini/skills/`, `.agent/workflows/` | Renamed + Modified | High (30 files) |
| Public docs (`README.md`, `CLAUDE.md`, `docs/`) | Modified | High (~250 files across 3 languages) |
| CI workflows (`.github/workflows/release-*.yml`) | Modified | Medium |
| GitHub repository name | Renamed (`gh api -X PATCH`) | High (preserved by automatic redirects) |
| crates.io publication | New crate | High (`straymark-cli@3.11.0` published; `devtrail-cli@3.10.0` left intact) |
| Private local dirs (`Propuesta/`, `china/`) | Renamed + Modified | Low (operator-private) |
| Agent persistent memory | Modified | Low (operator-local) |

## Implementation Plan

The rebranding executes as **9 sequential PRs** organized by layer (L1–L11 in the master plan). Each PR is admin-merged after CI passes, blast radius limited to one PR.

1. **PR 1 (L1)** — This ADR. Establishes the immutable record of the decision before touching anything else.
2. **L2** — GitHub repo rename via `gh api -X PATCH repos/StrangeDaysTech/devtrail -f name=straymark`. Not a PR; an out-of-tree operation.
3. **PR 3 (L3)** — Rebrand Rust source (Cargo.toml metadata, structs, GITHUB_REPO const, strings, test fixtures).
4. **PR 4 (L4)** — Rebrand paths and governance root (`dist/.devtrail/` → `dist/.straymark/`, `dist/DEVTRAIL.md` → `dist/STRAYMARK.md`, CLI hardcoded paths, dist-manifest).
5. **PR 5 (L5)** — Rebrand 30 skill/workflow files.
6. **PR 6 (L6)** — Rebrand public docs (README, CLAUDE.md, governance, adopter docs in 3 languages).
7. **PR 7 (L7)** — Rebrand CI workflows and asset filename prefixes.
8. **PR 8 (L9)** — Rebrand private local dirs (`Propuesta/`, `china/`) and agent persistent memory.
9. **PR 9 (L10+L11)** — Bump `dist-manifest.yml` to 4.11.0, `Cargo.toml` to 3.11.0, CHANGELOG entry, push tags `fw-4.11.0` and `cli-3.11.0`, CI publishes assets and `straymark-cli@3.11.0` on crates.io.

The full plan with branch names, scopes, and risks is in the operator's plan file (`/home/montfort/.claude/plans/serialized-bouncing-milner.md`).

## Success Metrics

- **Post-PR 9** (the rebranding is "complete"):
  - `gh release view fw-4.11.0` shows asset `straymark-fw-4.11.0.zip`.
  - `gh release view cli-3.11.0` shows assets prefixed `straymark-cli-v3.11.0-`.
  - `cargo install straymark-cli` works; `straymark --version` reports 3.11.0.
  - `straymark init` in a fresh directory creates `.straymark/` (not `.devtrail/`).
  - `gh api repos/StrangeDaysTech/devtrail` returns a 301/302 redirect to `/straymark`.
  - `https://crates.io/crates/straymark-cli` lists version 3.11.0.

## Validation Criteria

| Metric | Target Value | Measurement Method | Timeline |
|--------|-------------|-------------------|----------|
| Test suite green post-rebrand | 100% pass rate | `cargo test --all` after PR 4 | Same session |
| Release CI green for both workflows | Both jobs `success` | `gh run list --workflow=release-{cli,framework}.yml --limit 1` after PR 9 tag push | Within 30 min of push |
| Crate published successfully | `straymark-cli@3.11.0` reachable | `cargo search straymark-cli` returns the new crate | Within 30 min of release |
| GitHub redirect functional | Old URL returns 301/302 | `curl -I https://github.com/StrangeDaysTech/devtrail` | Immediately after L2 |
| No "DevTrail" in live state of repo | Only historical refs match (CHANGELOG, releases) | `grep -r "DevTrail" --include="*.md" --include="*.rs" --include="*.toml" --include="*.yml"` after PR 9 | Same session |

## References

- Trademark conflict research session (claude.ai web, 2026-05): operator-private, not linked here.
- Master rebranding plan: operator's plan file (private).
- Pre-flight availability checks: documented inline in this ADR's Context section.
- Issue #111 (Follow-ups backlog pattern): the most recent feature shipped under "DevTrail" before this ADR.
- Issue #102 (Audit v1): the prior major release under "DevTrail".

---

## Revision History

| Date | Author | Change |
|------|--------|--------|
| 2026-05-08 | José Villaseñor Montfort | Initial creation. Status: accepted. |

*This document was produced with assistance from generative AI tools (Claude 4.7); all responsibility for the content rests with the human author.*

<!-- Template: DevTrail | https://strangedays.tech (the project, at the time of this ADR's creation, was named DevTrail; this footer is preserved as historical evidence of the pre-rebrand state) -->
