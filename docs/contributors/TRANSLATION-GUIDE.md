# Translation Guide - StrayMark

This guide provides rules and guidelines for translating StrayMark documentation to other languages.

---

## Table of Contents

1. [Language Configuration](#language-configuration)
2. [Translation Scope](#translation-scope)
3. [What to Translate](#what-to-translate)
4. [What NEVER to Translate](#what-never-to-translate)
5. [File Structure](#file-structure)
6. [Translation Process](#translation-process)
7. [Quality Checklist](#quality-checklist)

---

## Language Configuration

StrayMark uses a configuration file to set the project's language:

**File**: `.straymark/config.yml`

```yaml
# Language setting for templates and documentation
# Supported values: en, es, zh-CN
# Default: en (English)
language: en
```

### How It Works

1. AI agents read `.straymark/config.yml` at the start of each session
2. Based on the `language` value, they use templates from the appropriate path:

| Language | Template Path |
| --- | --- |
| `en` (default) | `.straymark/templates/TEMPLATE-*.md` |
| `es` | `.straymark/templates/i18n/es/TEMPLATE-*.md` |
| `zh-CN` | `.straymark/templates/i18n/zh-CN/TEMPLATE-*.md` |

3. If the config file doesn't exist or `language` is not set, English is used as default

### Changing the Language

To switch to Spanish:

```yaml
language: es
```

The AI agent will then use Spanish templates when creating documentation.

---

## Translation Scope

### Files to Translate

| Category | Files | Location |
| --- | --- | --- |
| **Main Documentation** | README.md, CONTRIBUTING.md, CODE_OF_CONDUCT.md | `docs/i18n/{lang}/` |
| **Adopter Documentation** | ADOPTION-GUIDE.md, CLI-REFERENCE.md, WORKFLOWS.md | `docs/i18n/{lang}/adopters/` |
| **Templates** | TEMPLATE-*.md (8 files) | `.straymark/templates/i18n/{lang}/` |
| **Governance** | PRINCIPLES.md, DOCUMENTATION-POLICY.md, AGENT-RULES.md, GIT-BRANCHING-STRATEGY.md, QUICK-REFERENCE.md | `.straymark/00-governance/i18n/{lang}/` |

### Files NOT to Translate

| File | Reason |
| --- | --- |
| `CLAUDE.md` | AI agent configuration - works identically in any language |
| `GEMINI.md` | AI agent configuration |
| `.cursorrules` | AI agent configuration |
| `.github/copilot-instructions.md` | AI agent configuration |
| Validation scripts | Technical code |
| GitHub Actions workflows | Technical code |

---

## What to Translate

### Headers and Section Titles

```markdown
<!-- English -->
## Summary

<!-- Spanish -->
## Resumen
```

### Descriptive Text and Explanations

```markdown
<!-- English -->
[Brief description of what was done and why]

<!-- Spanish -->
[Descripción breve de lo realizado y por qué]
```

### Table Headers (descriptive columns only)

```markdown
<!-- English -->
| File | Change |

<!-- Spanish -->
| Archivo | Cambio |
```

### Comments and Instructions in Templates

```markdown
<!-- English -->
> **IMPORTANT**: This document requires human review.

<!-- Spanish -->
> **IMPORTANTE**: Este documento requiere revisión humana.
```

---

## What NEVER to Translate

### 1. Document Type Prefixes

These are technical identifiers that must remain in English:

```
AILOG    AIDEC    ADR    ETH    REQ    TES    INC    TDE
```

**Correct:**
```markdown
# AILOG: Implementación de autenticación OAuth
```

**Incorrect:**
```markdown
# REGAI: Implementación de autenticación OAuth  ❌
```

### 2. YAML Keys (Front-matter)

All YAML keys must remain in English:

```yaml
---
id: AILOG-2025-01-27-001        # ✓ Keep as-is
title: Título en español        # ✓ Translate value only
status: accepted                # ✓ Keep as-is (enum value)
created: 2025-01-27             # ✓ Keep as-is
agent: claude-code-v1.0         # ✓ Keep as-is
confidence: high                # ✓ Keep as-is (enum value)
review_required: true           # ✓ Keep as-is
risk_level: medium              # ✓ Keep as-is (enum value)
tags:
  - seguridad                   # ✓ Translate tag values
  - autenticación
---
```

### 3. Enumerated Values

Keep these values in English:

| Category | Values |
| --- | --- |
| **Status** | `draft`, `accepted`, `deprecated`, `superseded` |
| **Confidence** | `high`, `medium`, `low` |
| **Risk Level** | `low`, `medium`, `high`, `critical` |
| **Severity** | `SEV1`, `SEV2`, `SEV3`, `SEV4` |
| **Priority** | `must`, `should`, `could`, `wont` |
| **Debt Type** | `code`, `architecture`, `infrastructure`, `documentation`, `testing` |
| **Requirement Type** | `functional`, `non-functional`, `constraint` |

### 4. File Paths and Directory Names

```markdown
<!-- Keep paths exactly as shown -->
.straymark/
.straymark/07-ai-audit/agent-logs/
.straymark/templates/TEMPLATE-AILOG.md
```

### 5. File Naming Convention Pattern

```
[TYPE]-[YYYY-MM-DD]-[NNN]-[description].md
```

The pattern itself and examples should keep the format, though `description` can be in the target language when creating actual documents.

### 6. Agent Identifiers

```yaml
agent: claude-code-v1.0    # ✓ Never translate
agent: cursor-v1.0         # ✓ Never translate
agent: gemini-cli-v1.0     # ✓ Never translate
```

### 7. Technical Terms in Code Blocks

```yaml
# Keep technical terms
confidence: high | medium | low    # Do not translate options
risk_level: low | medium | high | critical
```

### 8. Cross-References

```markdown
See [ADR-2025-01-20-001] for details.    # Keep reference format
Basado en [REQ-2025-01-15-003].          # Keep reference format
```

### 9. URLs and Links

```markdown
[Strange Days Tech](https://strangedays.tech)    # Keep URLs exactly
```

### 10. Brand Names

```
Strange Days Tech
StrayMark
Claude Code
Cursor
GitHub Copilot
Gemini CLI
```

---

## File Structure

### Directory Layout

```
straymark/
├── docs/
│   ├── adopters/
│   │   └── ADOPTION-GUIDE.md        # English adopter docs
│   ├── contributors/
│   │   └── TRANSLATION-GUIDE.md     # This file
│   └── i18n/
│       ├── es/                      # Spanish translations
│       │   ├── README.md
│       │   ├── CONTRIBUTING.md
│       │   ├── CODE_OF_CONDUCT.md
│       │   └── adopters/
│       │       └── ADOPTION-GUIDE.md
│       ├── zh-CN/                   # Chinese Simplified translations
│       │   ├── README.md
│       │   ├── CONTRIBUTING.md
│       │   ├── CODE_OF_CONDUCT.md
│       │   └── adopters/
│       │       └── ADOPTION-GUIDE.md
│       └── [other-lang]/            # Future languages
│
└── .straymark/
    ├── templates/
    │   ├── TEMPLATE-*.md            # English (default)
    │   └── i18n/
    │       ├── es/
    │       │   └── TEMPLATE-*.md    # Spanish templates
    │       └── zh-CN/
    │           └── TEMPLATE-*.md    # Chinese Simplified templates
    │
    └── 00-governance/
        ├── *.md                     # English (default)
        └── i18n/
            ├── es/
            │   └── *.md             # Spanish governance docs
            └── zh-CN/
                └── *.md             # Chinese Simplified governance docs
```

### File Naming

Translated files keep the same filename as the original:

| Original | Translation |
| --- | --- |
| `README.md` | `docs/i18n/es/README.md` |
| `docs/adopters/ADOPTION-GUIDE.md` | `docs/i18n/es/adopters/ADOPTION-GUIDE.md` |
| `TEMPLATE-AILOG.md` | `.straymark/templates/i18n/es/TEMPLATE-AILOG.md` |

---

## Translation Process

### Step 1: Copy the Original

```bash
cp README.md docs/i18n/es/README.md
```

### Step 2: Translate Content

1. Translate headers and section titles
2. Translate descriptive text
3. Translate table headers (descriptive columns)
4. Keep all technical tokens unchanged
5. Update internal links to point to translated versions where available

### Step 3: Verify Technical Tokens

Run through the checklist below to ensure no technical tokens were translated.

### Step 4: Test Links

Verify all internal links work correctly.

---

## Quality Checklist

Before submitting a translation, verify:

### Technical Tokens

- [ ] Document type prefixes unchanged (AILOG, AIDEC, ADR, etc.)
- [ ] YAML keys unchanged (id, title, status, agent, confidence, etc.)
- [ ] Enum values unchanged (draft, accepted, high, medium, low, etc.)
- [ ] File paths unchanged (.straymark/, 07-ai-audit/, etc.)
- [ ] Agent identifiers unchanged (claude-code-v1.0, etc.)
- [ ] Cross-reference format unchanged ([ADR-YYYY-MM-DD-NNN])

### Content Quality

- [ ] All headers translated
- [ ] All descriptive text translated
- [ ] Technical accuracy maintained
- [ ] Consistent terminology throughout
- [ ] No machine translation artifacts

### Structure

- [ ] Same sections as original
- [ ] Same table structures
- [ ] Same code block formatting
- [ ] Links updated appropriately

---

## Supported Languages

| Code | Language | Status |
| --- | --- | --- |
| `es` | Spanish (Español) | Available |
| `pt` | Portuguese (Português) | Planned |
| `zh-CN` | Chinese Simplified (简体中文) | Available |

---

## Contributing Translations

To contribute a new translation:

1. Create the directory structure for your language
2. Translate files following this guide
3. Submit a pull request with all translated files
4. Include the language code in PR title: `i18n(XX): Add [Language] translation`

See [CONTRIBUTING.md](./../../CONTRIBUTING.md) for general contribution guidelines.

---

*StrayMark — Because every change tells a story.*

[Strange Days Tech](https://strangedays.tech)
