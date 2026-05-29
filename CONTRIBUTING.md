# Contributing to StrayMark

Thank you for your interest in contributing to StrayMark! This document provides guidelines and information for contributors.

**Languages**: English | [Español](docs/i18n/es/CONTRIBUTING.md) | [简体中文](docs/i18n/zh-CN/CONTRIBUTING.md)

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Contributor License Agreement (CLA)](#contributor-license-agreement-cla)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)
- [Documentation Standards](#documentation-standards)

---

## Code of Conduct

This project is governed by our [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

In short: be respectful, inclusive, and constructive in all interactions. Harassment, discrimination, and trolling are not tolerated. Please read the [full Code of Conduct](CODE_OF_CONDUCT.md) before contributing.

---

## Contributor License Agreement (CLA)

This project requires all contributors to sign a **Contributor License Agreement (CLA)** before their pull requests can be merged. We use [CLA Assistant](https://cla-assistant.io/) to manage this process.

### How it works

1. When you open your first pull request, CLA Assistant will automatically post a comment asking you to sign the CLA.
2. Click the link in the comment to review and sign the agreement.
3. The CLA only needs to be signed once — it covers all future contributions to this project.
4. Once signed, CLA Assistant will update the PR check status and your contribution can proceed to review.

If you have questions about the CLA, please open a [Discussion](https://github.com/StrangeDaysTech/straymark/discussions).

---

## How Can I Contribute?

### Reporting Bugs

Before creating a bug report, please check existing issues to avoid duplicates.

**When reporting a bug, include:**

- A clear, descriptive title
- Steps to reproduce the behavior
- Expected behavior
- Actual behavior
- Screenshots (if applicable)
- Environment details (OS, AI platform, etc.)

### Suggesting Features

Feature suggestions are welcome! Please include:

- A clear description of the feature
- The problem it solves
- Potential implementation approaches
- Any alternatives you've considered

### Improving Documentation

Documentation improvements are highly valued:

- Fix typos or unclear wording
- Add examples
- Improve explanations
- Translate to other languages

### Submitting Code

Code contributions should:

- Fix a bug or implement a feature
- Include appropriate tests (if applicable)
- Follow the project's style guidelines
- Update documentation as needed

---

## Development Setup

### Prerequisites

- **Git**
- **A text editor** (VS Code recommended)
- **Rust toolchain** (for CLI development — install via [rustup.rs](https://rustup.rs/))
- **Node.js 20+** (optional, for markdownlint)

### Setup Steps

1. **Fork the repository**

   Click "Fork" on the [GitHub repository page](https://github.com/StrangeDaysTech/straymark).

2. **Clone your fork**
   ```bash
   git clone https://github.com/your-username/straymark.git
   cd straymark
   ```

3. **Install the pre-commit hook**
   ```bash
   echo 'straymark validate --staged' > .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

4. **Install development tools (optional)**
   ```bash
   # Markdown linting
   npm install -g markdownlint-cli
   ```

5. **Create a branch**
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

6. **Make your changes and validate**
   ```bash
   straymark validate
   ```

---

## Pull Request Process

### Before Submitting

- [ ] Run `straymark validate` successfully
- [ ] Update documentation if needed
- [ ] Add yourself to CONTRIBUTORS.md (if applicable)
- [ ] Write a clear PR description

### PR Title Format

Use conventional commit format:

```
type(scope): description

Examples:
feat(templates): add template for security reviews
fix(validation): correct regex for file naming
docs(readme): clarify installation steps
chore(ci): update GitHub Actions workflow
```

**Types:**
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `chore` - Maintenance tasks
- `refactor` - Code refactoring
- `test` - Test additions or fixes

### PR Description Template

```markdown
## Summary
Brief description of changes

## Motivation
Why is this change needed?

## Changes
- Change 1
- Change 2

## Testing
How were these changes tested?

## Checklist
- [ ] `straymark validate` passes
- [ ] Documentation updated
- [ ] No sensitive information included
```

### Review Process

1. A maintainer will review your PR
2. Address any requested changes
3. Once approved, a maintainer will merge

---

## Style Guidelines

### Markdown

- Use ATX-style headers (`#`, `##`, etc.)
- Use fenced code blocks with language identifiers
- Use tables for structured data
- Keep lines under 120 characters when practical
- Use blank lines to separate sections

### YAML Front-matter

```yaml
---
id: TYPE-YYYY-MM-DD-NNN
title: Clear, descriptive title
status: draft | accepted | deprecated
created: YYYY-MM-DD
# Additional fields as needed
---
```

### File Naming

StrayMark documents:
```
[TYPE]-[YYYY-MM-DD]-[NNN]-[description].md
```

- Use lowercase for description
- Use hyphens to separate words
- Keep descriptions concise but clear

### Code in Scripts

- Use clear variable names
- Add comments for complex logic
- Follow shell/PowerShell best practices

---

## Documentation Standards

### Adding a New Document Type

If you're proposing a new document type:

1. **Create the template**
   - Add `TEMPLATE-NEWTYPE.md` to `dist/.straymark/templates/`
   - Follow existing template patterns

2. **Update governance docs**
   - `dist/.straymark/00-governance/DOCUMENTATION-POLICY.md`
   - `dist/.straymark/00-governance/AGENT-RULES.md`
   - `dist/.straymark/QUICK-REFERENCE.md`

3. **Update agent configs**
   - `dist/dist-templates/directives/` (distribution templates)

4. **Update validation**
   - Add the new type to the CLI validation logic (`cli/src/commands/validate.rs`)
   - `dist/.github/workflows/docs-validation.yml`

5. **Document the change**
   - Create an ADR explaining the new type
   - Update the README if needed

### Writing Templates

Templates should include:

- Complete YAML front-matter with all fields
- Clear section headers
- Placeholder text explaining what goes in each section
- Examples where helpful

### Writing Governance Documents

- Be clear and unambiguous
- Use tables for reference information
- Include examples
- Keep rules actionable

---

## CLI Development

The StrayMark CLI is written in Rust and located in the `cli/` directory.

### Building

```bash
cd cli
cargo build
```

### Running Tests

```bash
cd cli
cargo test
```

### Release Build

```bash
cd cli
cargo build --release
```

The release binary is optimized with LTO and stripped for minimal size.

### Architecture

```
cli/src/
├── main.rs              # Entry point + clap CLI definition
├── commands/
│   ├── mod.rs           # Subcommand routing
│   ├── init.rs          # straymark init [path]
│   ├── update.rs        # straymark update (combined)
│   ├── update_framework.rs # straymark update-framework
│   ├── update_cli.rs    # straymark update-cli
│   ├── remove.rs        # straymark remove [--full]
│   ├── status.rs        # straymark status [path]
│   └── about.rs         # straymark about
├── config.rs            # Config and checksums management
├── download.rs          # GitHub Releases API (prefix-filtered)
├── inject.rs            # Directive file injection (markers)
├── manifest.rs          # dist-manifest.yml parsing
├── platform.rs          # OS/arch detection for binary downloads
├── self_update.rs       # CLI binary self-update logic
└── utils.rs             # Helpers (hashing, colors, paths)
```

> **Note**: Framework and CLI use independent versioning (`fw-*` and `cli-*` tags). See [CLI Reference](docs/adopters/CLI-REFERENCE.md#versioning) for details.

---

## Questions?

If you have questions about contributing:

1. Check existing [Issues](https://github.com/StrangeDaysTech/straymark/issues)
2. Check [Discussions](https://github.com/StrangeDaysTech/straymark/discussions)
3. Open a new Discussion for general questions
4. Open an Issue for specific bugs or features

### Adopting StrayMark?

If you're using StrayMark on a real project and want to send telemetry and findings upstream:

1. **Announce it** in the [Adopters discussion category](https://github.com/StrangeDaysTech/straymark/discussions/new?category=adopters) — this records your adoption and what you commit to share.
2. **File findings** as Issues using the *Adopter feedback / upstream finding* template, cross-linked to your adoption discussion.
3. See the [Adopter Feedback guide](docs/adopters/ADOPTER-FEEDBACK.md) for the full flow and the [adopters registry](ADOPTERS.md).

---

## Recognition

Contributors are recognized in:

- GitHub's contributor graph
- Release notes for significant contributions
- CONTRIBUTORS.md (for recurring contributors)

Thank you for helping make StrayMark better!

---

*StrayMark — Because every change tells a story.*

[Strange Days Tech](https://strangedays.tech)
