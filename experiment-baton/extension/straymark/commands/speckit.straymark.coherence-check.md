---
description: "Read-only coherence check before implementing — surfaces cross-spec contract drift (issue #304)"
---

# StrayMark Coherence Check

Run the Baton coherence engine, scoped to the active feature, **before**
implementation begins — so a consumer is never coded against a contract that a
decision elsewhere already changed (the issue #304 pattern).

## Behavior

Invoked as the `before_implement` hook. It:

1. Reads `.specify/extensions/straymark/straymark-config.yml` for `gate`
   (`advisory` | `block`), `min_confidence`, and an optional `binary` path.
2. Discovers the `straymark-baton` binary: config `binary:` → `PATH` →
   `cargo run` inside `$BATON_REPO` (development).
3. Resolves the active feature from `.specify/feature.json` and runs, read-only:
   `straymark-baton coherence . --spec <feature> --min-confidence <min>`.
4. Surfaces the findings. With `gate: advisory` (default) the flow always
   continues; with `gate: block` it fails `before_implement` when blocking
   findings exist.

## Execution

Determine the event name from the hook that triggered this command, then run:

- **Bash**: `.specify/extensions/straymark/scripts/bash/coherence-check.sh <event_name>`

Replace `<event_name>` with the actual hook event (normally `before_implement`).

## Configuration

In `.specify/extensions/straymark/straymark-config.yml`:

```yaml
gate: advisory          # advisory | block
min_confidence: medium  # low | medium | high
binary: ""              # path to straymark-baton; empty → PATH, then cargo/$BATON_REPO
```

## Graceful Degradation

- If `straymark-baton` is not found: skips with a note — **the SpecKit flow
  continues** (the hook never breaks `implement`).
- If no active feature is detected: runs repo-wide instead of feature-scoped.
- Always read-only: the check never mutates the repository.
