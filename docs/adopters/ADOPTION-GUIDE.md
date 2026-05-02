# DevTrail - Adoption Guide

**A comprehensive guide for adopting DevTrail in new or existing projects.**

[![Strange Days Tech](https://img.shields.io/badge/by-Strange_Days_Tech-purple.svg)](https://strangedays.tech)

**Languages**: English | [Español](../i18n/es/adopters/ADOPTION-GUIDE.md) | [简体中文](../i18n/zh-CN/adopters/ADOPTION-GUIDE.md)

---

## Table of Contents

1. [What is DevTrail?](#what-is-devtrail-framework)
2. [Who is it for?](#who-is-it-for)
3. [Benefits](#benefits)
4. [Standards Compliance](#standards-compliance)
5. [Adoption Path A: New Projects](#adoption-path-a-new-projects)
6. [Adoption Path B: Existing Projects](#adoption-path-b-existing-projects)
7. [Configuration](#configuration)
8. [Verification](#verification)
9. [FAQ](#faq)

---

## What is DevTrail?

DevTrail is an **ISO 42001-aligned AI governance platform** for software development teams. It provides:

- **12 structured document types** covering the full development and AI lifecycle
- **Regulatory compliance automation** — EU AI Act, ISO 42001, NIST AI RMF scoring and audit trails
- **AI agent accountability** through mandatory identification, confidence tracking, and autonomy limits
- **Human oversight** via required review workflows for critical and high-risk changes
- **Traceability** connecting requirements → design → implementation → testing → incidents

### Core Principle

> **"No significant change without a documented trace — and proof of governance."**

DevTrail ensures that every meaningful change — whether by human or AI — is documented, attributed, and auditable. Teams that adopt DevTrail produce evidence compatible with **ISO/IEC 42001 certification** and **EU AI Act compliance**.

### Why Now?

The **EU AI Act becomes mandatory in August 2026**. ISO/IEC 42001 is the international standard for AI Management Systems. Organizations using AI in development need documented governance — not as a nice-to-have, but as a regulatory requirement. DevTrail operationalizes these requirements from day one.

### What DevTrail is NOT

- It is not a documentation generator — it provides structure, templates, and governance rules
- It is not a replacement for code comments or API docs
- It is not a project management tool or version control system
- It is not a full ISO 42001 implementation — it produces compatible evidence within its scope

---

## Who is it for?

### Target Users

| User Type | Adoption Drivers |
|-----------|-----------------|
| **Teams using AI coding assistants** | Prove governance for regulatory audits and quality assurance |
| **High-risk AI systems** | EU AI Act mandates documented risk management and transparency |
| **Organizations seeking ISO 42001** | DevTrail produces certification-ready evidence |
| **Regulated industries** (finance, healthcare, EU-based) | Regulatory compliance mandatory by August 2026 |
| **Solo Developers** | Track decisions and AI-assisted changes with structure |
| **Open Source Maintainers** | Document contribution decisions transparently |

### Compatible Development Environments

DevTrail provides configuration files for:

| Platform | Configuration File | Status |
|----------|-------------------|--------|
| **Claude Code** (Anthropic) | `CLAUDE.md` | ✅ Supported |
| **Cursor** | `.cursorrules` | ✅ Supported |
| **GitHub Copilot CLI** | `.github/copilot-instructions.md` | ✅ Supported |
| **Gemini CLI** (Google) | `GEMINI.md` | ✅ Supported |
| **Other AI Tools** | Copy rules from any config file | ✅ Adaptable |

### Compatible Methodologies

DevTrail works with any development methodology:

| Methodology | How DevTrail Fits |
|-------------|-------------------|
| **Agile/Scrum** | REQ documents map to user stories; ADRs capture sprint decisions |
| **Waterfall** | Full traceability from requirements through implementation |
| **DevOps/SRE** | INC documents for post-mortems; TDE for technical debt tracking |
| **Domain-Driven Design** | ADRs document bounded context decisions |
| **Test-Driven Development** | TES documents capture test strategies |

---

## Benefits

### For Regulatory Compliance

| Benefit | Description |
|---------|-------------|
| **EU AI Act Ready** | Risk classification, incident reporting, and transparency templates built in |
| **ISO 42001 Compatible** | Documentation structure aligns with certification audit requirements |
| **NIST AI RMF Mapped** | 12 GenAI risk categories and governance functions explicitly covered |
| **Audit Trail Complete** | `devtrail audit` generates exportable timeline and traceability reports |
| **Compliance Scoring** | `devtrail compliance` provides percentage-based regulatory gap analysis |

### For Development Teams

| Benefit | Description |
|---------|-------------|
| **Institutional Memory** | Decisions survive team member changes |
| **Onboarding Acceleration** | New members understand "why" through ADRs and AIDECs |
| **Reduced Rework** | Context preserved prevents repeated mistakes |
| **Clear Accountability** | Know who (or what) made each change |

### For AI-Assisted Development

| Benefit | Description |
|---------|-------------|
| **AI Transparency** | Every AI action is logged with confidence levels |
| **Human Oversight** | Critical decisions require human approval |
| **Ethical Safeguards** | ETH and DPIA documents ensure responsible AI use |
| **Governance Metrics** | `devtrail metrics` tracks review rates, risk distribution, and trends |

---

## Standards Compliance

DevTrail aligns with and supports compliance for:

### Software Engineering Standards

| Standard | How DevTrail Helps |
|----------|-------------------|
| **ISO/IEC/IEEE 29148:2018** | REQ documents follow structured requirements format with external interfaces and traceability |
| **ISO/IEC 25010:2023** | 9 quality characteristics evaluated in ADRs and REQ non-functional requirements |
| **ISO/IEC/IEEE 29119-3:2021** | TES documents follow test documentation hierarchy (Policy → Strategy → Plan) |
| **ISO/IEC 12207** | Lifecycle documentation coverage |

### AI Management & Governance

| Standard | How DevTrail Helps |
|----------|-------------------|
| **ISO/IEC 42001:2023** | Vertebral standard — AI-GOVERNANCE-POLICY.md maps all Annex A controls to DevTrail documents |
| **EU AI Act** | Risk classification in ETH, incident reporting timelines in INC, regulatory fields in AILOG |
| **NIST AI RMF 1.0 / 600-1** | 12 GenAI risk categories in ETH/AILOG, MAP/MEASURE/MANAGE/GOVERN coverage |
| **ISO/IEC 23894:2023** | AI risk management aligned with ETH and AI-RISK-CATALOG (Fase 3) |
| **GDPR** | ETH documents with GDPR Legal Basis section, DPIA for privacy impact assessments |

### China Regulatory Coverage *(opt-in via `regional_scope: china`)*

| Standard | How DevTrail Helps |
|----------|-------------------|
| **TC260 AI Safety Governance Framework v2.0** | Five-level risk grading (TC260RA), `tc260_*` fields on ETH/MCARD/AILOG/SEC |
| **PIPL — Personal Information Protection Law** | PIPIA template with PIPL Art. 55-56 sections, `pipl_*` fields, retention ≥ 3 years (`TYPE-003`) |
| **GB 45438-2025** *(mandatory)* | AILABEL template covering explicit + implicit labeling for generative content |
| **CAC Algorithm Filing** | CACFILE template with single + dual filing tracking; cross-checks from MCARD via `cac_filing_required` |
| **GB/T 45652-2025** | Training-data security section in SBOM and MCARD; `gb45652_training_data_compliance` field |
| **CSL 2026** | INC extension with `csl_severity_level`, deadline-coherence rules (1h / 4h+72h+30d windows) |

> Activate by adding `china` to `regional_scope` in `.devtrail/config.yml`. See the *Configuration* section below and the installed `CHINA-REGULATORY-FRAMEWORK.md` guide for details.

### Architecture Documentation

| Standard | How DevTrail Helps |
|----------|-------------------|
| **ADR (Architecture Decision Records)** | Native ADR support with extended metadata and immutability rules |
| **arc42** | ADRs complement arc42 decision documentation |
| **C4 Model** | ADRs document decisions at each C4 level (C4-DIAGRAM-GUIDE in Fase 4) |

### Compliance & Governance

| Regulation | How DevTrail Helps |
|------------|-------------------|
| **GDPR** | ETH documents for privacy, DPIA for data protection impact |
| **SOC 2** | Change documentation and access logging via AILOG |
| **ISO 27001** | Security decision documentation via SEC assessments |
| **HIPAA** | Audit trails for healthcare applications |

### Observability (Optional)

| Standard | How DevTrail Helps |
|----------|-------------------|
| **OpenTelemetry** | Optional observability sections in REQ, TES, INC; tag `observabilidad` for instrumentation changes |

---

## Adoption Path A: New Projects

### Option 1: CLI (Recommended)

**Quick install (prebuilt binary):**

```bash
# Linux / macOS
curl -fsSL https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.sh | sh
```

```powershell
# Windows (PowerShell)
irm https://raw.githubusercontent.com/StrangeDaysTech/devtrail/main/install.ps1 | iex
```

Or install from source with Cargo:

```bash
cargo install devtrail-cli
```

Then initialize and commit:

```bash
cd your-project
devtrail init .

git add .devtrail/ DEVTRAIL.md
git commit -m "chore: adopt DevTrail"
```

The CLI automatically:
- Downloads the latest DevTrail release from GitHub
- Sets up the `.devtrail/` directory structure
- Creates `DEVTRAIL.md` with governance rules
- Configures AI agent directives (`CLAUDE.md`, `GEMINI.md`, `.cursorrules`, etc.)
- Copies CI/CD workflows

### Option 2: Manual Setup

1. **Download the latest release**

   Go to [GitHub Releases](https://github.com/StrangeDaysTech/devtrail/releases) and download the latest `fw-*` release ZIP (e.g., `fw-4.4.1`).

2. **Extract to your project**
   ```bash
   unzip devtrail-fw-*.zip -d your-project/
   ```

3. **Commit the structure**
   ```bash
   git add .devtrail/ DEVTRAIL.md
   git commit -m "chore: adopt DevTrail for documentation governance"
   ```

---

## Adoption Path B: Existing Projects

### Phase 1: Assessment (Day 1)

1. **Evaluate current documentation**

   Answer these questions:
   - Do you have existing ADRs? Where are they located?
   - Do you have a `docs/` folder? What does it contain?
   - Are there any naming conventions already in place?
   - Do you use any AI coding assistants?

2. **Plan the migration**

   | Current State | Recommended Action |
   |---------------|-------------------|
   | No documentation | Start fresh with DevTrail |
   | Docs in `docs/` folder | Keep `docs/` for user-facing docs, add `.devtrail/` for development docs |
   | Existing ADRs | Migrate to `.devtrail/02-design/decisions/` with new naming |
   | Mixed documentation | Categorize and migrate gradually |

### Phase 2: Installation (Day 1-2)

1. **Add DevTrail structure**
   ```bash
   # Using CLI (recommended)
   devtrail init .

   # Or manually: download the latest fw-* release from GitHub Releases
   # https://github.com/StrangeDaysTech/devtrail/releases
   ```

2. **Resolve conflicts with existing `docs/`**

   DevTrail uses `.devtrail/` specifically to avoid conflicts:

   ```
   your-project/
   ├── docs/                    ← Keep for API docs, user guides, etc.
   │   ├── api/
   │   └── user-guide/
   ├── .devtrail/              ← Add for development documentation
   │   ├── 00-governance/
   │   ├── 01-requirements/
   │   └── ...
   └── src/
   ```

### Phase 3: Migration (Week 1-2)

1. **Migrate existing ADRs**

   For each existing ADR:
   ```bash
   # Old: docs/adr/001-use-postgresql.md
   # New: .devtrail/02-design/decisions/ADR-2024-01-15-001-use-postgresql.md
   ```

   Add DevTrail metadata to the front-matter:
   ```yaml
   ---
   id: ADR-2024-01-15-001
   title: Use PostgreSQL for primary database
   status: accepted
   created: 2024-01-15
   agent: human
   confidence: high
   review_required: false
   risk_level: high
   # Preserve original metadata
   original_id: "001"
   migrated_from: "docs/adr/001-use-postgresql.md"
   ---
   ```

2. **Document the migration**

   Create an AILOG documenting the migration:
   ```
   .devtrail/07-ai-audit/agent-logs/AILOG-2025-01-27-001-devtrail-adoption.md
   ```

### Phase 4: Team Adoption (Week 2-4)

1. **Update contribution guidelines**

   Add to your `CONTRIBUTING.md`:
   ```markdown
   ## Documentation

   This project uses [DevTrail](https://github.com/StrangeDaysTech/devtrail) for documentation governance.

   - All significant changes must be documented in `.devtrail/`
   - AI-assisted changes require AILOG entries
   - Architectural decisions require ADR documents

   See `.devtrail/QUICK-REFERENCE.md` for document types and naming.
   ```

2. **Enable pre-commit hooks (optional)**
   ```bash
   # Install the pre-commit hook
   echo 'devtrail validate --staged' > .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit

   # Or with Husky
   npx husky add .husky/pre-commit "devtrail validate --staged"
   ```

3. **Enable GitHub Actions (optional)**

   The workflow at `.github/workflows/docs-validation.yml` will automatically validate documentation on PRs.

### Phase 5: Gradual Rollout

| Week | Focus |
|------|-------|
| Week 1 | Core team adopts DevTrail for new decisions |
| Week 2 | Migrate critical existing ADRs |
| Week 3 | Enable validation in CI/CD |
| Week 4 | Full team adoption; document existing technical debt |

---

## Configuration

### Regional Regulatory Scope

`.devtrail/config.yml` controls which compliance frameworks are evaluated by `devtrail compliance` and which document types are exposed by `devtrail new`:

```yaml
regional_scope:
  - global   # NIST AI RMF + ISO/IEC 42001 (always recommended)
  - eu       # EU AI Act + GDPR
  - china    # TC260 v2.0, PIPL/PIPIA, GB 45438, CAC, GB/T 45652, CSL 2026
```

**Default if omitted**: `[global, eu]` — preserves the behavior of every DevTrail version prior to `fw-4.3.0`.

When you add `china` to the list:

- Four China-specific document types become available via `devtrail new`: `PIPIA`, `CACFILE`, `TC260RA`, `AILABEL`.
- Six new compliance checkers run with `devtrail compliance` (or via `--region china` / `--standard china-*`).
- Twelve scope-aware validation rules activate (`CROSS-004` to `CROSS-011`, `TYPE-003` to `TYPE-006`).
- Five new governance guides are referenced from `.devtrail/00-governance/` (`CHINA-REGULATORY-FRAMEWORK.md`, plus per-framework guides for TC260, PIPL/PIPIA, CAC filing, and GB 45438 labeling) — all available in EN, ES, and zh-CN.

A project without `china` in `regional_scope` is unaffected: no new files, no new prompts, no new rules. Adding `china` later is fully reversible.

### Customizing Agent Identifiers

Each AI platform has its own configuration file that:

1. Identifies the agent (e.g., `claude-code-v1.0`)
2. Defines when to document (>10 lines, security changes, etc.)
3. Sets autonomy limits
4. Specifies templates location
5. Requires documentation reporting
6. **Enforces Git workflow** (branch naming, conventional commits, no direct commits to `main`)

Update the agent identifier to match your versioning:

```yaml
# In any agent config file
agent: claude-code-v1.0      # Default
agent: claude-code-v2.1      # Your custom version
agent: acme-corp-claude-v1   # Organization-specific
```

### Customizing Document Types

To add a new document type:

1. **Create the template**
   ```
   .devtrail/templates/TEMPLATE-NEWTYPE.md
   ```

2. **Update governance docs**

   Add the new type to:
   - `.devtrail/00-governance/DOCUMENTATION-POLICY.md`
   - `.devtrail/00-governance/AGENT-RULES.md`
   - `.devtrail/QUICK-REFERENCE.md`

3. **Update agent configs**

   Add the new type to all agent configuration files.

4. **Update validation**

   Add the new type to the CLI validation logic and update:
   - `.github/workflows/docs-validation.yml`

### Customizing Folder Structure

The numbered folder structure (`00-governance`, `01-requirements`, etc.) is designed for:
- Logical ordering in file explorers
- Clear separation of concerns
- Easy navigation

You can rename folders, but update all references in:
- Agent configuration files
- Governance documents

---

## Verification

### Verification with Skills (Claude Code)

If using Claude Code, verify documentation compliance with the built-in skill:

```bash
/devtrail-status
```

This skill shows:
- What DevTrail documents were created recently
- Which modified files may need documentation
- Overall documentation compliance status

### Manual Verification

After adoption, verify your setup:

```bash
# Run validation (cross-platform)
devtrail validate
```

### Checklist

- [ ] `.devtrail/` folder structure exists
- [ ] At least one agent config file exists (`CLAUDE.md`, `GEMINI.md`, etc.)
- [ ] Governance documents are present in `.devtrail/00-governance/`
- [ ] Templates are present in `.devtrail/templates/`
- [ ] Git branching strategy documented in `.devtrail/03-implementation/`
- [ ] `QUICK-REFERENCE.md` is accessible
- [ ] `devtrail validate` runs without errors
- [ ] (Optional) Pre-commit hook is installed
- [ ] (Optional) GitHub Actions workflow is enabled

---

## FAQ

### General Questions

**Q: Does DevTrail replace my existing documentation?**

A: No. DevTrail is for *development process documentation* (decisions, changes, reviews). Keep your existing `docs/` folder for user-facing documentation, API references, and guides.

**Q: Do I need to use AI coding assistants to benefit from DevTrail?**

A: No. DevTrail works for human-only teams too. The AI audit features (AILOG, AIDEC, ETH) become especially valuable when using AI assistants, but ADR, REQ, TDE, and other document types are useful for any team.

**Q: How much overhead does DevTrail add?**

A: DevTrail follows a "minimum viable documentation" principle. Only significant changes require documentation. Trivial changes (typos, formatting) are explicitly excluded.

### Technical Questions

**Q: Why use `.devtrail/` instead of `docs/`?**

A: The `docs/` folder is commonly used for user-facing documentation, GitHub Pages, or generated API docs. Using `.devtrail/` avoids conflicts and clearly separates development documentation from user documentation.

**Q: Can I use DevTrail with monorepos?**

A: Yes. You can either:
- Have one `.devtrail/` at the root for the entire monorepo
- Have separate `.devtrail/` folders in each package/service
- Use a hybrid approach with shared governance at root

**Q: How do I handle sensitive information?**

A: DevTrail explicitly prohibits documenting credentials, tokens, or secrets. The validation scripts check for common sensitive patterns and warn you. For genuinely sensitive decisions, document the *existence* of the decision without the sensitive details.

### Adoption Questions

**Q: My team is resistant to more documentation. How do I convince them?**

A: Start small:
1. Begin with just ADRs for architectural decisions
2. Show value through faster onboarding of new team members
3. Demonstrate time saved when revisiting old decisions
4. Gradually expand to other document types

**Q: How do I handle documents created before adopting DevTrail?**

A: You have three options:
1. **Migrate**: Convert old documents to DevTrail format (recommended for important docs)
2. **Reference**: Keep old docs in place, reference them from DevTrail docs
3. **Archive**: Move old docs to an archive folder, start fresh with DevTrail

**Q: What if my AI assistant doesn't follow the rules?**

A: DevTrail rules are instructions, not enforcement. If an AI assistant creates non-compliant documentation:
1. The pre-commit hook (`devtrail validate --staged`) will catch validation errors
2. CI/CD will flag issues in PRs
3. You can manually fix and educate the AI in the next prompt

---

## Getting Help

- **CLI Reference**: [CLI-REFERENCE.md](CLI-REFERENCE.md) — detailed command reference
- **Workflows**: [WORKFLOWS.md](WORKFLOWS.md) — recommended daily usage patterns
- **Issues**: [GitHub Issues](https://github.com/StrangeDaysTech/devtrail/issues)
- **Discussions**: [GitHub Discussions](https://github.com/StrangeDaysTech/devtrail/discussions)
- **Contributing**: See [CONTRIBUTING.md](../../CONTRIBUTING.md)

---

---

<div align="center">

**DevTrail** — Because every change tells a story.

[Back to README](../../README.md) • [Strange Days Tech](https://strangedays.tech)

</div>
