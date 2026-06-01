# Polish Charter 模式 — StrayMark

> 一个 Etapa(或 SpecKit `Polish` 阶段)的关闭 Charter 是检测一类反复出现的反模式 — **"声明了表层但未接线"** — 的承重门,而该反模式是 user-story Charter 的测试套件系统性无法捕获的。

**语言**: [English](../../POLISH-CHARTER-PATTERN.md) | [Español](../es/POLISH-CHARTER-PATTERN.md) | 简体中文

---

## 状态

**v1 — 在 N=2 个独立域中已验证。** 两个轴,有意分开报告以免混淆:

- **独立域:2 个。** `StrangeDaysTech/sentinel`(Go 后端,CHARTER-19 → CHARTER-27,2026-05-22)与 `StrangeDaysTech/lnxdrive`(Rust Linux 云同步守护进程 + GTK 桌面,2026-05,[发现 #209](https://github.com/StrangeDaysTech/straymark/issues/209))。一个 Rust 桌面应用验证首先在 Go 后端中看到的模式,正是 [N-status 门](../../../ADOPTERS.md)所要求的强 cross-domain 信号。
- **出现次数:3 次。** Sentinel 浮现了原始子类(1–4);LNXDrive 浮现了一个性质上全新的出现 — 一次*已交付缓解措施的 cross-component 回归*(下方子类 5)。

CLI 结晶的 N=2 门(反映自 [`FOLLOW-UPS-BACKLOG-PATTERN.md`](FOLLOW-UPS-BACKLOG-PATTERN.md))**现已跨越**。约定 + 命名反模式仍是可移植的核心;机械化检查升级为 `straymark analyze declared-vs-wired` 子命令(由配置驱动的 set-difference,v0 范围在 cli-3.17.x+ 中交付 — 见[未决问题](#未决问题))。Adopter 仍可使用专门的 polish Charter 与(可选的)项目本地 CI 守卫在本地复制此发现。

---

## 何时适用此模式

一个 Etapa 的关闭 Charter(用 SpecKit 词汇:跟随 Foundation + N 个 user-story 阶段之后的 `Polish` 阶段 — 见 [`SPECKIT-CHARTER-BRIDGE.md`](SPECKIT-CHARTER-BRIDGE.md))常被当作美容清理:WCAG 审核、文案修订、推迟但不阻塞的剩余物。参考实现的经验信号更强 — 当 polish Charter 是 end-to-end 二进制对照已记录的操作员 runbook **首次被执行**的地方时,它会浮现一类潜伏回归,而带有 mock adapter(例如 `humatest`、`gomock`、内存事件总线)的集成测试 harness 会系统性地绕过这类回归。

当满足**任一**以下条件时,采用此模式(将 polish Charter 视为承重的债务检测门,而非美容清理):

- Etapa 交付了 ≥3 个 handler,其集成测试使用 mock adapter,绕过组合应用的 boot 路径(真实路由器、真实 middleware 链、真实 env-var 清单)。
- Etapa 引入了 ≥1 个表层制品,其声明位置与接线位置位于**不同文件或不同模块**(例如:在 `metrics/` 中声明的 metric instrument 与在 `handlers/` 中记录;在 quickstart 中记录的 env var 与在 fake adapter 中读取;HTML embed 引用了在别处注册的路由)。
- 操作员 runbook(`quickstart.md` §boot、§smoke、§verification)记录的命令从未对 Etapa HEAD 构建的二进制 end-to-end 执行过。
- Etapa 关闭一个 SpecKit `Polish` 阶段,早期阶段通过 `(T103 polish)` 类注解将工作推迟到此处。

低于这些阈值(Etapa 没有 mock adapter 集成测试、没有 cross-module 声明/接线分离、runbook 在 CI 中持续被执行)时,per-Charter 测试套件本身通常就足够 — 过早采用此模式只会增加 polish Charter 的开销而无回报。

---

## 形式

### 命名的反模式:声明了表层但未接线

模式的核心交付物是一个**命名的反模式**,polish Charter 一贯地浮现它,并且它可被还原为机械化检查:

> **声明了表层但未接线** — 当一个 feature 合约的某部分(文档、公共 API、metric 注册、HTML 模板正文、公共路由标记)宣告了一种行为,而本应实现该行为的另一部分(env-var 消费者、handler 调用、instrument 记录调用、路由注册器、prefix 列表)却没有实现时。声明位置与接线位置在代码库中相距甚远。工具与 review 流程都不会将二者关联。CI 各自隔离地测试两侧,绿色通过。

polish Charter 是**发现载体**:它是浮现此类回归的最廉价方法,因为它对二进制 end-to-end 执行已记录的 runbook,而不是对一个已被直接接线到声明位置的测试 harness 执行。

### 四个泛化子类

该反模式至少呈现为四个子类。每个子类映射到一个可在 CI 中编码的机械化检查(今日为 per-project;未来 v1 的 cross-project CLI 工具候选 — 见[未决问题](#未决问题))。该列表有意避免特定语言与 runtime;具体实例化随 stack 而变:

| # | 声明位置 | 接线位置 | 机械化检查 |
|---|---|---|---|
| 1 | 操作员 runbook(`quickstart.md`、`deploy/README.md`)中记录的 env var | 应用代码中的 env-var 消费者(`os.Getenv`、`process.env`、`ENV[]`) | 每个记录的 env var 都至少有一个消费位置 |
| 2 | metrics 包中声明的 metric instrument / 可观测性符号 | handler 或 worker 代码中的记录 / 递增调用位置 | 每个声明的 instrument 都至少有一个记录调用位置 |
| 3 | 从渲染的 HTML 或嵌入模板中引用的 URL(`<script src="/...">`、`<link href="/...">`) | 注册到同一 API 表面的路由 | 已服务 HTML 中的每个 `src=`/`href=` 都解析到一个已注册的路由 |
| 4 | 标记为公共-按-合约的路由(handler 文档注释、专用标记) | auth middleware 的公共前缀 / 公共路径列表中的条目 | 每个公共-按-合约的 handler 都有匹配的前缀条目 |
| 5 | 客户端声明的 IPC/RPC proxy 方法(D-Bus proxy、gRPC stub、REST 客户端)— **尤其是在缓解措施移除了服务端方法之后被重新引入的那种** | 真正实现该方法的服务端 / 守护进程接口 | 每个声明的 proxy 方法都解析到一个已实现的接口方法;cross-component 的 API 变更必须更新**所有**消费者 |

跨子类的统一一句话是:

> **每个声明的表层制品都至少有一个可从真实请求触达的接线位置。**

扩展该列表的 adopter(参考实现尚未浮现的新声明↔接线对)欢迎通过 issue 或 PR 贡献额外子类。

### 子类 5 命名:已交付缓解措施经由未更新的下游消费者发生回归

LNXDrive 将子类 5 浮现为一次*已交付缓解措施的回归,跨越了组件边界* — 比一个全新 gap 更尖锐的数据点。生产者(一个 D-Bus 守护进程)通过移除一个携带 token 的方法并交付一个 token-safe 替代,关闭了一个安全风险。一个独立的组件(一个 GTK 偏好设置客户端,经由不同构建系统编译)**仍在调用已移除的方法**并在客户端获取 token — 正是缓解措施已经消除的行为。

两个叠加因素使其对每一个现有 backstop 都不可见:

- **跨边界盲区。** 生产者与消费者位于不同的 crate,由不同的工具链(Cargo vs Meson)构建,仅在运行时经由总线连接。zbus/D-Bus proxy 在*运行时*而非编译时被验证 — 因此守护进程自己的测试通过,客户端干净编译,没有任何单一测试套件跨越该合约。
- **feature-gate 后的死代码。** 这个陈旧调用位于一个 `#[cfg(feature = "goa")]` 之后,而 `Cargo.toml` 从未定义该 feature。它被完全编译*掉* — 死代码既击败了 CI 也击败了代码 review,因为两者都不会执行一个未定义的 feature。首次激活该 feature 甚至浮现了一个从未编译过的潜伏类型错误:该路径从未接线的确凿证据。

捕获它的可读信号是 polish/审核的**ex-ante 合约检查** — 客户端声明的 proxy 方法与守护进程已实现接口的 diff。这将"每个声明的表层制品都有一个接线位置"的机械化检查推广到其 cross-component 推论:**生产者侧的 API 变更必须更新、或至少考虑该 API 的每一个声明消费者。** 将其操作化的 Charter 纪律存在于[模板指南](#相关)(#209.c):一个触及 cross-component API 的缓解措施在 `## 要修改的文件` 中列出*所有*消费者,使生产者的变更无法静默地孤立一个消费者。

### 为什么集成测试会遗漏这些

跨四个子类的共同原因是:标准集成测试 harness 通过测试 API(`humatest.NewTestAdapter`,其他 stack 中的等价物)直接挂载 handler,绕过了声明与接线在其中相连接的组合步骤。被测 handler 由测试 fixture 正确接线;被破坏的是生产组合。CI 的绿色信号反映的是"给定一个请求,handler 行为正确" — 而不是"请求能触达 handler",也不是"声明的制品从生产可触达"。

polish Charter 的手动 smoke(`./binary && curl <已记录的-recipe>`)重新引入了组合步骤,在它触及的第一个子类实例处浮现 gap。

---

## 采用流程

对于首次使用此模式关闭 Etapa 的 adopter:

1. **声明一个 polish Charter**,其范围明确为:(a) 对 Etapa HEAD 构建的 `./binary` end-to-end 执行已记录的操作员 runbook;(b) 针对 Etapa 引入的制品验证以上四个子类。将该 Charter 预算为 **L**(而非 XS/S/M)— 来自参考实现的经验证据是每次首次 polish 会话浮现 ~10 个 gap。
2. **预期会产生紧随其后的 Charter**,而非剩余的范围蔓延。polish Charter 浮现的每个 gap 都获得一个专门的 follow-on Charter(例如:server boot 修复、auth middleware 修复、fake provider 实现、instrument 记录调用接线)。polish Charter 不吸收它们 — 它对它们做 triage。
3. **原子化地更新操作员 runbook**,以记录文档 gap(§boot 中缺失的 env vars、与实现不匹配的 smoke 形状、fake adapter 中所声称但不存在的行为)。runbook 是测试规范;如果它错了,二进制与文档都会失去对齐。
4. **在 Etapa 关闭时,提交一份回顾**([`AIDEC`](../../../docs/contributors/WHAT-IS-A-CHARTER.md) 或等价物),按根因对浮现的 gap 分类:环境依赖腐烂、文档漂移、或"声明了表层但未接线"。更清晰的切分对预测哪些 CI 守卫(若有)能在 PR 时刻捕获每一类至关重要。
5. **可选地落地 CI 守卫**,针对 Etapa 中最常见的子类。参考实现落地了三个:full-chain boot test(捕获 runtime 变体的子类 3+4)、declared-vs-wired analyzer(静态地捕获子类 1+2;动态地捕获子类 3+4)、以及操作员 runbook smoke test(捕获 runbook 漂移)。analyzer 最具可移植性;boot test 的形态是 project-specific。

对于后续 Etapa 上的 adopter:相同流程,并预测随着项目本地 CI 守卫成熟、工程师内化四个子类,每次 polish Charter 的 gap 数量会下降。

---

## 参考实现

`StrangeDaysTech/sentinel` CHARTER-19(polish Charter,2026 年 5 月)→ CHARTER-27(AIDEC 后的 CI 守卫):

- polish Charter 会话在 ~6 小时内浮现了 **10 个不同的潜伏 gap**,催生 5 个 follow-on Charter(CHARTERs 20/21/22/23/24)加上 3 个推迟的 follow-up。其中两个 gap 是已交付到生产却从未真正工作过的 feature(US3 Preference Center 401 循环长达 10 天;7 个 OTel instrument 被声明但从未记录长达 10 天)。
- 根因回顾是 [AIDEC-2026-05-22-001](https://github.com/StrangeDaysTech/sentinel/pull/93)("采用 polish-Charter-as-debt-detection 模式 + Etapa 3 的 3 个预备 CI 守卫")。它按类别(环境腐烂、文档漂移、声明了表层但未接线)对 10 个 gap 分类,并将 Sentinel 承诺在打开下一个 Etapa 之前落地三个 CI 守卫。
- 三个 CI 守卫作为 [sentinel#94](https://github.com/StrangeDaysTech/sentinel/pull/94)(CHARTERs 25/26/27)落地:full-chain boot test、declared-vs-wired multipass analyzer(子类 2 完全接线;子类 1+3+4 为 follow-up 留作 stub)、操作员 runbook smoke test。

参考实现包含一个可证伪的预测:下一个 Etapa 的 polish Charter 将浮现 ~80% 更少的 gap。验证该预测(或在预测失败时浮现的新 gap 类别)是重新审视该模式 `v0 → v1` 升级的天然经验触发器。

发起此次讨论的 RFC 是 [straymark#199](https://github.com/StrangeDaysTech/straymark/issues/199),在 polish Charter 会话展开过程中以五条评论更新保留了经验链。

---

## 未决问题

这些问题在 v0 中未解决。该模式的未来修订,或 CLI helper,可能解决它们:

- **结晶为 `straymark analyze declared-vs-wired` CLI 子命令** — *N=2 门已跨越;v0 范围已确定。* 随着 LNXDrive 在第二个域中验证该模式,框架交付一个**由配置驱动的 set-difference** v0:操作员提供声明侧的 glob+regex 与接线侧的 glob+regex(regex 捕获组即符号名),命令报告声明了但未接线的符号(`D \ W`)。这在*任何* stack 上都是机械化可处理的,正因为 stack 特定知识存在于 adopter 的 regex 中而非 CLI 中 — 并且它直接捕获子类 5(客户端 D-Bus proxy 方法名 vs 服务端接口方法名)。**推迟到后续修订:** 子类 1–4 的基于 AST 的变体(env-var 文档、metric instrument、HTML embed、公共路由标记),它们需要按 stack 的解析器;以及 runtime/动态检查(full-chain boot、路由解析),它们本质上是 project-local。
- **子类枚举的完整性**。四个子类是参考实现所浮现的。额外候选:在迁移中声明但从未被应用代码读/写的数据库列;声明但从未被检查的 feature flag;定义但从未被序列化的 protobuf 字段。每个额外子类至少需要一次 adopter 的经验浮现才能进入正典。
- **与 `straymark charter close --polish-checklist` 的集成**。一个 polish 专用子命令可以浮现规范化清单(end-to-end 执行 runbook;验证每个声明制品都有接线位置;验证 env-var 清单匹配二进制的实际需求;验证 runbook 中引用的 CLI 工具都存在)。门:在 `declared-vs-wired` CLI 子命令落地之后,因为清单的最后一项将调用它。
- **按 stack 的实例化指南**。四个子类与语言无关;具体检查形态(Go 的 `analysis.Pass`、TypeScript 的 AST walker、Python 的 `ast` 模块等)与语言相关。该模式的未来修订可能将按 stack 的参考实现作为兄弟文档托管。
- **工作量预算校准**。参考实现观察到首次 polish Charter 约 ~10 gap。预测是这会随着项目本地守卫的成熟而急剧下降。该模式的 v1 可以发布来自 N≥2 数据点的预算指南(按 Etapa 成熟度的 XS/S/M/L)。

---

## 致谢

由 Sentinel adopter(N=1)通过 [issue #199](https://github.com/StrangeDaysTech/straymark/issues/199) 发起。经验基础:`StrangeDaysTech/sentinel` 中的 CHARTER-19 → CHARTER-27 链,回顾 [AIDEC-2026-05-22-001](https://github.com/StrangeDaysTech/sentinel/pull/93)。

经由 LNXDrive adopter(Rust 桌面,第二个独立域)的[发现 #209](https://github.com/StrangeDaysTech/straymark/issues/209)结晶为 **v1(N=2)**,它贡献了子类 5(已交付缓解措施经由未更新的下游消费者发生回归)并触发了 `analyze declared-vs-wired` 子命令。配套的[发现 #210](https://github.com/StrangeDaysTech/straymark/issues/210)增加了 `charter new` 的侦察纪律与 `CHARTER-FILES-EXIST` 验证规则。作者:José Villaseñor Montfort。

*本文档在生成式 AI 工具(Claude 4.7)的协助下产生;所有内容责任由人类作者承担。*

---

## 相关

- [SPECKIT-CHARTER-BRIDGE.md](SPECKIT-CHARTER-BRIDGE.md) — 定义此模式为之附加承重语义的 SpecKit `Polish` 阶段。
- [FOLLOW-UPS-BACKLOG-PATTERN.md](FOLLOW-UPS-BACKLOG-PATTERN.md) — 在同一 adopter 中验证的 v0 兄弟模式;共享用于 CLI 结晶的 N=1 → N=2 升级门。
- [EMERGENT-OBSERVATION-DESIGN.md](EMERGENT-OBSERVATION-DESIGN.md) — polish Charter 的债务检测角色在 Etapa 关闭表面实例化的元模式。
- [AGENT-RULES.md](AGENT-RULES.md) — 治理 follow-up 表面(`R<N> (new, not in Charter)`、TDE 提升)如何从 polish Charter 发现流入更广泛治理 backlog 的 agent 侧指令。

---

*StrayMark fw-4.20.0 | [Strange Days Tech](https://strangedays.tech)*
