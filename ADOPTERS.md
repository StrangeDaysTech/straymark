# StrayMark Adopters

This is the living registry of projects that have adopted StrayMark **and committed to sending
telemetry and findings upstream**. It's deliberately not a popularity list — being here means a
project feeds real evidence back into the framework's evolution.

> Guides live in [`docs/adopters/`](docs/adopters/). This file is the *registry*; the
> [Adopter Feedback guide](docs/adopters/ADOPTER-FEEDBACK.md) explains the intake flow.

## Registry

| Project | Org | Domain / Stack | Since | Telemetry shared | Adoption discussion | N-status |
|---------|-----|----------------|-------|------------------|---------------------|----------|
| [Sentinel](https://github.com/StrangeDaysTech/sentinel) | Strange Days Tech | Go backend service | fw-2.x | Charter telemetry, dual external audits, pattern candidates | — (pre-dates this registry) | **N=1** — reference adopter |
| [LNXDrive](https://github.com/StrangeDaysTech/lnxdrive) | Strange Days Tech | Rust — Linux cloud-sync daemon + desktop (FUSE / D-Bus / systemd) | fw-4.19.0 | Charter telemetry, dual external audits, pattern candidates | [#205](https://github.com/StrangeDaysTech/straymark/discussions/205) | **N=2** — second domain (vs Sentinel's Go backend) |

*Want to be listed? See [How to get listed](#how-to-get-listed).*

## How the N-status works

StrayMark crystallizes patterns by **independent validation count**, not by intuition:

- **N=1** — a pattern observed in a single project/domain. It gets *documented* (in
  `dist/.straymark/00-governance/<NAME>.md` once accepted upstream, or as a local RFC), but stays
  **manual**. Sentinel is the N=1 reference: most of the patterns shipped between fw-4.13 and fw-4.19
  (Polish Charter, Charter-chain evolution, surface-declaration-without-wiring) originated there.
- **N=2** — a *second, independent* validation, ideally in a **different domain** (a Rust desktop app
  validating a pattern first seen in a Go backend is far stronger than another Go backend). N=2 is the
  gate that justifies **automating** the pattern in the CLI.

This is why the registry tracks domain and N-status: an adopter announcing they'll validate an
existing N=1 pattern in a new domain is the most valuable signal the project receives.

## How to get listed

1. **Announce** — open a discussion in the **Adopters** category
   ([new discussion](https://github.com/StrangeDaysTech/straymark/discussions/new?category=adopters)).
   The form captures your stack, the versions you adopted, what feedback you commit to, and any
   N-context (a pattern you'll be validating).
2. **A maintainer adds your row** here, linking back to that discussion.
3. **Send findings** as you go — actionable ones as Issues using the *Adopter feedback / upstream
   finding* template, each cross-linked to your adoption discussion. See the
   [Adopter Feedback guide](docs/adopters/ADOPTER-FEEDBACK.md).

> **On privacy:** telemetry (`.straymark/charters/CHARTER-NN.telemetry.yaml`) stays in *your* repo by
> default. Nothing is collected automatically. Sharing is always your explicit, manual act — anonymize
> anything sensitive before posting it upstream.

---

*StrayMark — Because every change tells a story.*

[Strange Days Tech](https://strangedays.tech)
