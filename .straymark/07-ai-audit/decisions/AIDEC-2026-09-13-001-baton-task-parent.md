---
id: AIDEC-2026-09-13-001
title: Conservative explicit parent resolution for Baton tasks
status: accepted
created: 2026-09-13
agent: codex-cli-v0.154.0
confidence: high
review_required: false
risk_level: medium
eu_ai_act_risk: not_applicable
nist_genai_risks: [human_ai_config]
iso_42001_clause: []
tags: [baton, inheritance]
related:
  - 07-ai-audit/agent-logs/AILOG-2026-09-13-001-baton-task-inheritance.md
  - 07-ai-audit/agent-logs/AILOG-2026-09-17-001-baton-task-inheritance-maintainer-review-amendment.md
---

# AIDEC: Explicit task parent

## Context

Issue #427 requests the inheritance ratified in #332 without adding task declaration
slots. One spec may have multiple Charters; filenames and titles cannot assign a task.

## Alternatives

1. Pick the first/newest/active Charter or infer from titles: more coverage, but
   ambiguous ownership could silently recommend a cheaper model.
2. Require exactly one originating_spec relation to the existing sibling spec.md
   inside the project: bounded implementation with explicit evidence, but leaves
   multi-Charter specs and incomplete inventories undeclared.
3. Design a new task-to-Charter mapping: fuller solution, requires a separate schema
   decision and exceeds this first adopter contribution.
4. (Added in maintainer review.) Every Charter whose originating_spec resolves to the
   sibling spec.md is a candidate parent; inherit when all candidates declare the same
   work_verb and design_provenance. Same evidence as option 2 — neither proves which
   tasks a Charter owns — but stable under the charter-chain pattern.

## Decision

Proposed in the PR: option 2. **Amended in maintainer review (2026-09-17): option 4**,
chosen by the maintainer. Option 2 is not monotonic: Sentinel's specs 002–005 carry
2–8 Charters each, so a spec that gains an agreeing Charter (e.g. a polish Charter)
would retroactively unclassify tasks already done. Uniqueness never established
ownership either, so it bought no safety that agreement does not. Agreement is on the
declaration itself, not on the class the current classifier derives: `operate` and
`operate` + `design_provenance: new` do not agree. A candidate declaring nothing means
disagreement.

Candidate parents are read from the raw frontmatter, so a Charter the typed parser
rejects (an out-of-schema field) still counts as a parent. Only an unreadable
frontmatter makes the parent inventory incomplete; inheritance is then disabled for
the project and the CLI names the blocking Charters on stderr. Only work_verb and
design_provenance flow to the child; parent effort_estimate is not a measurement of
task size. Missing or invalid verbs remain unclassifiable in the existing classifier;
no title fallback or task frontmatter.

## Consequences and review

Preserves deterministic and read-only recommendations. Known false negatives remain
visible as undeclared, and a disagreeing Charter still unclassifies a spec's tasks —
correctly, since ownership is then ambiguous. Extending mappings or risk policy is
separate work. Accepted by the maintainer as the implementation rule for #332's task
inheritance; it is not a schema ratification and adds no declaration slot.
