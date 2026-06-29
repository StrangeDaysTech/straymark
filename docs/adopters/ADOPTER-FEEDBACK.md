# StrayMark - Adopter Feedback

**How to announce your adoption and send telemetry and findings upstream.**


---

## Table of Contents

1. [Why feedback matters](#why-feedback-matters)
2. [Two channels](#two-channels)
3. [What telemetry is — and where it lives](#what-telemetry-is-and-where-it-lives)
4. [How to share it](#how-to-share-it)
5. [The N=1 → N=2 gate](#the-n1--n2-gate)
6. [Quick reference](#quick-reference)

---

## Why feedback matters {#why-feedback-matters}

StrayMark does not collect anything automatically. There is no remote endpoint, no opt-in beacon, no
adoption dashboard. The framework evolves on **evidence that adopters choose to send** — which means
the quality of the next release is a direct function of what real projects report back.

Most of the patterns shipped between fw-4.13 and fw-4.19 came from a single adopter
([Sentinel](https://github.com/StrangeDaysTech/sentinel)) feeding findings upstream over many
Charters. The framework gets better when more projects do the same, from more domains.

## Two channels {#two-channels}

Feedback has two distinct natures, so it has two homes:

| | **Announcement** | **Findings** |
|---|---|---|
| **Where** | Discussions → **Adopters** category | **Issues** (`Adopter feedback / upstream finding` template) |
| **When** | Once, when you adopt | Ongoing, as you discover things |
| **What** | Your project, stack, versions, what you commit to send | A specific gap, friction, bug, or pattern candidate — telemetry-backed |
| **Lifecycle** | Stays open as your adoption record | Closed when addressed |

Open the [Adopters discussion](https://github.com/StrangeDaysTech/straymark/discussions/new?category=adopters)
first; then **cross-link** every finding Issue back to it. That link is what ties a finding to a known
adopter and its N-context.

## What telemetry is — and where it lives {#what-telemetry-is-and-where-it-lives}

When you close a Charter (`straymark charter close`), StrayMark records structured telemetry to
`.straymark/charters/CHARTER-NN.telemetry.yaml` — estimation accuracy (time, not line-count), agent
behavior, external-audit results, scope changes, and qualitative wins/frictions. The shape is defined
by [`charter-telemetry.schema.v0.json`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/schemas/charter-telemetry.schema.v0.json).

**This file stays in your repository.** It is not transmitted anywhere. Sharing it upstream is always
a deliberate, manual act on your part.

## How to share it {#how-to-share-it}

1. **Decide what's relevant.** A whole telemetry file is rarely needed — the useful part is usually a
   block (an `effort` drift, an `external_audit` delta, a `qualitative.friction_points` list) that
   backs a specific claim.
2. **Anonymize.** Strip anything sensitive — internal names, secrets, private repo paths — before it
   leaves your repo.
3. **Attach it to a finding.** Paste the excerpt into the *Adopter feedback / upstream finding* Issue,
   in the telemetry field (rendered as YAML), and link your adoption discussion.

The maintainer may, with your consent, anonymize and aggregate findings across projects into blog
posts or documentation — but only what you've chosen to publish.

## The N=1 → N=2 gate {#the-n1--n2-gate}

StrayMark crystallizes patterns by **independent validation count**:

- **N=1** — a pattern seen in one project/domain. Documented, but kept **manual**.
- **N=2** — a second, independent validation, ideally in a **different domain**. This is the gate that
  justifies **automating** the pattern in the CLI.

A Rust desktop app validating a pattern first observed in a Go backend is a far stronger N=2 than
another Go backend. So when you announce — and when you file findings — say whether you're validating
an existing pattern, and from which domain. That single piece of context is often the most valuable
thing an adopter contributes.

## Quick reference {#quick-reference}

| You want to… | Do this |
|---|---|
| Announce your adoption | Open an [Adopters discussion](https://github.com/StrangeDaysTech/straymark/discussions/new?category=adopters) |
| Report a gap / friction / pattern | Open an Issue with the *Adopter feedback / upstream finding* template |
| Back a finding with data | Paste an anonymized `charter_telemetry:` excerpt into the Issue |
| Get into the registry | Announce first; a maintainer adds you to [`ADOPTERS.md`](https://github.com/StrangeDaysTech/straymark/blob/main/ADOPTERS.md) |

See also: [Adoption Guide](ADOPTION-GUIDE.md) · [Recommended Workflows](WORKFLOWS.md) · [CLI Reference](CLI-REFERENCE.md)

---

*StrayMark — Because every change tells a story.*

[Strange Days Tech](https://strangedays.tech)
