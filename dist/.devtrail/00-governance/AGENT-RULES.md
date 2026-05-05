# Rules for AI Agents - DevTrail

> This document defines the rules that all AI agents must follow when working on projects under DevTrail.

**Languages**: English | [Español](i18n/es/AGENT-RULES.md) | [简体中文](i18n/zh-CN/AGENT-RULES.md)

---

## 1. Mandatory Identification

### When Starting a Session

Every agent must identify themselves with:
- Agent name (e.g.: `claude-code-v1.0`, `cursor-v1.0`, `gemini-cli-v1.0`)
- Agent version if available

### In Every Document

Include in the frontmatter:
```yaml
agent: agent-name-v1.0
confidence: high | medium | low
```

---

## 2. When to Document

### MANDATORY - Create document

| Situation | Type | Notes |
|-----------|------|-------|
| Code complexity above threshold | AILOG | Run `devtrail analyze <changed-files> --output json`. If `summary.above_threshold > 0`, create AILOG (default threshold: 8). **Fallback**: if CLI unavailable, apply >20 lines of business logic heuristic |
| Decision between 2+ technical alternatives | AIDEC | Document alternatives |
| Changes in auth/authorization/PII | AILOG + ETH | `risk_level: high`, ETH requires approval |
| Changes in public API or DB schema | AILOG | `risk_level: medium+`, consider ADR |
| Changes in ML models or AI prompts | AILOG | `risk_level: medium+`, human review required |
| Integration with external service | AILOG | - |
| Addition/removal/upgrade of security-critical dependencies | AILOG | Human review required |
| Changes affecting AI system lifecycle (deployment, retirement) | AILOG + ADR | Human review required |
| Changes to OTel instrumentation (spans, attributes, pipeline) | AILOG | Tag `observabilidad`, see §9 |

### PROHIBITED - Do not document

- Credentials, tokens, API keys
- Personally identifiable information
- Secrets of any kind

### OPTIONAL - Does not require document

- Formatting changes (whitespace, indentation)
- Typo corrections
- Code comments
- Minor style changes

---

## 3. Autonomy Limits

### Create Freely

| Type | Description |
|------|-------------|
| AILOG | Logs of actions performed |
| AIDEC | Technical decisions made |

### Create Draft → Requires Human Approval

| Type | Description |
|------|-------------|
| ETH | Ethical reviews |
| ADR | Architectural decisions |

### Propose → Requires Human Validation

| Type | Description |
|------|-------------|
| REQ | System requirements |
| TES | Test plans |

### Create Draft → Requires Human Approval (new types)

| Type | Description |
|------|-------------|
| SEC | Security assessments (`review_required: true` always) |
| MCARD | Model/System cards (`review_required: true` always) |
| DPIA | Data Protection Impact Assessments (`review_required: true` always) |

### Create Freely (new types)

| Type | Description |
|------|-------------|
| SBOM | Software Bill of Materials (factual inventory) |

### Identify Only → Human Prioritizes

| Type | Description |
|------|-------------|
| TDE | Technical debt |
| INC | Incident conclusions |

---

## 4. When to Request Human Review

Mark `review_required: true` when:

1. **Low confidence**: `confidence: low`
2. **High risk**: `risk_level: high | critical`
3. **Security decisions**: Any change in auth/authz
4. **Irreversible changes**: Migrations, deletions
5. **User impact**: Changes affecting UX
6. **Ethical concerns**: Privacy, bias, accessibility
7. **ML model changes**: Changes to model parameters, architecture, or training data
8. **AI prompt changes**: Modifications to prompts or agent instructions
9. **Security-critical dependencies**: Addition, removal, or upgrade of security-sensitive packages
10. **AI lifecycle changes**: Deployment, retirement, or major version changes of AI systems

---

## 5. Document Format

### Use Templates

Before creating a document, load the corresponding template:

```
.devtrail/templates/TEMPLATE-[TYPE].md
```

### Naming Convention

```
[TYPE]-[YYYY-MM-DD]-[NNN]-[description].md
```

### Location

| Type | Folder |
|------|--------|
| AILOG | `.devtrail/07-ai-audit/agent-logs/` |
| AIDEC | `.devtrail/07-ai-audit/decisions/` |
| ETH | `.devtrail/07-ai-audit/ethical-reviews/` |
| ADR | `.devtrail/02-design/decisions/` |
| REQ | `.devtrail/01-requirements/` |
| TES | `.devtrail/04-testing/` |
| INC | `.devtrail/05-operations/incidents/` |
| TDE | `.devtrail/06-evolution/technical-debt/` |
| SEC | `.devtrail/08-security/` |
| MCARD | `.devtrail/09-ai-models/` |
| SBOM | `.devtrail/07-ai-audit/` |
| DPIA | `.devtrail/07-ai-audit/ethical-reviews/` |

### Tags and Related

When populating the `tags` and `related` fields in frontmatter:

**Tags:**
- Use kebab-case keywords: `sqlite`, `api-design`, `gnome-integration`
- 3 to 8 tags per document describing topic, technology, or component
- Tags enable search and categorization in `devtrail explore`

**Related:**
- Reference other **DevTrail documents only** — use the document filename with `.md` extension
- If the document is in a subdirectory within `.devtrail/`, include the relative path: `07-ai-audit/agent-logs/daemon/AILOG-2026-02-03-001-file.md`
- If the document is in the same directory, the filename alone is sufficient
- **Do not** put task IDs (T001, US3), issue numbers, or external URLs in `related` — put those in the document body instead

---

## 6. Communication with Humans

### Be Transparent

- Explain the reasoning behind decisions
- Document considered alternatives
- Admit uncertainty when it exists

### Be Concise

- Get to the point
- Avoid unnecessary jargon
- Use lists and tables when appropriate

### Be Proactive

- Identify potential risks
- Suggest improvements when evident
- Alert about technical debt

---

## 7. Error Handling

If the agent makes an error:

1. **Document** the error in an AILOG
2. **Explain** what went wrong
3. **Propose** correction
4. **Mark** `review_required: true`

---

## 8. Document Updates

### Create New vs Update

| Situation | Action |
|-----------|--------|
| Minor correction | Update existing document |
| Significant change | Create new document |
| Obsolete document | Mark as `deprecated` |
| Complete replacement | Create new + mark previous as `superseded` |

### When Updating

- Update the `updated` field in frontmatter
- Add a note in the history section if it exists
- Maintain consistency with related documents

---

## 9. Observability (OpenTelemetry)

When working on projects that use OpenTelemetry:

### Rules

- **Do not** capture PII, tokens, or secrets in OTel attributes or logs
- **Record** instrumentation pipeline changes (new spans, changed attributes, Collector configuration) in AILOG with tag `observabilidad`
- **Create** AIDEC or ADR when adopting OTel in distributed projects — document the adoption decision and backend selection
- **Set** `observability_scope` in frontmatter when the change involves OTel instrumentation

### Documentation Triggers

| Change | Document | Additional |
|--------|----------|-----------|
| New spans or changed attributes | AILOG | Tag `observabilidad` |
| OTel backend selection | AIDEC or ADR | If distributed system |
| Collector pipeline configuration | AILOG | Tag `observabilidad` |
| Sampling strategy changes | AIDEC | Document rationale |
| Observability requirements | REQ | Use Observability Requirements section |
| Trace propagation tests | TES | Use Observability Tests section |
| Incident with trace evidence | INC | Include trace_id/span_id in timeline |
| Instrumentation debt | TDE | Tag `observabilidad` |

---

## 10. Architecture Diagrams (C4 Model)

When creating ADR documents that involve architectural changes:

- **Include** a Mermaid C4 diagram at the appropriate level
- **Use** `C4Context` for system-level decisions (who uses the system, external dependencies)
- **Use** `C4Container` for service/container-level decisions (applications, databases, message queues)
- **Use** `C4Component` for internal module decisions (components within a service)
- **See** `00-governance/C4-DIAGRAM-GUIDE.md` for syntax reference and examples

> Diagrams are optional for minor decisions. Use them when the decision changes system boundaries, introduces new services, or modifies inter-service communication.

---

## 11. API Specification Tracking

When a change modifies API endpoints:

- **Verify** that the corresponding OpenAPI or AsyncAPI specification is updated
- **Reference** the spec path in the AILOG or ADR using the `api_spec_path` field (in REQ) or `api_changes` field (in ADR)
- **Document** breaking API changes in an ADR with `risk_level: high`

---

## 12. Audit Checkpoint (Charter workflow)

When co-implementing a Charter, the agent **proactively offers** an external multi-model audit at a specific moment in the workflow. The checkpoint is **soft** — it never blocks `charter close` and is never escalated to enforcement. External audit is opt-in by design (cost, trust in the operator's primary discipline).

### When to emit the checkpoint

Emit the checkpoint **once per Charter** when **all four** triggers are simultaneously true:

1. The Charter is in status `in-progress` or `declared` (not `closed`).
2. All tasks in the Charter's `## Tasks` section are marked `[x]` complete (or the agent just completed the last one).
3. `devtrail charter drift <CHARTER-ID>` exits 0 (no unaccounted drift).
4. The developer has **not** invoked `devtrail charter close <CHARTER-ID>` yet, nor mentioned intent to close.

If the developer declined audit on a previous turn for the same Charter, **do not re-emit** in subsequent turns of the same conversation.

### Form of the checkpoint message

Render the message like this (substitute `<CHARTER-ID>` and the recommendation reasoning):

```
Reached the checkpoint for <CHARTER-ID>. Implementation is done, drift
check is clean, only `devtrail charter close` is pending.

At this point you can run an external audit (typically 2 LLMs of
different families + a calibrator) that produces cross-model findings
on the implementation.

My recommendation: [YES / NO], because:
  - <one specific reason grounded in the Charter, AILOGs, or diff>

If you decide to audit:
  Run /devtrail-audit-prompt <CHARTER-ID> and I will write the unified
  audit prompt to .devtrail/audits/<CHARTER-ID>/audit-prompt.md.
  Then open one or more auditor-side CLIs (gemini-cli, claude-cli,
  copilot-cli, codex-cli) in this repo and invoke
  /devtrail-audit-execute <CHARTER-ID> in each — recommendation: at
  least 2 auditors of different model families. When and only when
  ALL auditors you commissioned have completed, return here and run
  /devtrail-audit-review <CHARTER-ID>. I will consolidate the N
  reports into a review.md document with verdicts, remediation plan,
  and auditor ratings, and merge the YAML block into the Charter
  telemetry.

If you decide not to audit:
  Continue with `devtrail charter close <CHARTER-ID>` when you're
  ready. External audit is fully optional — DevTrail's declarative
  Charter + drift check + AILOG discipline gives the cycle enough
  rigor for confident close without it.
```

### Heuristics for the YES/NO recommendation

These are heuristics, not rigid rules — you are close to the context, refine them with the adopter.

**Recommend YES when** (any one suffices):

- The Charter touched security-critical surface (auth, RLS, secret handling, IAM).
- The Charter introduced a new component (not a refactor) that the developer has not co-implemented before.
- An associated AILOG documents an `R<N>` with `confidence: low | medium` and `risk_level: medium` or higher.
- The developer marked the Charter as `effort_estimate: L` and this is the adopter's first Charter.
- The developer **explicitly** asked for cross-model validation in the Charter trigger.
- **Structural complexity signal** *(only when the CLI was compiled with the `analyze` feature, true for official binaries)*: the diff in `range` introduces or modifies at least one function whose cognitive complexity exceeds **2× the configured threshold** in `.devtrail/config.yml` (`complexity.threshold`, default `8` → ≥ `17`). A dense new function is exactly the case where two cross-family auditors capture implementation gaps that a single model misses. **Graceful-degradation:** if the binary lacks the `analyze` feature, silently skip this signal — do not warn, do not mention.

**Recommend NO when** (all of these together):

- The Charter is a refactor or documentation change (no new behavior).
- `effort_estimate` is `XS` or `S`.
- Associated AILOGs all have `confidence: high` and no emergent `R<N+1>` risks.
- The Charter's `risk_level` is `low` (or unset).

**Default case (no clear signal either way):** recommend **NO** with neutral framing ("I don't see a specific signal that justifies the cost of two additional models; close when you're ready"). The cost of external audit is real — do not inflate adoption by recommending YES on inertia.

### Rules of engagement

- The checkpoint is **never** repeated within the same Charter once the developer responds.
- The checkpoint **does not** block any subsequent action. If the developer ignores it and runs `charter close`, close proceeds normally — there is no enforcement and there will not be (this is a permanent v0+v1 design decision; see `Propuesta/devtrail-audit-skills.md` §2.2).
- The checkpoint is **not** counted in any quality metric. There is no "% Charters audited" KPI in `devtrail metrics` — by design, to avoid creating an incentive to inflate the audit count.
- If the developer accepts the audit, the workflow proceeds via three skills in sequence: `/devtrail-audit-prompt` (writes the unified prompt at the canonical path) → `/devtrail-audit-execute` × N (one per auditor-side CLI the operator opens — these run in those CLIs, not in the main agent) → `/devtrail-audit-review` (consolidates N reports inline into `.devtrail/audits/<id>/review.md` and merges the YAML into telemetry). Operators never copy/paste prompts or reports — file exchange happens via canonical paths under `.devtrail/audits/`.

---

*DevTrail v4.8.0 | [Strange Days Tech](https://strangedays.tech)*
