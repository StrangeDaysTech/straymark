---
name: straymark-loom
description: Drive the Loom visualization server lifecycle from the agent window — up / down / status. Launches `straymark loom serve` in the background and hands the operator a link to click, so the terminal-free user never opens a shell. EXPERIMENTAL (Loom v0).
---

# StrayMark Loom Skill

Own the Loom server process lifecycle so a terminal-free operator stays in the chat window. Loom is StrayMark's loopback-only, read-only development dashboard (Knowledge Graph + Architecture Map). This skill starts it, stops it, and reports its status — the operator just gets a URL to click.

> ⚠️ **EXPERIMENTAL.** Loom is an opt-in Loom v0 experiment — loopback-only, read-only, not part of the supported Framework/CLI contract. The binary is downloaded on demand from the `loom-*` GitHub releases and cached in `~/.straymark/bin/` on first use. See `docs/adopters/LOOM.md`.

## When to use this skill

Trigger on any of:

- The operator asks to "open Loom", "show the architecture map", "visualize the project", or "start/stop the dashboard".
- A `/straymark-architecture` refinement just finished and the operator wants to see the 2D/3D overlay.
- The operator asks whether Loom is running / on which port.

The Architecture Map view is only meaningful once `.straymark/architecture/model.yml` is refined — if it is still a raw seed (every component in `unassigned`), suggest `/straymark-architecture` first. The Knowledge Graph view works with no setup.

## Instructions

The default port is **7700** on `127.0.0.1`. Pass `--port N` to override.

### up — launch in the background

```bash
straymark loom serve --no-open &     # background; --no-open so no browser is spawned from the agent
# (use --port N to override the default 7700)
```

Run with `--no-open` (the agent owns the process, not a browser) and in the background so the chat stays responsive. On **first use** the `loom-*` binary downloads to `~/.straymark/bin/` — tell the operator this is opt-in by download and may take a moment. Once it is listening, hand the operator the link:

```bash
sleep 1 && curl -sI http://127.0.0.1:7700/ | head -1   # confirm it's up (expect HTTP 200)
```

Report: **`Loom is up → http://127.0.0.1:7700`** (Architecture tab for the map, `2D | 3D` toggle for the axonometric view).

### down — stop it

```bash
pkill -f 'straymark-loom' || pkill -f 'loom serve'
```

Confirm the port is free (`curl` fails / `lsof` empty) and report **`Loom stopped.`**

### status — running? which port?

```bash
ps aux | grep -E 'straymark-loom|loom serve' | grep -v grep   # process + port flag
lsof -iTCP -sTCP:LISTEN -P 2>/dev/null | grep -E '7700|loom'   # confirm the listening port
curl -sI http://127.0.0.1:7700/ | head -1                      # liveness on the default port
```

Report whether Loom is running and the URL, or **`Loom is not running.`**

## Report result

Surface the URL plainly so the operator can click it. Example:

```
✓ Loom is up → http://127.0.0.1:7700
  Architecture tab = the "you are here" map; 2D | 3D toggle = axonometric view.
  Terminal-only alternative (no server): `straymark status --where`.
```

## What this skill does NOT do

- **It does not open a browser from the agent.** It always runs `loom serve --no-open` and reports the URL; the operator clicks it.
- **It does not write into the project.** Loom is read-only — it only visualizes what the CLI computes. The CLI (`validate`, `audit`, `charter drift`) stays the source of truth.
- **It does not author the architecture model.** Use `/straymark-architecture` to generate/refine it and `/straymark-architecture-sync` to keep it current.

> **Terminal compatibility**: If the terminal does not support box-drawing characters (Unicode), use plain-text formatting with dashes and pipes instead.
