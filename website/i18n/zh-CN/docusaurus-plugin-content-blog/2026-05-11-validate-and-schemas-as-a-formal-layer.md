---
slug: validate-与作为形式层的-schema
title: validate 与作为形式层的 schema
authors:
  - jose
tags:
  - straymark
  - validate
  - schemas
  - governance
  - phase-2
date: 2026-05-11
description: 464 行 bash 被一个 flag 替换。从"我检查这看起来像一份文档"变成"我检查这是否满足正典 schema，在你要求的模式下，附带具体的运行结果"。这就是 CLI 路线图 Phase 2 与 validate 的形式化。
---

*464 行 bash 被一个 flag 替换。从 "我检查这看起来像一份文档" 变成 "我检查这是否满足正典 schema，在你要求的模式下，附带具体的运行结果"。这就是 CLI 路线图 Phase 2 与 validate 的形式化。*

<!-- truncate -->

## 1. 拥有 schema 与针对 schema 进行验证，并非同一回事

`straymark validate` 自 2026 年 3 月 25 日起就已存在 —— PR #27，CLI Phase 1 第一段弧线的组成部分。那时它所做的事情很小：验证 `.straymark/` 中的文档具有有效的 Markdown 语法、可解析的 frontmatter，以及正确类型的最小字段集。它有用，但称不上*执行*。

2026 年 5 月 11 日，随着 `fw-4.6.0 / cli-3.7.0` 发布，改变的是别的东西：CLI 路线图的 Phase 2 关闭了七个 PR 组成的完整弧线（#73 到 #79），并将它们整合进一个 PR（#80）。CHANGELOG 直白地说明了这一点：

> *"自重新定位以来第一个承载功能的发布……`Propuesta/devtrail-cli-roadmap.md` 的 Phase 2 以 7 个 bisect-safe PR（#73–#79）落地，在此处汇总供审阅者参考。本次发布关闭了 Sentinel 实验打开的实证循环：Charter 关闭时的遥测、Charter 关闭时的漂移检测，以及针对 `review_required: true` 文档的正典批准信号。"*

对于这篇文章而言，重要的不是七个 PR 作为各自里程碑的意义；而是其中三个触及 `validate` 命令和 schema 的 PR。3 月份的"我检查这看起来像一份文档"，到 5 月变成了"我检查这是否满足该类型的正典 schema，在你要求的模式下，附带具体的运行结果"。

---

## 2. 从手工 bash 到命令 flag

这次变化中最显眼的部分很小：`--staged` flag。

在 Phase 2 之前，采用者在提交前验证文档的推荐方式是一个叫做 `pre-commit-docs.sh` 的 bash 脚本。它位于 `dist/.straymark/scripts/`，共有 464 行。它做了合理的事情：读取 `git diff --cached --name-only`，过滤 `.straymark/` 中的文件，用 `awk` 解析它们的 frontmatter，检查字段。它能用。而且它是 464 行 bash，每个采用者都必须相信它执行了正确的事情、用正确的版本、用正确的 YAML 解析器。

3 月 28 日的一个 PR（commit `d9ea874`）用一个 flag 替换了这 464 行：

```bash
straymark validate --staged
```

同样地读取暂存文件，同样地按 `.straymark/` 过滤，同样地进行验证。区别在于：今天所有这些都住在二进制文件内的 Rust 中，而不是磁盘上的 bash 里。采用者不必维护脚本；脚本不会在版本之间产生偏差；行为在 Linux、macOS 和 Windows 上完全一致。框架不再要求采用者信任一个松散的脚本，而是给了他们一个带有版本化二进制文件隐含承诺的命令。

这与第 6 篇文章记录的归档纪律模式相同（保留 git 历史而非重写），或第 4 篇文章记录的决策 A1 模式（无需调用 API 的编排）：从"操作者手动执行 X，存在偏差风险"转变为"框架将 X 正典化为一个声明式命令"。`--staged` 不是一个异域功能；它是被扫除的手工债务。

---

## 3. 三个 v0 schema

Phase 2 也整合了位于 `dist/.straymark/schemas/` 的三个 schema：

| Schema | 描述内容 | 来源 |
| --- | --- | --- |
| `charter.schema.v0.json` | Charter 作为一等实体的结构（第 4 篇） | Sentinel `/plan-audit`（6 个周期，格式 v1 → v2 → v3） |
| `charter-telemetry.schema.v0.json` | Charter 遥测（第 3 篇，在第 10 篇中扩展） | Sentinel 的 5 个 `PLAN-NN.telemetry.yaml` 文件 |
| `audit-output.schema.v0.json` | 外部审计的正典输出（第 4 篇 + 第 10 篇） | Sentinel 双重审计（Copilot + Gemini + 批判性 Claude） |

每个 schema 在其根部都带有一个 `$comment` 字段。`charter.schema.v0.json` 中的那个字面上写道：

> *"EXPERIMENTAL v0。从 Sentinel /plan-audit 结晶而来（6 个周期，格式 v1 → v2 → v3）。在第二个领域（前端、ML 流水线或基础设施即代码）验证之前，不会稳定至 v1.0。"*

`charter-telemetry.schema.v0.json` 中的类似：

> *"EXPERIMENTAL v0。源自 Sentinel 中的 5 个 PLAN-NN.telemetry.yaml 文件（一个项目，Go 后端）—— 与 charter.schema.v0.json 相同的 N=1 领域警告。在第二个领域验证之前，不会稳定至 v1.0。"*

`audit-output.schema.v0.json` 中的同理：

> *"EXPERIMENTAL v0.x。CLI 路线图的 Phase 3。从在 Sentinel 中实证验证的双重审计模式结晶而来。"*

三者共享一个明确声明：*不经过第二个领域，不进行结晶*。这是框架原则 #12，不是以治理文档中的独立段落形式出现，而是**直接写在 JSON Schema 内部，在任何标准验证器都会读取、任何 schema 浏览工具都会向用户呈现的字段中**。框架不隐藏其 schema 的临时性质；它把这一点放在任何人都无法错过的地方。

这比看起来更重要。schema 生态系统（JSON Schema、OpenAPI、Avro）的惯常做法是发布 `v1`，然后随着变化升版。用 `$comment` 标注 `v0` 并声明"这是实验性的，直到第二个领域验证它"，是从一开始就承认第一次结晶并非最终结论。schema 本身是诚实的。

---

## 4. 命令的三种模式

`straymark validate` 不再是一个只有单一流程的命令。Phase 2 之后，它拥有三种运行模式，每种模式都与生命周期中的一个时刻相耦合：

- **`all` 模式（默认）** —— 不带 flag 的 `straymark validate`。递归遍历 `.straymark/`，解析每个 `.md` 文件，尝试解析其类型（AILOG、AIDEC、ADR、ETH、REQ、TES、INC、TDE、MCARD、SBOM、SEC、DPIA、Charter），应用相应的 schema，并逐文件报告违规。这是*完整审计*模式。适用于 CI，适用于检查新的仓库，适用于采用者想衡量积累了多少形式债务的场景。

- **`--staged` 模式** —— `straymark validate --staged`。替代 bash 的那个。仅验证 `git diff --cached --name-only` 报告为已暂存且位于 `.straymark/` 内的文件。它的自然归宿是 `.git/hooks/pre-commit`，自 fw-4.6.0 起，`straymark init --hooks` 会自动安装它。这是*运行门控*模式 —— 防止损坏文档进入 `main` 的那个。

- **`--check-pending-reviews` 模式** —— `straymark validate --check-pending-reviews`。三者中最微妙的，也是值得更多篇幅介绍的那个。

在 Phase 2 之前，智能体在 frontmatter 中标记为 `review_required: true` 的文档（低置信度的 AIDEC、高风险的 AILOG 等）会等待*人工批准*。但没有正典的方式来*发出*该批准信号。操作者阅读文档，在心里批准，继续前进。没有结构性的批准痕迹。这正是 Issue [#67](https://github.com/StrangeDaysTech/straymark/issues/67)。

`--check-pending-reviews` 关闭了这个漏洞。该命令遍历所有带有 `review_required: true` 的文档，列出它们的 `confidence`、`risk_level`、作者（人类或智能体）以及日期。批准是一个将 `review_required: true` → `review_required: false` 的提交，提交信息中带有人类联合署名 —— 而 `validate --check-pending-reviews` 就会停止列出该文档。批准是一个*正典信号*，而非与自己的口头约定。

这是关闭智能体循环的部分。框架允许智能体在没有事先人工批准的情况下创建文档（第 1 篇，属性 #2：*"无阻断控制的文化许可"*）；但在适当时候将其标记为待审；现在有了一个命令来列出待处理项，没有逃生舱口。没有审计文档会无声地丢失。

---

## 5. Phase 2 关闭实证循环

完整的 CHANGELOG 句子值得保留，因为它凝练了本次发布的意义：

> *"本次发布关闭了 Sentinel 实验打开的实证循环：Charter 关闭时的遥测、Charter 关闭时的漂移检测，以及针对 `review_required: true` 文档的正典批准信号（解决 issue #67）。"*

三个部分，关闭了一段始于六 Plan 实验（第 3 篇）的弧线：

1. **Charter 关闭时的遥测** —— Sentinel 在实验中手工填写的 YAML 块，成为了可验证的 `charter-telemetry.schema.v0.json`。
2. **Charter 关闭时的漂移检测** —— 145 行的 bash 脚本（`check-plan-drift.sh`）成为了 `straymark charter drift` 子命令，集成进验证流程。
3. **正典批准信号** —— 操作者在心里完成的事情，变成了一个具有结构性后果的命令：`--check-pending-reviews`。

这是博客中反复出现的模式：每次框架将 Sentinel 手工实践的内容结晶成命令，都是在实证验证该实践有效之后才进行的。这次的不同之处在于，**验证该结晶的命令是包的一部分**。Phase 2 之前，Sentinel 有模板、声明了 schema，以及文档遵循 schema 的隐含承诺。Phase 2 之后，有了一个**检查**的命令。这不是能力上的变化；这是保证上的变化。

---

## 6. 原则 #12，转化为 schema

这篇文章的哲学部分 —— 也是我最想留存记录的部分 —— 是这样的：

三个 v0 schema 的 `$comment` 编码了框架自 4 月以来一直以散文形式声明的一个原则。第 4 篇将其命名为：*"schema 被有意标记为 `v0` 而非 `v1`，因为框架的原则 #12 规定，没有第二个领域的验证，schema 不会结晶。"* 第 7 篇将其应用于 TDE 设计：*"框架再次验证了原则 #12：没有第二个领域，schema 不会结晶。"* 第 10 篇将其应用于模式 1 和模式 2：*"文档毫不简化地说明：该模式在 Sentinel 中得到了实证验证。等待第二个领域，再进行更高层次的结晶。"*

Phase 2 做了不同的事情。**它不是声明原则；它把原则写进 schema 本身，写在任何 schema 消费者都会读到的地方。** `$comment` 不是营销语言；它是协议。任何 JSON Schema 2020-12 验证器都会暴露它；任何解析 schema 的 IDE 都会显示它。这是框架第一次将其形式化的临时性质本身也进行形式化。

如果要用一句话来总结：框架验证它所拥有的，知道它所拥有的是临时的，并将这一点以书面形式写在*验证发生的地方*。验证临时性是纪律。过早结晶并验证结晶物是神学。

---

## 7. 结语

我从这个过程中得出的四条结论：

1. **拥有 schema 与针对 schema 进行验证，并非同一回事。** 第一步是声明性的；第二步是运行性的。Phase 2 做的是第二步，而非第一步。两种框架模式之间的差异 —— *"我有承诺"* 与 *"我有检查承诺的命令"* —— 决定了采用者是否能信任所声明的内容。

2. **"手工 bash → 命令 flag"的模式是结构性的。** `--staged` 不是一个孤立的功能。它是 Phase 2 版本的同一模式 —— 第 4 篇为外部审计周期记录了它，第 10 篇为 Charter 演化 CLI 辅助工具记录了它：冗长的命令式 → 可读的声明式 → 版本化。每次框架用一个 flag 扫除一个 bash 脚本，它就提升了一层保证。

3. **`--check-pending-reviews` 关闭了智能体的循环。** 第 1 篇命名为无需预先批准的文化许可，在这里得到完成：智能体可以创建，但标记为待审的文档不会丢失 —— 命令会列出它们，直到人类通过明确的提交批准它们为止。没有这个信号，第 1 篇的文化许可就会是疏忽。

4. **声明的临时性 > 过早的形式化。** v0 schema 的 `$comment` 将原则 #12 变成协议，而非教条。任何消费 schema 的工具都知道该形式化是临时的。这是一个年轻框架所能拥有的最强结构性诚实：验证你所拥有的，而不假装你所拥有的是最终定论。

---

*锚点：PR [#27](https://github.com/StrangeDaysTech/straymark/pull/27) —— 原始 `validate` 版本（2026 年 3 月）。Phase 2：PR #73 至 #79 + PR [#80](https://github.com/StrangeDaysTech/straymark/pull/80) —— `fw-4.6.0 / cli-3.7.0`（合并于 2026-05-11）。Schemas：`dist/.straymark/schemas/{charter,charter-telemetry,audit-output}.schema.v0.json`。[Issue #67](https://github.com/StrangeDaysTech/straymark/issues/67) —— `--check-pending-reviews` 的起源。*

*本文档在生成式 AI 工具（Claude 4.7）的协助下撰写；内容的全部责任由人类作者承担。*
