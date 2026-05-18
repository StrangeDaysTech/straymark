---
slug: exploring-the-framework
title: Exploring the framework
authors:
  - jose
  - claude-opus-47
tags:
  - straymark
  - tui
  - cli
  - i18n
  - structural-visibility
date: 2026-04-27T00:00:00.000Z
description: The TUI that produced visibility without meaning to
---
## 1. The tool that two weeks later revealed the outlier

Post 8 of this blog covered a very specific episode: the discovery of the only framework file whose canonical language was Spanish. The operator didn't find it through systematic inspection; they found it by running `straymark explore --lang es` during an audit preparation and noticing, visually, that one template was written backwards relative to the rest. The line from that post:

> *"A new surface of the framework (the TUI) put all the files into simultaneous view, and the inconsistency became obvious — like tidying the bookshelf and noticing one book is shelved spine-backwards."*

This post covers the tool. The TUI `straymark explore` was born almost three weeks before the discovery it enabled — on 25 April 2026 in `cli-3.4.0` — and matured across four more releases, all closed in under 48 hours. This isn't a post about code. It's a post about a specific piece of tooling that ratifies a thesis the blog already named: new tools produce visibility without meaning to.

---

## 2. Why a TUI in a Markdown framework

By late April 2026, the framework had something like fifty governed Markdown files — `PRINCIPLES.md`, `AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, `QUICK-REFERENCE.md`, `SPECKIT-CHARTER-BRIDGE.md`, plus templates in `dist/.straymark/templates/`, plus audit prompts, plus the versions in three languages of each one (`i18n/es/`, `i18n/zh-CN/`). All on disk, all browsable with `find` and `cat`, all perfectly accessible.

And, at the same time, all invisible. For an adopter cloning the framework for the first time, the first question wasn't *"where is rule N?"*. It was *"what's here?"*. And a recursive `ls` doesn't answer that question — it overwhelms it. A `grep` requires knowing what to search for. An editor opens one file at a time. The framework had reached the size where its own surface had become opaque.

`straymark explore` was born precisely to fix that: a navigable surface that answered *"what's here?"* in three seconds. Not an IDE; not a Read-the-Docs-style system; not a static site. A TUI — terminal user interface — that runs on the adopter's repo, indexes the framework's canonical files, shows them grouped by category, and lets you read them with a colored Markdown viewer inside the terminal.

The decision to build a TUI and not a web app is aligned with two things the blog already argued. First, Post 4's principle #10 — *"StrayMark is not an LLM gateway"* — generalizes to *"StrayMark is not whatever already does something else"*. We don't compete with MkDocs or Material for MkDocs. Second, the A1 decision of the same Post 4: orchestration-only. The TUI doesn't require a server, doesn't require a build step, doesn't require a connection: it runs on the adopter's repo, where the framework already lives. It's the humblest piece that could solve the problem.

---

## 3. What landed in `cli-3.4.0`

[PR #57](https://github.com/StrangeDaysTech/straymark/pull/57) from 25 April, merged as `cli-3.4.0`, did three concrete things:

- Introduced the `straymark explore` command. Two-panel layout when the terminal has 100 columns or more: navigation on the left (30%), document on the right (70%). Narrower terminals collapse to a single panel. Status bar at the foot with relevant shortcuts.
- Indexed the framework's canonical files into a navigable hierarchy: governance docs (`AGENT-RULES.md`, `DOCUMENTATION-POLICY.md`, ...), templates (`dist/.straymark/templates/`), audit prompts, and the adopter's directories (`docs/charters/`, `.straymark/07-ai-audit/`, ...). Each node expandable with `Enter`.
- Wired in the i18n resolver. The `--lang <code>` option lets you run `straymark explore --lang es` and get all governance docs in Spanish *if* a translation is available. If not, silent fallback to canonical EN. The commit's literal description:

> *"Wire `language` config and a new `--lang` flag through the explore TUI so framework governance docs are served from `i18n/<lang>/` when a translation exists, falling back silently to English otherwise."*

The i18n resolver part matters more than the TUI itself. Until `cli-3.4.0`, the helper `resolve_localized_path` was used only by `straymark new` (to resolve which template to inject into the adopter in the right language). PR #57 extracted it into `cli/src/utils.rs:146` as a shared helper, so that **a single definition of how `i18n/<lang>/` overlays resolve** backs both the `new` command and the `explore` command. The behavior stays predictable for the adopter: if you translate a template into `i18n/es/`, the TUI will show it the same way generation will use it. No surprises.

Markdown rendering uses `pulldown-cmark` for the parser and `ratatui` widgets to paint. Color-coded syntax, code blocks with borders, tables with rounded borders, headings indented by level. It isn't flashy; it's readable.

---

## 4. Four refinements in thirty-six hours

What happened between 25 and 27 April is what Post 9 named months later as a property of the process: when the proposal is well-written, implementation is execution. Four more PRs, closed in under 36 hours, took the TUI from "works" to "done":

| PR | Tag | Time | What it added |
|---|---|---|---|
| [#60](https://github.com/StrangeDaysTech/straymark/pull/60) | cli-3.5.0 | 25 Apr 22:36 | **Live language switcher.** `L` key cycles the display language without leaving the TUI: `en → es → zh-CN → en`. The index rebuilds in-place. |
| [#61](https://github.com/StrangeDaysTech/straymark/pull/61) | cli-3.5.0 (same release) | 25 Apr 23:41 | **OS locale auto-detect.** If the adopter has no `config.yml`, the TUI reads `$LC_ALL` / `$LANG` and maps the POSIX locale (e.g. `es_MX.UTF-8` → `es`) onto the closest supported language. |
| [#62](https://github.com/StrangeDaysTech/straymark/pull/62) | cli-3.5.1 | 26 Apr 00:07 | **Metadata panel translated.** 25 new i18n entries for labels and titles, with visual padding to keep cross-language alignment. |
| [#63](https://github.com/StrangeDaysTech/straymark/pull/63) | cli-3.5.2 | 26 Apr 00:13 | **Keybinding cleanup.** Removed undocumented vim aliases `l` and `h` (the lowercase `l` clashed with the `L` language switcher). Kept documented `j/k/g/G/n/N`. |

The arc's speed repeats the pattern. The five PRs of fw-4.11.0 covered in Post 6 (rebrand to StrayMark) closed in forty-three minutes. The three audit-cycle releases covered in Post 4 closed in one day. The TUI curve is similar: the design was clear in PR #57, and refinements came as the operator started using the command frequently and noticed the frictions.

The anecdote worth recording is `cli-3.5.2`: the `l` and `h` keys it removed were vim aliases — the operator uses them out of muscle memory in their editor. But when the uppercase-`L` language switcher landed hours earlier, the lowercase `l` and the uppercase `L` shared the same physical key, and the visual transition was confusing. The fix was to remove the undocumented aliases: the full-letter spelling (`j/k/g/G`) stays; the abbreviations that duplicated functions go away. It's a small UX detail that illustrates how TUI decisions tune against real use, not UX theory.

---

## 5. What the TUI revealed two weeks later

Here the post picks up Post 8's thread from the other end. What `cli-3.4.0` added to the framework wasn't just a navigation command; it was a **surface of simultaneous visualization**. That is: before the TUI, the framework's files existed but were read one by one. After the TUI, they could be seen all at once, grouped, tagged, in the same presentation mode.

That changes what the operator can notice.

On 12 May, two and a half weeks after the `cli-3.4.0` merge, the operator ran `straymark explore --lang es` before an external audit cycle. They got to the "audit prompts" group. Opened it. And noticed something that had been there for a year: the root `audit-prompt.md` was written *in Spanish*, while every other canonical framework file had English in root and `i18n/es/` overlays. One file, backwards relative to the convention.

The detail wasn't found by systematic review. It was found by visual contrast. It's exactly what Post 8 §3 codified as thesis:

> *"New tools produce visibility."*

The TUI didn't cause the outlier — the outlier had existed since the first commit that ported the Sentinel skill `plan-audit` into the canonical framework (Post 3 records this). The TUI didn't look for the outlier either; it isn't an inconsistency-detection tool. All it did was put every file on the same screen, in the same presentation mode, and let the human eye notice what had been there from the start. The conclusion worth recording is structural: any framework that crosses fifty canonical files needs a surface of simultaneous visualization, not because the surface is pedagogically important in itself, but because without it certain regularities of the repo become unobservable.

---

## 6. Small technical decisions that matter

Three TUI design details worth recording, all coherent with framework principles the blog has already named:

**Feature flag `tui`.** The CLI's `Cargo.toml` declares the TUI as an optional dependency (`ratatui` + `crossterm` + `pulldown-cmark` live behind the `tui` flag, enabled by default). An adopter who only wants `init`, `update`, `validate` can build a binary without the TUI via `cargo build --no-default-features`. It closes a sub-architectural decision that recurs in the framework: give the adopter value by default, but let them choose a lighter binary when the TUI adds nothing to the flow.

**Lazy document loading.** The `DocIndex` keeps a `HashMap<PathBuf, Document>` that fills up as each node is opened, not at startup. For a repo with a hundred governance files, this means `straymark explore` boots in under 200ms and only reads disk when the operator hits `Enter`. The decision is technically trivial; what isn't trivial is that it was made consciously. The TUI doesn't compete with an IDE; it competes with `cat`. It has to feel just as immediate.

**History stack for hyperlinks.** When one framework document mentions another (e.g., `AGENT-RULES.md §3` refers to `DOCUMENTATION-POLICY.md §6`), the TUI lets you jump to the referenced one with a click — and back with `Esc`. Internally there's a navigation stack that restores the previous document's scroll position. It's the piece that turns reading the framework from *"I open a file, close it, open another"* into *"I follow a conversation between documents without losing the thread"*. The number of cross-references the framework has today — between principles, agent rules, schemas, templates — makes this navigation structurally important, not cosmetic.

---

## 7. Closing

What I took from the process, in four claims:

1. **A framework that crosses a certain size needs a surface of simultaneous visualization.** It isn't ergonomics; it's the condition for certain regularities of the repo to be observable. Post 8 documented the consequence; this post documents the tool.

2. **The TUI doesn't cause the discoveries; it enables them.** The audit-prompt outlier had existed since May of the previous year. All that changed in April was that a screen appeared where it could be seen next to its peers. The tool doesn't have opinions about content; it changes the conditions under which the human eye can notice regularities.

3. **When the design is clear, implementation is hours, not sprints.** Five PRs in thirty-six hours — language-aware, live switcher, OS detection, panel i18n, keybinding cleanup — are only possible because the design was closed in PR #57. It's the same pattern as Posts 4, 6 and 9.

4. **The most interesting technical decision can be a compile-time flag.** The `tui` feature flag explicitly declares that the TUI is optional, that the framework still works without it, that the adopter decides. That architectural humility is the difference between a CLI that assumes how the adopter works and a CLI that offers itself where it can be useful.

With this post the blog covers the `H-14` milestone the first batch left as explicit debt. The remaining candidate milestones that `PLAN-INVESTIGACION.md §1.43-55` recorded — `AGENTS.md` universal inject, full EN/ES/zh-CN i18n coverage, `straymark validate` as a formal layer, CLA assistant — stay available for future batches. No promised cadence, but recorded.

---

*Anchors: PR [#57](https://github.com/StrangeDaysTech/straymark/pull/57) — `cli-3.4.0` (language-aware explore, 25 Apr 2026). PRs [#60](https://github.com/StrangeDaysTech/straymark/pull/60) · [#61](https://github.com/StrangeDaysTech/straymark/pull/61) · [#62](https://github.com/StrangeDaysTech/straymark/pull/62) · [#63](https://github.com/StrangeDaysTech/straymark/pull/63) — refinements `cli-3.5.0` to `cli-3.5.2` (25-26 Apr 2026). Code: `cli/src/tui/` (15 files). Adopter doc: `docs/adopters/CLI-REFERENCE.md` §`straymark explore`.*

*Co-written by José Villaseñor Montfort (Strange Days Tech) and Claude Opus 4.7 (1M context).*
