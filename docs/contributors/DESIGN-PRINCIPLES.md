# StrayMark — Design Principles

**Version:** 0.2.2 (empirical annotations + editorial pass for public readability: internal references to Sentinel artifacts are generalized; Plan → Charter rename in going-forward vocabulary)
**Date:** 2026-05-02
**Author:** Jose Villaseñor Montfort — StrangeDaysTech
**Purpose:** Articulate the product philosophy in compact form, to serve as a reference against design decisions and as a criterion for saying no to features that don't fit.

---

## Why this document exists

Software products drift, over time, toward whatever incentive pushes them hardest. If the pressure is revenue, they drift toward features that sell. If the pressure is adoption, they drift toward features that go viral. If the pressure is competition, they drift toward parity with the most visible competitor. Each of those drifts is individually rational and collectively destructive: the product loses its original shape and becomes another averaged-out point in the market.

This document exists so StrayMark has an explicit anchor. Future decisions must be checkable against these principles. A feature that violates a principle may still be a good idea, but it requires a deliberate conversation about *which* principle is being relaxed and *why*. The existence of the document doesn't guarantee discipline; the conscious question it forces, does.

The principles are ranked by hierarchy: when two conflict, the earlier one wins.

**Version 0.2 — empirical annotations.** This version incorporates lessons from the first experiment that stresses the principles against evidence: six execution cycles on a real adopter project (Sentinel — a Go backend system), with three iterations of the Charter format (v1 → v2 → v3) and two dual external audits per cycle. The annotations per principle concentrate on the three the experiment exercised directly (#6, #9, #12); the hierarchy and the wording of the twelve principles do not change. The detailed evidence lives in `straymark-thesis-validation.md` (preserved in `docs/decisions/proposals/` for historical reference).

---

## 1. The tool serves the craft, not the product

StrayMark exists so that the engineer working with AI agents produces work they're proud of. That is the final metric. Any feature that improves adoption, retention, or revenue metrics at the cost of degrading the quality of the user's work is a feature that violates the foundational principle.

In practice, this means decisions are made with the question *"does this help the engineer do better engineering?"*, not with the question *"does this grow the product?"*. The two questions sometimes converge and sometimes don't. When they diverge, the first wins.

## 2. The primary user is the senior engineer orchestrating agents

Not the VP Engineering. Not the CISO. Not the compliance officer. The primary user is the engineer with strong technical judgment who uses AI agents to work on projects they couldn't realistically tackle alone, and who needs externalized cognitive discipline so the agent doesn't introduce systemic chaos.

Features are designed for that person. Flows are optimized for that person. Documentation is written for that person. When other people (compliance officers, managers, auditors) eventually use the product, they do so on a foundation that respects the engineer first. Inverting this hierarchy is the most common cause of developer-tooling products turning bad.

## 3. Strict open source, no asterisks in the core

The framework, the CLI, and the TUI are and always will be open-source under a permissive license. No capped features in the OSS to push toward paid tiers. No mandatory telemetry. No account requirement for full use. No "community edition" inferior in core capabilities to an "enterprise edition".

When a commercial layer eventually exists (Cloud, hosting, services), it must offer genuinely additive value, not rescue functionality stripped from the OSS to manufacture demand. The difference between honest open-core and bait-and-switch open-core is honesty about what is added value vs. what is the tool itself.

## 4. Regulatory compliance is a side effect, not the product

ISO/IEC 42001, EU AI Act, NIST AI RMF, GDPR are useful frameworks that give structure to documentation. StrayMark supports them as an optional layer because they are valuable for users who need them. But the product is not "a compliance tool". The product is an engineering tool that produces, as a side effect, evidence compatible with those frameworks.

This distinction matters because the compliance buyer and the engineer using the tool have different needs. When the product is positioned as compliance, design decisions skew toward the compliance buyer and degrade the engineer's flow. Keeping compliance as a side effect protects the quality of the primary flow.

## 5. Schema-driven before feature-driven

The product's central entities (Stage Closure Bundle, Charter, Document) are defined first as formal schemas with strict versioning. Features are built against those schemas. This has three structural benefits: it lets multiple implementations coexist without coordinating, it makes evolution controlled and reversible, and it forces thinking about the contract before the implementation.

The practical consequences are that breaking schema changes are rare and deliberate, that schemas are first-class product documentation, and that features that require violating a schema must first propose the schema's evolution.

## 6. Cognitive discipline over raw productivity

StrayMark does not compete with tools that promise *"write code 10× faster with AI"*. It competes, if it competes with anything, against the chaos that fast AI code generates in serious projects. The metric that matters is not raw agent speed but sustained quality of the system produced.

This means features that add justified friction (validations, gates, documentation requirements, pre-work before planning) are acceptable and sometimes desirable. Features that promise to remove friction at the cost of the agent's reasoning quality are suspect by default.

**v0.2 annotation — virtue vs. ceremony distinction.** Empirical evidence revealed that not all friction in the flow has the same character. Friction is *virtuous* when it externalizes public signal verifiable by third parties: formally naming risks that emerge during execution makes heterogeneous external auditors capture them; a drift-check at work closure detects declared-vs-modified file mismatches with zero false positives in empirical validation; dual auditing with two heterogeneous models calibrates cross-model and catches gaps a single auditor misses. Friction is *attackable ceremony* when it only generates manual triage or prescribes without allowing context adaptation: a rigid template that clashes with the destination module's convention, or a drift-check alerting on divergences already documented elsewhere, are the two observed cases. The first is preserved; the second is a format bug, not a virtue of the principle. Detailed evidence and actionable criteria live in `straymark-thesis-validation.md` §5 (preserved in `docs/decisions/proposals/`).

## 7. Local-first, Cloud as amplifier

The CLI works completely, without network, in the local repo. That is the first experience and the foundation everything builds on. Cloud exists to amplify value (cross-repo aggregation, signed evidence, semantic historical search, team collaboration) but never to replace the CLI or turn it into a thin client.

A user who never connects to Cloud must still get most of the product's value. This protects the tool against Cloud failure modes, against business-model changes, and against the classic drift of products that start local and end up requiring mandatory connection.

## 8. Project memory lives in the repo, not in an external database

AILOGs, ADRs, AIDECs, Charters, and bundles live as versioned files alongside the code. The reason is structural: project memory must outlive the product that generated it. If StrayMark disappears tomorrow, the `.straymark/` files remain readable, parseable, and useful because they are structured markdown and JSON conforming to public schemas.

This decision gives up some capabilities that would be easier with a centralized backend (for example, instant full-text search without sync), but gains long-term reliability and user control over their own data. The asymmetry is deliberate and permanent.

## 9. Simplicity over capability

When two designs meet the same goal, the simpler one wins. When a feature is genuinely useful but requires significant complexity, it's better to wait until the usage pattern is validated in at least three real projects before crystallizing it into the product.

This applies to both the product and the product's language. Command names, document types, and framework concepts are chosen to be immediately understandable, not to sound technically sophisticated. If a concept requires lengthy explanation for a new user, it's probably poorly designed.

**v0.2 annotation — bash before framework when bash suffices.** Empirical evidence demonstrated that a ~145-line bash script (awk + grep + git) covers the "drift between declared and modified files" case with zero false positives across two tests. The conscious decision not to implement it initially in the CLI's language (Rust) is justified thus: bash is zero-build, no additional dependencies, inspectable in place; maintenance cost is low. When porting to the CLI in phase 2 of the roadmap (`straymark-cli-roadmap.md`, preserved in `docs/decisions/proposals/`), preserve the bash simplicity where possible — invoke the shell from Rust or reimplement carefully without inflating the logic. Complexity crystallizes only when the usage pattern demands it, not by default.

## 10. Honesty about what the tool does not do

StrayMark does not evaluate models. It is not an LLM gateway. It does not replace the engineer's judgment. It does not prevent all hallucinations. It does not automatically certify regulatory compliance. The product pages, documentation, and public communication are explicit about these limits.

The reason is practical: users who adopt the tool with realistic expectations stick with it. Users who adopt it believing it does things it doesn't abandon it frustrated. Early honesty about the limits is a better marketing investment than broad promises.

## 11. The community takes care of the tool, not the other way around

StrayMark users are typically engineers who know more than the product team about their own use cases. Contributions, issues, and feedback are treated with professional respect, and roadmap decisions are made incorporating that view. Not democratically (the product needs a coherent vision), but as serious input.

This does not mean doing everything the community asks. It means explaining when "no" is said and why, keeping the conversation visible and public, and accepting that the product will be better off from the intelligent pressure of those who use it seriously than from the team's isolated decisions.

## 12. The product's velocity is the velocity of learning

StrayMark must not advance faster than what we learn about how it is actually used. Crystallizing features prematurely, before having real usage data in at least three different projects, generates high maintenance costs over features that may be wrong. The patience to wait for evidence is discipline, not inaction.

This translates into an explicit preference for: prototypes before features, observation before crystallization, stable schemas over changing features, and incremental evolution over large rewrites.

**v0.2 annotation — the spirit of N≥3 facing limited initial evidence.** The empirical evidence available at the time of writing this version comes from a single adopter project, in a single domain (Go backend), with a single author. Applied literally, the principle would block any crystallization from that evidence. Applied in spirit — rigorous observation before crystallization, incremental evolution with real data — the available evidence covers three axes of structural diversity: (a) variable scale of the units of work (XS, S, M across five cycles), (b) format iteration under empirical pressure (v1 → v2 → v3, each derived from the previous cycle), (c) cross-model auditor calibration (two heterogeneous models converging to similar scores in the most stressed cycle). These six cycles × three formats × three scales are evidence equivalent to the N≥3 the principle aims to protect.

What remains undiversified is the domain. To preserve the principle in its operational form: any schema that crystallizes from that evidence is published as `v0.json` with an *experimental* mark. The transition to stable `v1.0` requires validation with a second project in another domain (frontend, ML pipeline, infra-as-code). Any feature that depends on an assumption without evidence — like the conditional-approval assumption, which requires multi-actor flow that the available evidence did not exercise — is explicitly blocked until a project that does exercise it appears. See `straymark-thesis-validation.md` §6 and §8 (preserved in `docs/decisions/proposals/`) for the argument's development and the list of specific gaps per assumption.

**Meta-meta pattern detected — format self-evolution.** Empirical evidence revealed that the format improves itself: each Charter (Plan in historical terminology) executed generates data that refines the next format. This suggests extending the principle in future versions: the tool evolves with itself; each use is input for the next version. For now this is documented as observation, not as an additional principle, until a second project confirms the pattern.

---

## How to use this document

When a non-trivial design decision arises, it is worth contrasting it explicitly against the twelve principles and recording the result. Three useful patterns:

**Decisions that satisfy all principles**: proceed normally.

**Decisions that conflict with one principle**: explicitly document which principle is being relaxed, why, and what evidence would lead to reverting the decision. This record should live in an AIDEC or ADR.

**Decisions that violate foundational principles (1, 2, 3, 4)**: are not made without first reviewing whether the principle is still correct. If it is, the decision is discarded. If not, the principle is updated *before* making the decision, not after.

The principles are formally reviewed every six months with the accumulated evidence. Changes between formal reviews are possible but must be deliberate and documented.

---

*This document is a guardrail, not a manual. Its usefulness is measured in how many times it makes us pause before making a decision we would later have regretted.*
