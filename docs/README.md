# Documentation

This directory contains StrayMark's user-facing documentation, organized by audience.

## Structure

```
docs/
├── adopters/           # For teams adopting StrayMark
│   ├── ADOPTION-GUIDE.md
│   ├── CLI-REFERENCE.md
│   └── WORKFLOWS.md
├── contributors/       # For developers contributing to StrayMark
│   └── TRANSLATION-GUIDE.md
└── i18n/               # Translations
    └── es/             # Spanish
        ├── README.md
        ├── CONTRIBUTING.md
        ├── CODE_OF_CONDUCT.md
        └── adopters/
            └── ADOPTION-GUIDE.md
```

## Versioning

StrayMark uses independent version tags: `fw-*` for framework releases and `cli-*` for CLI releases. See the [CLI Reference](adopters/CLI-REFERENCE.md#versioning) for details.

## For Adopters

| Document | Description |
|----------|-------------|
| [ADOPTION-GUIDE.md](adopters/ADOPTION-GUIDE.md) | Step-by-step guide for adopting StrayMark in new or existing projects |
| [CLI-REFERENCE.md](adopters/CLI-REFERENCE.md) | Complete CLI command reference with flags, arguments, and examples |
| [WORKFLOWS.md](adopters/WORKFLOWS.md) | Recommended daily workflows, update cadences, and team patterns |

## For Contributors

| Document | Description |
|----------|-------------|
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Development setup, PR process, style guidelines |
| [TRANSLATION-GUIDE.md](contributors/TRANSLATION-GUIDE.md) | Rules and guidelines for translating StrayMark documentation |

## Key References

| Document | Description |
|----------|-------------|
| [QUICK-REFERENCE.md](../dist/.straymark/QUICK-REFERENCE.md) | One-page cheat sheet: document types, naming, metadata |
| [STRAYMARK.md](../dist/STRAYMARK.md) | Unified governance rules (source of truth) |

## Translations

| Language | Documents |
|----------|-----------|
| [Español](i18n/es/README.md) | README, CONTRIBUTING, CODE_OF_CONDUCT, ADOPTION-GUIDE |

---

> **Note**: Development governance documentation (policies, templates, agent rules) lives in [`dist/.straymark/`](../dist/.straymark/) — not in this directory. See [QUICK-REFERENCE.md](../dist/.straymark/QUICK-REFERENCE.md) for an overview.
