# StrayMark - 采用者反馈

**如何宣告你的采用，并向上游发送遥测数据与发现。**


---

## 目录

1. [为什么反馈很重要](#为什么反馈很重要)
2. [两个渠道](#两个渠道)
3. [遥测数据是什么——以及它存放在哪里](#遥测数据是什么以及它存放在哪里)
4. [如何分享](#如何分享)
5. [N=1 → N=2 关口](#n1--n2-关口)
6. [快速参考](#快速参考)

---

## 为什么反馈很重要

StrayMark 不会自动收集任何数据。没有远程端点，没有 opt-in 信标，没有采用情况仪表盘。框架靠
**采用者主动选择发送的证据**演进——这意味着下一个版本的质量，直接取决于真实项目反馈了什么。

fw-4.13 到 fw-4.19 之间发布的大多数模式，都来自单一采用者
（[Sentinel](https://github.com/StrangeDaysTech/sentinel)）在许多 Charter 中持续向上游反馈发现。
当更多项目、来自更多领域地这样做时，框架会变得更好。

## 两个渠道

反馈有两种不同的性质，因此有两个归宿：

| | **宣告** | **发现** |
|---|---|---|
| **位置** | Discussions → **Adopters** 类别 | **Issues**（`Adopter feedback / upstream finding` 模板） |
| **时机** | 一次，在采用时 | 持续进行，随着你的发现 |
| **内容** | 你的项目、技术栈、版本、承诺发送的内容 | 一个具体的缺口、摩擦、缺陷或模式候选——有遥测数据支撑 |
| **生命周期** | 作为你的采用记录保持开启 | 处理完后关闭 |

先开一个 [Adopters 讨论](https://github.com/StrangeDaysTech/straymark/discussions/new?category=adopters)；
然后将每个发现 Issue **交叉链接**回它。这个链接正是把一个发现与已知采用者及其 N 上下文绑定起来的纽带。

## 遥测数据是什么——以及它存放在哪里

当你关闭一个 Charter（`straymark charter close`）时，StrayMark 会把结构化遥测数据记录到
`.straymark/charters/CHARTER-NN.telemetry.yaml`——估算准确度（按时间，而非代码行数）、智能体行为、
外部审计结果、范围变更，以及定性的收获/摩擦点。其结构由
[`charter-telemetry.schema.v0.json`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/schemas/charter-telemetry.schema.v0.json)
定义。

**这个文件留在你的仓库里。**它不会被传输到任何地方。向上游分享它，始终是你这一方刻意的、手动的行为。

## 如何分享

1. **决定哪些是相关的。**很少需要整个遥测文件——有用的部分通常是一个能支撑某个具体论点的代码块
   （一个 `effort` 偏差、一个 `external_audit` 差异、一份 `qualitative.friction_points` 列表）。
2. **匿名化。**在它离开你的仓库之前，去掉任何敏感信息——内部名称、密钥、私有仓库路径。
3. **附到一个发现上。**把摘录粘贴到 *Adopter feedback / upstream finding* Issue 的遥测字段
   （以 YAML 渲染），并链接你的采用讨论。

经你同意，维护者可以把多个项目的发现匿名化并聚合到博客文章或文档中——但只限于你已选择公开的部分。

## N=1 → N=2 关口

StrayMark 按**独立验证次数**来固化模式：

- **N=1**——在单一项目/领域中观察到的模式。已记录，但保持**手动**。
- **N=2**——第二次独立验证，最好是在**不同的领域**。这是证明把该模式**自动化**进 CLI 合理性的关口。

一个用 Rust 写的桌面应用，去验证一个最初在 Go 后端观察到的模式，远比另一个 Go 后端更强的 N=2。
所以当你宣告时——以及当你提交发现时——请说明你是否在验证一个已有模式，以及来自哪个领域。这一条上下文，
往往是采用者所能贡献的最有价值的东西。

## 快速参考

| 你想要… | 这样做 |
|---|---|
| 宣告你的采用 | 开一个 [Adopters 讨论](https://github.com/StrangeDaysTech/straymark/discussions/new?category=adopters) |
| 报告缺口 / 摩擦 / 模式 | 用 *Adopter feedback / upstream finding* 模板开一个 Issue |
| 用数据支撑发现 | 把匿名化的 `charter_telemetry:` 摘录粘贴进 Issue |
| 进入注册表 | 先宣告；维护者会把你加入 [`ADOPTERS.md`](https://github.com/StrangeDaysTech/straymark/blob/main/ADOPTERS.md) |

另见：[采用指南](ADOPTION-GUIDE.md) · [推荐工作流](WORKFLOWS.md) · [CLI 参考](CLI-REFERENCE.md)

---

*StrayMark — 因为每一次变更都讲述着一个故事。*

[Strange Days Tech](https://strangedays.tech)
