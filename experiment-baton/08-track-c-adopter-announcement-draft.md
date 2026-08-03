# Discussion draft — Baton Track C call for adopters

> **Where to post:** GitHub Discussions → **Adopters** category
> (`https://github.com/StrangeDaysTech/straymark/discussions/new?category=adopters`).
> **Language:** English below; post bilingually (es translation appended) if the
> adopter base warrants — Sentinel alone is es-speaking.
> **Post after:** the `ci(baton)` release-channel PR merges and `baton-0.2.0`
> is cut (`git tag baton-0.2.0 && git push origin baton-0.2.0`).

---

**Title:** Baton Track C — declared work routing is open for forward-validation (experimental)

**Body:**

Baton — StrayMark's experimental cost-aware work router — has graduated its
schema into the Framework (fw 4.38.0) and now needs **real governance traffic**
to validate it. Track C is open: here is what that means for your project.

## What changed

Since decision #332, Baton no longer guesses work classes from titles.
Classification is **declaration-based**: you declare the class of each work
unit in frontmatter, and Baton routes from that declaration — honestly.
An undeclared unit is not an error; it routes to the frontier tier with a
nudge to declare.

The fields (optional, advisory — absence is silent, invalid values warn but
never block):

```yaml
work_verb: design | implement | audit | operate
design_provenance: new | upstream   # only significant for implement
```

Two determination rules worth knowing: defining a bounded foundational
contract is `implement`, not `design`; and `implement` +
`design_provenance: upstream` degrades to mechanical work.

## What we're asking

1. **Declare `work_verb` on new units** — Charter frontmatter and follow-up
   backlog entries. Nothing else in your cadence changes.
2. **Grab the Baton binary** from the `baton-*` release (GitHub-release-only,
   like Loom; only the latest is kept): download your platform's
   `straymark-baton-v{version}-{target}` asset, extract, put on PATH.
3. **Sanity check** (read-only, mutates nothing):

   ```bash
   straymark-baton classify .
   straymark-baton route . --dry-run
   ```

4. **After 2–4 weeks** of declarations, run the simplified calibration from
   the [adopter kit](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/07-track-c-adopter-kit.md)
   (sample ~20–30 declared units, label true verb/provenance, report
   agreement) and file findings via the usual
   [Adopter feedback channel](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/ADOPTER-FEEDBACK.md).

## What we're NOT asking

No title-scan enrichment, no cost estimates, no model execution, no changes
to your CI. Baton is read-only and recommend-only; `route` requires
`--dry-run` and has no execution path.

## Docs

- [Baton guide for adopters](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/BATON.md) (EN · es · zh-CN)
- [Adopter kit — Track C](https://github.com/StrangeDaysTech/straymark/blob/main/experiment-baton/07-track-c-adopter-kit.md)
- [CLI reference — the advisory vocabulary checks](https://github.com/StrangeDaysTech/straymark/blob/main/docs/adopters/CLI-REFERENCE.md)

This is gate #3 of five for graduating Baton to `straymark-core`. Your
forward-validation data is what unlocks it.

*StrayMark — Because every change tells a story.*
