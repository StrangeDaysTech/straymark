---
slug: what-the-feature-flag-compiled-away
title: feature flag 编译掉的东西
authors:
  - jose
tags: [straymark, charters, governance, polish-charter, anti-pattern, adopters, lnxdrive]
date: 2026-05-31
description: 第二个 adopter，在另一种语言和另一套 stack 上，验证了「声明了表层但未连线」这个反模式 —— 而我们有意推迟的 CLI helper 终于发布了。
---
*在我们命名一个反模式并有意拒绝马上为它打造工具的一周后，第二个项目 —— 另一种语言、另一套 stack —— 以更锋利的形态再次把它浮现出来：这不是一个新的 gap，而是一项已经交付的安全缓解措施发生了回归，藏在一个从未定义过的 feature flag 背后的死代码里。这正是框架说自己在等待的 N=2 信号。被推迟的 helper 发布了。*

<!-- truncate -->

> *"它干净地编译通过了 —— GOA 模块是死代码，而 zbus proxy 在 runtime 验证，不在 compile time 验证，所以没有任何东西标出来。第一次激活这个 feature，甚至浮现了一个从来没被编译过的潜伏类型错误。"*

这句话来自 Issue [#209](https://github.com/StrangeDaysTech/straymark/issues/209)，由 [LNXDrive](https://github.com/StrangeDaysTech/lnxdrive) 在 5 月 31 日提交。LNXDrive 是一个 Linux 云同步 daemon 和 GTK 桌面客户端，用 Rust、FUSE、D-Bus、systemd 一路写到底。它是 [StrayMark adopters registry](https://github.com/StrangeDaysTech/straymark/blob/main/ADOPTERS.md) 里的第二个项目，也是第一个真正不同于 Sentinel Go backend 的领域。这个差异就是本文的全部重点。

八天前，在[《二进制藏不住的东西》](what-the-binary-couldnt-hide)中，我们记录了一个反模式 —— **声明了表层但未连线** —— 它在 Sentinel 的一次 polish Charter 中，六小时内浮现了十次。我们给它命名，把 pattern doc 作为 `fw-4.18.0` 发布，并且**有意推迟 CLI helper**，理由说得很直白：Sentinel 是 N=1。那四个子类只是一个 adopter、一个 stack 浮现出来的形状。基于一个领域失败模式打造的 tooling，往往会把那些选择固化成不能泛化的 framework 默认值。pattern doc 的 `## Open questions` 用一句话设下闸门：*"结晶为 `straymark analyze declared-vs-wired` CLI 子命令……闸门：N=2 adopters。"*

本文记录的，就是 N=2 到来时发生了什么 —— 以及为什么这一次比最初命名模式的那十几个案例更有分量。

## 把自己编译出去的回归

LNXDrive 的 Charter `CHARTER-01-road-to-v0-1-0-alpha-1` 有一个 Fase 1，关闭了一个安全风险 `RISK-002`。这个 fix 的形状值得精确说清楚，因为回归正是它的镜像。

认证流程最初让 daemon 暴露一个 D-Bus 方法 `Auth.CompleteAuthWithTokens`，由客户端把 OAuth token 通过 bus 传过去。token 走 bus 就是风险。Fase 1 用正确方式关闭了它：**移除**这个方法，改为交付 `Auth.CompleteAuthViaGOA(account_path)` —— daemon 现在自己从 GNOME Online Accounts 获取 token，在 daemon 侧完成，token 永远不经过 bus。缓解措施交付了。daemon 的测试通过了。`RISK-002` 关闭了。

然后 Fase 3 审计 GTK4 preferences panel —— 这是一个单独的 crate，通过 Meson 编译，而不是通过 daemon 的 Cargo build。这个 panel **仍然在调用 `complete_auth_with_tokens`**，也仍然在 client-side 获取 token。当 producer 改变合同时，consumer 从未被更新。缓解措施跨过组件边界发生了回归，藏在 Fase 1 工作完全没有碰到的系统另一半里。

下面是它为什么能在 Fase 1 与 Fase 3 之间保持不可见 —— 以及为什么没有测试、没有编译器、也没有 reviewer 抓住它：

- **那次死调用藏在一个不存在的 feature flag 背后。**客户端的 GOA 代码路径位于 `#[cfg(feature = "goa")]` 之下。`Cargo.toml` 从未定义 `goa` feature。所以整个模块都被编译*出去*了 —— 它是死代码。crate 绿色构建，因为坏掉的路径根本没有被构建。CI 不会执行一个未定义的 feature；代码审查也不会，它读到那行然后继续往下走。当 Fase 3 的 panel 工作第一次真正激活这个 feature 时，它浮现了一个**从来没被编译过的潜伏类型错误** —— 这是具体且不可反驳的证据：这条路径从未真正连线过哪怕一次。
- **边界是三重的。**Producer 和 consumer 住在不同 crate 中，用不同 toolchain 构建（daemon 用 Cargo，panel 用 Meson），只在 runtime 通过 D-Bus 接合。而 zbus proxy 在 *runtime* 验证，不在 compile time 验证 —— 所以即使 feature 打开，声明了 `complete_auth_with_tokens` 的 proxy 也会针对一个已经不再提供该方法的 daemon interface 编译通过，只有当真实调用打到真实 bus 上时才会失败。

把 Rust 和 D-Bus 的具体细节拿掉，你得到的就是 Sentinel 那篇文章命名的同一个机械错误：一个 artifact 在某处被声明（客户端 proxy method），本应支撑它的实现住在另一处（daemon interface），而没有任何东西 —— 编译器没有，CI 没有，review 也没有 —— 把这两者关联起来。声明位置和连线位置相距太远，而回归就藏在它们之间。

## 为什么它比命名模式的十个 gap 更锋利

Sentinel 的案例是*新* gap：Preference Center 连续十天 401-loop，七个 OTel instrument 被声明却从未记录。feature 已经 ship，却从未真正工作。已经很糟。

LNXDrive 的案例在一个对 pattern canon 很重要的维度上更糟：它是一项**已经交付、已经审计过的缓解措施发生了回归。**`RISK-002` 原本被正确关闭。fix 是真的。然后它悄悄漂回不合规状态 —— 不是因为有人重新引入了有风险的方法，而是因为 API 的 *consumer* 在 *producer* 改变合同时从未更新。让这次回归变得可读的不是测试，而是审计中的**ex-ante contract check**：把客户端声明的 proxy method 与 daemon 实际实现的 interface 做 diff。

这为 pattern doc 赢得了第 5 个子类，也赢得了自己的名字：**已交付缓解措施通过未更新的 downstream consumer 发生回归。**最初四个子类都是「某个声明了的 surface 从未被连线」的变体。第五个增加了一道更锋利的边：「某个声明了的 surface *曾经*连线过，连线作为 fix 被移除，而另一个组件继续调用旧合同。」

关于计数需要说一句，因为 StrayMark 通过验证计数来结晶模式，诚实对待算术很重要。这里有两个轴，pattern doc 现在分别报告它们，避免混淆：

- **独立领域：2。**Sentinel（Go backend）和 LNXDrive（Rust desktop）。一个 Rust 桌面应用验证一个最初在 Go backend 里看到的 pattern，这是强的 cross-domain 信号 —— 比另一个 Go backend 强得多。这个轴才是打开 CLI 自动化的闸门。
- **发生次数：3。**Sentinel 的最初浮现，加上 LNXDrive 内部较早的 Fase 1 文档漂移，再加上这次 cross-component 回归。

闸门是领域轴，而现在它已经被跨过了。

## 我们说过正在等待的闸门

在 Sentinel 那篇文章里，我写过：面对一个鲜明发现，人的自然反应是过度构建回应；纪律恰好相反：先命名元模式，等至少两个 adopter 浮现它的形状之后再做工具。我也点明了具体风险 —— *"一个 CLI helper 会把 framework 绑定到一种 runtime，而那种 runtime 反映的是某个特定 stack 的失败模式。"*

第二个领域正好让我们绕开这个风险。已经发布的 helper —— [`cli-3.18.0`](https://github.com/StrangeDaysTech/straymark/releases/tag/cli-3.18.0) 里的 `straymark analyze declared-vs-wired` —— 不编码 Go，不编码 Rust，不编码 D-Bus，也不编码 HTTP。它是一个**由 config 驱动的 set difference**。你给它一个*声明*侧和一个*连线*侧，每一侧都是一个 `(glob, regex)` 对，每个 regex 的 capture group 就是 symbol name。它报告那些出现在声明侧、但没有出现在连线侧的 symbol：

```bash
straymark analyze declared-vs-wired \
  --declared-glob "client/**/*.rs"     --declared-pattern 'fn (\w+)' \
  --wired-glob    "daemon/**/*.rs"      --wired-pattern   'fn (\w+)'
```

拿 LNXDrive 的布局来跑，它会标出 `complete_auth_with_tokens` —— 在客户端 proxy 中声明，daemon interface 中缺席 —— 并以非零退出码结束。stack-specific 的知识完全住在 adopter 的两个 regex 中，并作为一个命名 profile 一次性提交到 `.straymark/config.yml`。framework 提供 set-difference 机器；adopter 提供在自己的 stack 里「declared」和「wired」分别是什么意思。这是 N=1 不可能产出的设计：只有一个领域时，你无法分辨工具中哪些部分是本质，哪些只是那个 stack 的偶然属性。第二个领域把它们分开了。

这就是闸门的全部论点：不是被断言，而是在野外被观察到。如果我们在 N=1 时就发布 helper —— 比如一个遍历 Go AST、查找已声明但未记录 OTel instrument 的 analyzer —— 它对一个通过 D-Bus 通信的 Rust 项目毫无用处，而我们会花一个 release cycle 去泛化一个过早建出来的东西。等待让我们付出了八天，换来的是一个 v0 表面诚实 cross-stack 的工具。

## 伴随发现，以及更便宜的 backstop

LNXDrive 同一天提交了第二个 issue，[#210](https://github.com/StrangeDaysTech/straymark/issues/210)。它更安静，但更广泛有用。它观察到 Charter 的 `## Files to modify` 小节是根据*假设过的*代码写的，而不是根据*读过的*代码写的 —— 所以反复出错。`RISK-002` 声明了一个新的 `dbus_iface.rs` 和一个 opaque `SessionHandle` 类型；二者都不存在，fix 实际交付在已经存在的 `service.rs` 中。`ISSUE-002` 命名了 `lnxdrive-config/src/parser.rs`；没有这个 crate，真正的 parser 是 `lnxdrive-core/src/config.rs`。一个 CI-hardening 条目指向的 engine workflow 位于 GitHub Actions 会静默忽略的子目录路径下 —— 它从未运行。每一行都在执行过程中变成了有记录的「premise correction」。

这正是 framework 作为 backstop 在工作：ex-ante 声明让后来的 drift *可读*。但根因在 drift 上游 —— 这是 authoring error，是一条从未存在过的 path，不是实现发生了偏离。它们是不同失败，把它们混在一起只会增加噪声。所以 `cli-3.17.0` 增加了一条 validate 规则 `CHARTER-FILES-EXIST`：当 `## Files to modify` 的某一行命名了一个磁盘上不存在、且没有标记为新建的 path 时，`straymark validate --include-charters` 会发出 warning。它是 warn-only、pure-Rust（因此不需要 drift check 所依赖的 bash），并且 —— 这是 #210 要求的重点 —— 它住在一个与 `charter drift` *不同的命令*里。一条从未存在的 path 是 Charter mis-declaration，应当就地修正；一条已声明但未修改的 path 是 implementation drift，应当 reconciliation。两个失败、两个 rule code、两个命令。Charter template 现在也在 `## Files to modify` 上方的注释里告诉作者：声明每条 path 之前先读它；当 Charter 触碰 cross-component API 时，列出*所有* consumer，而不只是 producer。最后这条纪律，如果当时存在，会在写任何代码之前就在 authoring time 抓住 GOA 回归。

## 我们仍然刻意没有做的事

和上次一样的克制，只是在另一个位置。`analyze declared-vs-wired` **只**发布第 5 个子类 —— IPC proxy-vs-interface check，也就是 LNXDrive 实际浮现的那个，以及仅靠两个 regex 就能跨 stack 机械处理的那个。第 1 到第 4 个子类的 AST-based 变体 —— 遍历代码查 env-var consumer、metric record site、HTML route resolution、public-route prefix —— 仍然被推迟，pattern doc 仍然把它们列在 `## Open questions` 下。它们需要 per-stack parser，而 regex capture 上的 set-difference 不是那个东西。动态检查 —— 启动 binary、在 runtime 解析 route —— 仍然天然是 project-local。我们跨过了 config-driven check 需要的闸门；没有把这次跨越当作许可证去构建 pattern 最终可能证明合理的一切。

还有一个较小的推迟也值得点名。我们考虑过在 Charter frontmatter 里加入硬字段 `recon: confirmed` —— 一个作者勾选的框，用来声明自己在 declaration 之前读过 tree。我们没有发布它。必填字段会破坏非交互式和 skill-driven 的 Charter 创建，而且它会制造一个说谎表面：agent 会反射性地勾选它。`charter new` 输出里的软提醒，加上机械 backstop `CHARTER-FILES-EXIST`，完成了同样工作，却没有那个失败模式。如果软版本在更多 adopter 中证明不够，那就是 revisiting 的信号 —— 同一套闸门逻辑，只是低一层。

## 如果你读到了这里

上一篇文章里的可移植练习依然成立，而第二个领域把它扩展了。如果你的系统被拆分成通过 runtime 接合的 producer 和 consumer —— daemon 和 client、service 和 SDK、server 和 generated stub —— 写下 consumer *声明*会调用的方法，再写下 producer *实际实现*的方法，然后 diff 这两个集合。你不需要 StrayMark；`grep` 和 `comm` 已经能走完大部分路。一个方向上的 set difference，就是本文讨论的回归。另一个方向上的 set difference，是可以删除的死 surface area。不管哪一边，那个数字都是你以前没有的信息。

如果你运行的是我们还没见过的 stack —— 而在 N=2 时，这仍然是绝大多数 stack —— 从「我项目里的有趣观察」到「被命名进 StrayMark canon」的路径，仍然走 #199 和 #209 都走过的同一个渠道：开一个 issue。第 6 个子类现在就藏在某个 codebase 里，在一个文件中声明、另一个文件中连线，等待那个会把它浮现出来的 adopter。

---

*StrayMark `fw-4.20.0` + `cli-3.17.0`（Release A）和 `cli-3.18.0`（Release B）—— Issues [#209](https://github.com/StrangeDaysTech/straymark/issues/209) · [#210](https://github.com/StrangeDaysTech/straymark/issues/210) · PRs [#211](https://github.com/StrangeDaysTech/straymark/pull/211) · [#212](https://github.com/StrangeDaysTech/straymark/pull/212)。Pattern：[`POLISH-CHARTER-PATTERN.md`](https://github.com/StrangeDaysTech/straymark/blob/main/dist/.straymark/00-governance/POLISH-CHARTER-PATTERN.md)（v1，N=2）。前篇：[《二进制藏不住的东西》](what-the-binary-couldnt-hide)。*

*本文借助生成式 AI 工具（Claude Opus 4.8）完成；内容责任全部归人类作者所有。*
