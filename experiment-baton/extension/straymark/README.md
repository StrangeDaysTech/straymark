# StrayMark Coherence Bridge — SpecKit extension

Runs the [Baton](../../01-baton-concept.md) coherence engine at **authoring
time** — on SpecKit's `before_implement` hook — so a consumer is never
implemented against a stale or assumed contract (the issue #304 pattern). It is
the *activation seam* of the Coherence Bridge (`CHARTER-02-activation-seam`):
Phase 1 detects on demand; this surfaces the same findings before code is
written.

**Read-only and non-breaking:** the hook never mutates the repo, and if the
`straymark-baton` binary is unavailable it skips with a note rather than blocking
the SpecKit flow.

## Requirements

- SpecKit ≥ 0.11 (the extension/hooks system).
- The experimental `straymark-baton` binary (Baton is experimental — no release
  yet). Build it from this repo: `cargo build --release -p straymark-baton`
  (binary at `target/release/straymark-baton`).

## Install (manual, while experimental)

1. Copy this directory into the target project:
   `cp -r experiment-baton/extension/straymark <project>/.specify/extensions/straymark`
2. Copy the config template:
   `cp <project>/.specify/extensions/straymark/config-template.yml \`
   `   <project>/.specify/extensions/straymark/straymark-config.yml`
3. Register the hook in `<project>/.specify/extensions.yml` under
   `installed:` and `hooks.before_implement:`:
   ```yaml
   installed:
     - straymark
   hooks:
     before_implement:
       - extension: straymark
         command: speckit.straymark.coherence-check
         enabled: true
         optional: true
         prompt: Run the StrayMark coherence check before implementing?
   ```
4. Point the extension at the binary — either set `binary:` in
   `straymark-config.yml`, put `straymark-baton` on `PATH`, or export
   `BATON_REPO=<path-to-straymark-checkout>` (dev mode uses `cargo run`).

## Configuration (`straymark-config.yml`)

| Key | Values | Meaning |
|---|---|---|
| `gate` | `advisory` (default) / `block` | advisory surfaces and continues; block fails `before_implement` on blocking findings |
| `min_confidence` | `low` / `medium` (default) / `high` | minimum confidence to report |
| `binary` | path / empty | binary location; empty → PATH then `cargo`/`$BATON_REPO` |

## What it runs

```
straymark-baton coherence . --spec <active-feature> --min-confidence <min>
```

Scoped to the feature being implemented (from `.specify/feature.json`), so the
agent sees exactly the cross-spec contract drift relevant to the code it is about
to write.
