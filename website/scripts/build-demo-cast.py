#!/usr/bin/env python3
"""
Build the asciinema cast played on the homepage.

Inputs are baked into this file — outputs were captured from a real
straymark CLI run on 2026-05-20 against a freshly initialized demo
project, then condensed to keep the recording illustrative (under 40s,
no giant tables). The cast is regenerated on demand:

    python3 website/scripts/build-demo-cast.py

Output: website/static/asciinema/straymark-demo.cast
"""
import json
import os
import pathlib
import time

OUT = pathlib.Path(__file__).resolve().parent.parent / "static" / "asciinema" / "straymark-demo.cast"

# Tunables — keep the total under ~40s for the homepage.
TYPE_DELAY = 0.055        # seconds between keystrokes
LINE_DELAY = 0.06         # between consecutive output lines
COMMAND_PAUSE = 0.40      # between hitting Enter and first output
THINK_PAUSE = 1.40        # after each command's output completes
END_HOLD = 3.50           # final prompt sits on screen this long before the cast loops
IDLE_LIMIT = 4.0          # any idle > this gets clipped by the player; must exceed END_HOLD

GREEN = "[32m"
DIM = "[2m"
BOLD = "[1m"
CYAN = "[36m"
RESET = "[0m"
PROMPT = f"{GREEN}~/demo $ {RESET}"


class Cast:
    def __init__(self, width=90, height=26):
        self.events = []
        self.t = 0.0
        self.header = {
            "version": 2,
            "width": width,
            "height": height,
            "timestamp": int(time.time()),
            "title": "StrayMark — 30-second demo",
            "idle_time_limit": IDLE_LIMIT,
            "env": {"SHELL": "/bin/zsh", "TERM": "xterm-256color"},
        }

    def push(self, data: str, dt: float = 0.0):
        self.t += dt
        self.events.append([round(self.t, 3), "o", data])

    def prompt(self):
        self.push(PROMPT, 0.0)

    def type(self, text: str):
        for ch in text:
            self.push(ch, TYPE_DELAY)

    def enter(self):
        self.push("\r\n", TYPE_DELAY)

    def output(self, lines, head_pause: float = COMMAND_PAUSE):
        first = True
        for line in lines:
            self.push(line + "\r\n", head_pause if first else LINE_DELAY)
            first = False

    def pause(self, dt: float):
        self.t += dt

    def serialize(self) -> str:
        out = [json.dumps(self.header, ensure_ascii=False)]
        for e in self.events:
            out.append(json.dumps(e, ensure_ascii=False))
        return "\n".join(out) + "\n"


def build() -> str:
    c = Cast()

    # Frame 1 — init
    c.prompt()
    c.pause(0.35)
    c.type("straymark init")
    c.enter()
    c.output([
        f"{DIM}Initializing StrayMark in /tmp/demo{RESET}",
        "→ Fetching latest release...",
        f"  Found version: {CYAN}fw-4.17.0{RESET}",
        "→ Downloading...",
        "→ Extracting files...",
        "→ Configuring AI agent directives...",
        f"{GREEN}✓{RESET} Configured AGENTS.md",
        f"{GREEN}✓{RESET} Configured CLAUDE.md",
        f"{GREEN}✓{RESET} Configured GEMINI.md",
        f"{GREEN}✓{RESET} Configured .cursor/rules/straymark.md",
        "",
        f"{BOLD}{GREEN}✓ StrayMark initialized successfully!{RESET}",
    ])
    c.pause(THINK_PAUSE)

    # Frame 2 — first Charter
    c.prompt()
    c.pause(0.20)
    c.type('straymark charter new --title "Auth refactor" --type S')
    c.enter()
    c.output([
        "",
        f"{GREEN}✓{RESET} Created: .straymark/charters/01-auth-refactor.md",
        "",
        f"  {DIM}Next steps:{RESET}",
        "    1. Edit the Charter to fill in Context, Scope, Files, Risks, Tasks.",
        "    2. Set the trigger field in frontmatter to a concrete observable signal.",
        "    3. When you start executing: change status from `declared` to `in-progress`.",
    ])
    c.pause(THINK_PAUSE)

    # Frame 3 — peek at what the Charter looks like
    c.prompt()
    c.pause(0.20)
    c.type("head -10 .straymark/charters/01-auth-refactor.md")
    c.enter()
    c.output([
        "---",
        f"{CYAN}charter_id{RESET}: CHARTER-01-auth-refactor",
        f"{CYAN}status{RESET}: declared",
        f"{CYAN}effort_estimate{RESET}: S",
        f"{CYAN}trigger{RESET}: \"...observable signal that justifies executing now\"",
        "---",
        "",
        f"{BOLD}# Charter: Auth refactor{RESET}",
        "",
        f"{DIM}> Status (mirrored from frontmatter): declared. Effort: S.{RESET}",
    ])

    # Give the eye time to land on the last line, then drop back to a clean
    # prompt. The trailing no-op event extends the cast timeline so the
    # player holds the final frame for END_HOLD seconds before looping.
    c.pause(1.8)
    c.prompt()
    c.push("", END_HOLD)

    return c.serialize()


def main():
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(build())
    duration = json.loads(OUT.read_text().splitlines()[-1])[0]
    print(f"Wrote {OUT.relative_to(pathlib.Path.cwd())} ({duration:.1f}s, {len(OUT.read_text().splitlines()) - 1} events)")


if __name__ == "__main__":
    main()
