//! Ratified #332 batch placement (#428): a batch's own ledger line overrides
//! its parent — the AILOG's frontmatter, else the Charter(s) whose ledger the
//! AILOG is (the link `charter batch-complete` writes through), when they agree.
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use straymark_baton::classify::TaskClass;
use straymark_baton::telemetry::build_report;
use straymark_baton::tiers::{Policy, Tier};
use straymark_baton::units::{inventory, Granularity};

static NEXT: AtomicU64 = AtomicU64::new(0);

const AILOG: &str = "AILOG-2026-09-13-001";

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "baton-batch-inheritance-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn write(&self, path: &str, content: &str) {
        let path = self.0.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    /// A ledger AILOG whose frontmatter carries `frontmatter` and whose ledger
    /// is `ledger` (the `## Batch Ledger` body).
    fn ailog(&self, file: &str, frontmatter: &str, ledger: &str) {
        let id = file.split('-').take(5).collect::<Vec<_>>().join("-");
        self.write(
            &format!(".straymark/07-ai-audit/agent-logs/{file}.md"),
            &format!("---\nid: {id}\ntitle: Synthetic\n{frontmatter}---\n\n# AILOG\n\n## Batch Ledger\n\n{ledger}"),
        );
    }

    fn charter(&self, number: u8, links: &str, declaration: &str) {
        self.write(
            &format!(".straymark/charters/{number:02}-example.md"),
            &format!("---\ncharter_id: CHARTER-{number:02}-example\nstatus: in-progress\neffort_estimate: M\ntrigger: synthetic\n{links}{declaration}---\n# Charter\n"),
        );
    }

    fn batches(&self) -> Vec<(String, Option<TaskClass>, Tier)> {
        let units = inventory(&self.0, Some(Granularity::Batch));
        let (routes, _) = build_report(&units, &Policy::default());
        routes
            .into_iter()
            .map(|r| (r.id.rsplit('#').next().unwrap().to_string(), r.class, r.tier))
            .collect()
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        // Never panic in Drop: a failing test is already unwinding.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn b(n: u8, class: Option<TaskClass>, tier: Tier) -> (String, Option<TaskClass>, Tier) {
    (format!("batch-{n}"), class, tier)
}

#[test]
fn issue_428_reproduction_ledger_line_overrides_ailog_frontmatter() {
    let p = Project::new();
    p.ailog(
        AILOG,
        "work_verb: operate\n",
        "### Batch 1 — Synthetic\n- **Work verb**: design\n\n### Batch 2 — Homogeneous\n(pending)\n",
    );
    assert_eq!(
        p.batches(),
        [
            b(1, Some(TaskClass::Planner), Tier::Frontier),
            b(2, Some(TaskClass::Operator), Tier::Local),
        ]
    );
}

#[test]
fn batches_inherit_from_the_charter_whose_ledger_the_ailog_is() {
    for links in [
        format!("originating_ailogs: [{AILOG}]\n"),
        // Spec-originated Charter: batch-complete writes to execution_ailogs[0].
        format!("originating_spec: specs/001/spec.md\nexecution_ailogs: [{AILOG}-slug]\n"),
    ] {
        let p = Project::new();
        p.charter(1, &links, "work_verb: implement\ndesign_provenance: new\n");
        p.ailog(&format!("{AILOG}-ledger"), "", "### Batch 1 — One\n");
        assert_eq!(
            p.batches(),
            [b(1, Some(TaskClass::Implementer), Tier::Economic)],
            "{links}"
        );
    }
}

#[test]
fn only_the_ledger_link_counts() {
    let p = Project::new();
    // Listed, but not first: batch-complete would not write this Charter's
    // ledger here. And `…-001b` is a different AILOG from `…-001`.
    p.charter(
        1,
        "originating_ailogs: [AILOG-2026-01-01-001, AILOG-2026-09-13-001]\n",
        "work_verb: operate\n",
    );
    p.charter(2, "originating_ailogs: [AILOG-2026-09-13-001b]\n", "work_verb: operate\n");
    p.ailog(AILOG, "", "### Batch 1 — One\n");
    assert_eq!(p.batches(), [b(1, None, Tier::Frontier)]);
}

#[test]
fn charter_parents_must_agree() {
    for (second, expected) in [
        ("work_verb: audit\n", b(1, Some(TaskClass::Auditor), Tier::Economic)),
        ("work_verb: design\n", b(1, None, Tier::Frontier)),
        ("", b(1, None, Tier::Frontier)),
    ] {
        let p = Project::new();
        p.charter(1, &format!("originating_ailogs: [{AILOG}]\n"), "work_verb: audit\n");
        p.charter(2, &format!("execution_ailogs: [{AILOG}]\n"), second);
        p.ailog(AILOG, "", "### Batch 1 — One\n");
        assert_eq!(p.batches(), [expected], "{second:?}");
    }
}

#[test]
fn the_ailog_frontmatter_is_nearer_than_the_charter() {
    let p = Project::new();
    p.charter(1, &format!("originating_ailogs: [{AILOG}]\n"), "work_verb: design\n");
    p.ailog(AILOG, "work_verb: audit\n", "### Batch 1 — One\n");
    assert_eq!(p.batches(), [b(1, Some(TaskClass::Auditor), Tier::Economic)]);
}

#[test]
fn an_override_replaces_the_whole_declaration() {
    let p = Project::new();
    p.ailog(
        AILOG,
        "work_verb: implement\ndesign_provenance: upstream\n",
        "\
### Batch 1 — Own verb, no provenance: not the parent's `upstream`
- **Work verb**: implement

### Batch 2 — A provenance line alone qualifies nothing
- **Design provenance**: new

### Batch 3 — Both, in either order
- **Design provenance**: upstream
- **Work verb**: implement
",
    );
    assert_eq!(
        p.batches(),
        [
            b(1, Some(TaskClass::Implementer), Tier::Economic),
            b(2, Some(TaskClass::Operator), Tier::Local),
            b(3, Some(TaskClass::Operator), Tier::Local),
        ]
    );
}

#[test]
fn examples_and_other_sections_never_declare_a_batch() {
    let p = Project::new();
    p.ailog(
        AILOG,
        "",
        "\
<!-- Heterogeneous batch? Add:
- **Work verb**: design
-->
### Batch 1 — One
Notes.

#### Details
- **Work verb**: operate

```markdown
### Batch 9 — Quoted
- **Work verb**: audit
```

## Modified Files
- **Work verb**: operate
",
    );
    assert_eq!(p.batches(), [b(1, None, Tier::Frontier)]);
}

#[test]
fn an_unreadable_ailog_frontmatter_blocks_inheritance_but_not_overrides() {
    let p = Project::new();
    p.charter(1, &format!("originating_ailogs: [{AILOG}]\n"), "work_verb: operate\n");
    p.write(
        &format!(".straymark/07-ai-audit/agent-logs/{AILOG}.md"),
        "---\nwork_verb: [broken\n---\n### Batch 1 — One\n\n### Batch 2 — Two\n- **Work verb**: audit\n",
    );
    assert_eq!(
        p.batches(),
        [
            b(1, None, Tier::Frontier),
            b(2, Some(TaskClass::Auditor), Tier::Economic),
        ]
    );
}

#[test]
fn an_unreadable_charter_blocks_only_charter_inheritance() {
    let p = Project::new();
    p.charter(1, &format!("originating_ailogs: [{AILOG}]\n"), "work_verb: operate\n");
    p.write(".straymark/charters/02-broken.md", "---\ncharter_id: [invalid\n---\n");
    p.ailog(AILOG, "", "### Batch 1 — One\n");
    p.ailog(
        "AILOG-2026-09-13-002",
        "work_verb: audit\n",
        "### Batch 1 — AILOG-level declaration still applies\n",
    );
    let units = inventory(&p.0, Some(Granularity::Batch));
    let verbs: Vec<_> = units.iter().map(|u| u.work_verb.as_deref()).collect();
    assert_eq!(verbs, [None, Some("audit")]);
}
