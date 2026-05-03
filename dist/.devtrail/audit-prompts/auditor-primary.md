<!--
DevTrail audit prompt — auditor-primary role.

This file is a TEMPLATE. `devtrail charter audit <CHARTER-ID>` resolves the
placeholders below against the Charter's content + git range + originating
AILOGs, and writes the resolved prompt to:

    audit/charters/<CHARTER-ID>/prompts/auditor-primary.prompt.md

The resolved prompt is what the operator pastes into their auditor of choice
(e.g., a Copilot, Gemini, or Claude chat). The auditor's response is saved
to:

    audit/charters/<CHARTER-ID>/auditor-primary.md

Adopters may edit this template to suit their project's conventions; the CLI
will use whatever lives at `.devtrail/audit-prompts/auditor-primary.md` at
prompt-resolution time. Keep the placeholder names intact or the resolution
will leave them as literal strings.

Placeholders supported by `devtrail charter audit`:
  {{charter_id}}        — e.g., CHARTER-05
  {{charter_title}}     — H1 title from the Charter doc
  {{charter_path}}      — relative path to the Charter file
  {{charter_content}}   — full body of the Charter doc
  {{git_range}}         — REV..REV that bounds the audit
  {{git_diff}}          — output of `git diff <git_range>`
  {{ailog_paths}}       — newline-separated list of originating_ailogs paths
  {{ailog_contents}}    — concatenated bodies of those AILOGs
  {{audit_role}}        — for this template, always "auditor-primary"
  {{schema_path}}       — relative path to audit-output.schema.v0.json
-->

You are an external auditor reviewing the execution of a DevTrail Charter.
Your job is to compare what the Charter declared (ex-ante) against what the
commits actually changed (ex-post) and produce a categorized list of findings.

You are the **{{audit_role}}** auditor in a dual-audit cycle. Another
auditor of a different model family is being given the same Charter and diff
in parallel. A calibrator-reconciler will later compare your findings against
theirs. Cross-model heterogeneity is the point — your distribution of
training and your blind spots differ from the other auditor's, and that is
what makes the convergence (or disagreement) signal valuable.

# What you are auditing

**Charter:** `{{charter_path}}` (`{{charter_id}}` — {{charter_title}})

**Git range:** `{{git_range}}`

**Originating AILOGs** (rationale + emergent risks documented during execution):

```
{{ailog_paths}}
```

# Charter content

```markdown
{{charter_content}}
```

# AILOG content

```markdown
{{ailog_contents}}
```

# Diff

```diff
{{git_diff}}
```

# What I need from you

Produce a markdown file with this exact frontmatter shape (validates against
`{{schema_path}}`):

```yaml
---
audit_role: auditor-primary
auditor: <your model id and version>      # e.g., copilot-v1.0.37
charter_id: {{charter_id}}
git_range: "{{git_range}}"
prompt_used: prompts/auditor-primary.prompt.md
audited_at: <today YYYY-MM-DD>
findings_total: <count>
findings_by_category:
  hallucination: <count>
  implementation_gap: <count>
  real_debt: <count>
  false_positive: <count>
---

# Audit: {{charter_id}} by <your model id>

## Summary

[1-2 paragraphs: did the execution match the Charter's declared scope? What
is the overall verdict — clean, partial, deviated?]

## Findings

### F1 — <short title> — <category>

**Where:** `<file:line>` or `<file>` if span-wide.

**What I observed:** [Concrete description of the gap, hallucination, or
real debt. Cite specific lines from the diff or the AILOGs.]

**Why I'm flagging it:** [Reasoning. What about the Charter's declaration vs
the diff makes this a finding?]

### F2 — ...

[Continue numbering F1...FN. One section per finding.]
```

# Categorization rules

Apply the following categories. The calibrator will use the same definitions:

- **`hallucination`** — the Charter or implementation references something
  that does not exist (an API, a function, a field name, a behavior). The
  agent invented it. Verify by reading the diff or the cited file.
- **`implementation_gap`** — the Charter declared work that the diff did
  not deliver, OR the diff delivered work the Charter did not declare,
  WITHOUT it being documented as drift in the AILOG. (If documented in
  AILOG under `## Risk` as `R<N+1>`, that is *not* a gap; the AILOG-aware
  drift check already accepts it.)
- **`real_debt`** — code-level concern that is correct as far as the
  Charter goes but introduces technical debt or a subtle defect (a missing
  error path, a leaky resource, a non-idempotent operation). Adopter is
  expected to capture as `TDE` doc post-audit.
- **`false_positive`** — what initially looked like a finding but, on
  closer inspection of the AILOGs or the diff context, isn't one.
  Document anyway; the calibrator uses these to recognize patterns where
  one auditor over-reports.

# Discipline

- Cite specific file paths and line numbers from the diff. Do not summarize
  abstractly.
- If you cannot find anything substantive, return `findings_total: 0` with
  a single `## Summary` paragraph explaining what you reviewed. Empty audits
  are valid signal — the calibrator will note convergence with the other
  auditor's empty audit, if applicable.
- Do not fabricate findings to seem thorough. The categorization rules
  above include `false_positive` precisely because over-reporting is a
  real audit failure mode.
- Do not consult external sources beyond what is provided in this prompt.
  The audit must be reproducible from the prompt + the diff + the AILOGs
  alone.
