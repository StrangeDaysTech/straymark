---
name: straymark-ailog
description: Create an AILOG (AI Action Log) document for the current changes. Quick shortcut for the most common document type.
---

# StrayMark AILOG Skill

Quickly create an AI Action Log (AILOG) document for the current changes.

## Instructions

This is a shortcut skill that creates AILOG documents directly.

### 1. Gather Context

```bash
# Get current date
date +%Y-%m-%d

# Get modified files
git status --porcelain

# Summarize the CURRENT work (staged + unstaged + untracked) — label each block.
# Avoid `HEAD~1`: it summarizes the previous commit, not the work being logged.
git diff --cached --stat        # staged changes
git diff --stat                 # unstaged changes
git status --porcelain          # includes untracked files
```

### 2. Confirm with User

**Always confirm before creating:**

```
╔══════════════════════════════════════════════════════════════════╗
║  StrayMark AILOG                                                  ║
╠══════════════════════════════════════════════════════════════════╣
║                                                                   ║
║  📊 Changes detected:                                             ║
║     • Files: [list of modified files]                             ║
║     • Lines: [+X / -Y]                                            ║
║                                                                   ║
║  📝 Will create:                                                  ║
║     AILOG-YYYY-MM-DD-NNN-[description].md                         ║
║                                                                   ║
║  Please provide a brief description of what was done:             ║
╚══════════════════════════════════════════════════════════════════╝
```

### 3. Determine Sequence Number

```bash
# Count existing AILOGs for today
ls .straymark/07-ai-audit/agent-logs/AILOG-$(date +%Y-%m-%d)-*.md 2>/dev/null | wc -l
```

Next number = count + 1, formatted as 3 digits (001, 002, etc.)

### 4. Check Language and Load Template

Read `.straymark/config.yml` for language setting:
- `en` (default): `.straymark/templates/TEMPLATE-AILOG.md`
- `es`: `.straymark/templates/i18n/es/TEMPLATE-AILOG.md`

### 5. Create Document

Fill template with:
- `id`: AILOG-YYYY-MM-DD-NNN
- `title`: User-provided description
- `created`: Current date
- `agent`: your runtime's canonical agent identity (see AGENT-RULES.md §1 — e.g. `claude-code-v1.0`, `gemini-cli-v1.0`, `codex-cli-v1.0`; do not assume Claude)
- `confidence`: based on change complexity
- `risk_level`: based on files modified

Save to: `.straymark/07-ai-audit/agent-logs/AILOG-YYYY-MM-DD-NNN-description.md`

### 6. Report Result

```
✅ AILOG created:
   .straymark/07-ai-audit/agent-logs/AILOG-YYYY-MM-DD-NNN-description.md

StrayMark: Created AILOG-YYYY-MM-DD-NNN-description.md
```

## Risk Level Guidelines

| Indicator | Risk Level |
|-----------|------------|
| Config/settings changes | low |
| Business logic changes | medium |
| Auth, security, payments | high |
| Database schema, migrations | critical |
