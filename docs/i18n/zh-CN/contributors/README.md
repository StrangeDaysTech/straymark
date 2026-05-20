# 贡献者文档

为 StrayMark 贡献者准备的资源——不论是阅读代码、撰写翻译、提议变更，还是想理解 framework 形成现貌背后的*为什么*。


---

## 概念

非时变的材料——在打开一个触及 framework 表面的 PR 之前，先阅读这些以理解 StrayMark *是什么*以及它*为何*呈现现有形态。

| 文档 | 涵盖什么 |
|---|---|
| [`DESIGN-PRINCIPLES.md`](DESIGN-PRINCIPLES.md) | 治理产品决策的十二条层级化原则。包含来自验证周期的 v0.2 实证注解，覆盖原则 #6（认知纪律）、#9（简洁）与 #12（速度 = 学习的速度）。 |
| [`WHAT-IS-A-CHARTER.md`](WHAT-IS-A-CHARTER.md) | Charter 工件的概念范围：对一项工作单元的事前声明，带有核验契约与审计锚点。映射出 GitHub SpecKit 的 `plan.md` 所覆盖的与 StrayMark Charter 所覆盖的之间的边界——它们不是同一回事。 |

## 工作流指南

| 文档 | 涵盖什么 |
|---|---|
| [`TRANSLATION-GUIDE.md`](/docs/contributors/TRANSLATION-GUIDE) | 将 StrayMark 文档翻译为其他语言的规则与约定。在提交新增或修改 `i18n/` 文件的 PR 之前请先阅读。*（目前仅英文版。）* |

## 历史提案（已归档）

项目在 CLI 之前演化阶段的提案与路线图，作为上下文保留——它们解释了当前形态是如何浮现的。这些**不**在向前的方向上维护；当前行为的权威来源是代码、`dist/.straymark/schemas/` 下的 schema，以及 CHANGELOG。可在 GitHub 上的 [`docs/decisions/proposals/`](https://github.com/StrangeDaysTech/straymark/tree/main/docs/decisions/proposals) 浏览：

| 文件 | 快照日期 | 捕获了什么 |
|---|---|---|
| `2026-04-30-thesis-validation.md` | 2026-04-30 | 针对 Sentinel 六个周期（Go 后端）的产品论点的实证验证——即促成 `DESIGN-PRINCIPLES.md` v0.2 注解的证据体。 |
| `2026-04-30-charter-telemetry.md` | 2026-04-30 | 用于观察真实项目中 Charter 执行的遥测埋点 schema。规范版本现已存放在 `dist/.straymark/schemas/charter-telemetry.schema.v0.json`。 |
| `2026-05-03-cli-roadmap.md` | 2026-05-03 | Rust CLI 的三阶段实施路线图，带有收尾标准。阶段 1–3 现已发布于 `cli-3.x`。 |
| `2026-05-03-audit-skills-design.md` | 2026-05-03 | 将 `/straymark-audit-prompt` 与 `/straymark-audit-review` skill 作为"人在闭环中"的检查点的设计。在 `fw-4.8.0` 中实施。 |
| `2026-05-03-audit-skills-rollout.md` | 2026-05-03 | audit skill 的运营性铺开计划（gating 标准、遥测、分阶段发布）。 |
| `2026-05-04-audit-cli-flow.md` | 2026-05-04 | 在首次实际遇到一个跨多次提交的 L 级 Charter（Sentinel CHARTER-07）之后，对外部审计流程的重新设计。在 `cli-3.10+` 中实施。 |

当前代码库的 ADR（架构决策记录）位于 GitHub 上的 [`docs/decisions/`](https://github.com/StrangeDaysTech/straymark/tree/main/docs/decisions)。

---

*另请参阅：[`../adopters/`](../adopters/ADOPTION-GUIDE.md) 是面向在自有项目中采用 StrayMark 的团队的文档，包括 [`ADOPTION-GUIDE.md`](../adopters/ADOPTION-GUIDE.md)、[`CLI-REFERENCE.md`](../adopters/CLI-REFERENCE.md) 与 [`WORKFLOWS.md`](../adopters/WORKFLOWS.md)。*
