//! Ratified #332 task inheritance, using only an explicit, unique Charter origin.
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use straymark_baton::classify::TaskClass;
use straymark_baton::telemetry::build_report;
use straymark_baton::tiers::{Policy, Tier};
use straymark_baton::units::{inventory, Granularity};

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "baton-task-inheritance-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let project = Self(root);
        project.write("specs/001-example/spec.md", "# Synthetic specification\n");
        project.write("specs/001-example/tasks.md", "- [ ] T001 A task\n");
        project
    }

    fn write(&self, path: &str, content: &str) {
        let path = self.0.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn charter(&self, number: u8, origin: &str, declaration: &str) {
        self.write(
            &format!(".straymark/charters/{number:02}-example.md"),
            &format!("---\ncharter_id: CHARTER-{number:02}-example\nstatus: in-progress\neffort_estimate: M\ntrigger: synthetic\noriginating_spec: {origin}\n{declaration}---\n# Charter: Example\n"),
        );
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        // Never panic in Drop: a failing test is already unwinding.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn task_route(root: &Path, only: Option<Granularity>) -> (Option<TaskClass>, Tier) {
    let units = inventory(root, only);
    let (routes, _) = build_report(&units, &Policy::default());
    let task = routes.iter().find(|r| r.id == "001-example:T001").unwrap();
    (task.class, task.tier)
}

#[test]
fn explicit_parent_declaration_flows_through_inventory_classifier_and_router() {
    for (declaration, class, tier) in [
        (
            "work_verb: implement\ndesign_provenance: new\n",
            TaskClass::Implementer,
            Tier::Economic,
        ),
        (
            "work_verb: implement\ndesign_provenance: upstream\n",
            TaskClass::Operator,
            Tier::Local,
        ),
        ("work_verb: design\n", TaskClass::Planner, Tier::Frontier),
        ("work_verb: audit\n", TaskClass::Auditor, Tier::Economic),
        ("work_verb: operate\n", TaskClass::Operator, Tier::Local),
    ] {
        let p = Project::new();
        p.charter(1, "specs/001-example/spec.md", declaration);
        assert_eq!(task_route(&p.0, None), (Some(class), tier));
        assert_eq!(
            task_route(&p.0, Some(Granularity::Task)),
            (Some(class), tier)
        );
        let tasks = inventory(&p.0, Some(Granularity::Task));
        assert!(
            tasks[0].effort_estimate.is_none(),
            "parent effort is not task effort"
        );
    }
}

#[test]
fn canonical_paths_match_but_other_specs_do_not() {
    let p = Project::new();
    p.charter(
        1,
        "./specs/001-example/../001-example/spec.md",
        "work_verb: operate\n",
    );
    p.write("specs/002-other/spec.md", "# Other\n");
    p.write("specs/002-other/tasks.md", "- [ ] T001 Other task\n");
    let (routes, _) = build_report(&inventory(&p.0, None), &Policy::default());
    assert_eq!(
        routes
            .iter()
            .find(|r| r.id == "001-example:T001")
            .unwrap()
            .tier,
        Tier::Local
    );
    assert_eq!(
        routes
            .iter()
            .find(|r| r.id == "002-other:T001")
            .unwrap()
            .tier,
        Tier::Frontier
    );
}

#[test]
fn parents_that_all_agree_keep_their_tasks_classified() {
    // The charter-chain case: a later Charter for the same spec declaring the
    // same thing must not retroactively unclassify the spec's tasks.
    let p = Project::new();
    let declaration = "work_verb: implement\ndesign_provenance: new\n";
    p.charter(1, "specs/001-example/spec.md", declaration);
    p.charter(2, "specs/001-example/spec.md", declaration);
    assert_eq!(
        task_route(&p.0, None),
        (Some(TaskClass::Implementer), Tier::Economic)
    );
}

#[test]
fn parents_that_disagree_or_do_not_declare_leave_tasks_undeclared() {
    for second in [
        "work_verb: design\n",
        "",
        "design_provenance: new\n",
        // Same class today, different declaration: agreement is on what was
        // declared, not on what the current classifier happens to derive.
        "work_verb: operate\ndesign_provenance: new\n",
    ] {
        let p = Project::new();
        p.charter(1, "specs/001-example/spec.md", "work_verb: operate\n");
        p.charter(2, "specs/001-example/spec.md", second);
        assert_eq!(task_route(&p.0, None), (None, Tier::Frontier), "{second:?}");
    }
}

#[test]
fn missing_parent_or_missing_spec_does_not_inherit() {
    let p = Project::new();
    assert_eq!(task_route(&p.0, None), (None, Tier::Frontier));
    p.charter(1, "specs/001-example/spec.md", "work_verb: operate\n");
    std::fs::remove_file(p.0.join("specs/001-example/spec.md")).unwrap();
    assert_eq!(task_route(&p.0, None), (None, Tier::Frontier));
}

#[test]
fn invalid_or_absent_verb_never_uses_titles_or_task_frontmatter() {
    for declaration in ["", "work_verb: invalid\n", "design_provenance: upstream\n"] {
        let p = Project::new();
        p.charter(1, "specs/001-example/spec.md", declaration);
        p.write(
            "specs/001-example/tasks.md",
            "---\nwork_verb: operate\n---\n- [ ] T001 operate audit design\n",
        );
        assert_eq!(task_route(&p.0, None), (None, Tier::Frontier));
    }
}

#[test]
fn invalid_provenance_cannot_downgrade_implementation() {
    let p = Project::new();
    p.charter(
        1,
        "specs/001-example/spec.md",
        "work_verb: implement\ndesign_provenance: invalid\n",
    );
    assert_eq!(
        task_route(&p.0, None),
        (Some(TaskClass::Implementer), Tier::Economic)
    );
}

#[test]
fn malformed_charter_cannot_hide_a_competing_parent() {
    let p = Project::new();
    p.charter(1, "specs/001-example/spec.md", "work_verb: operate\n");
    p.write(
        ".straymark/charters/02-broken.md",
        "---\ncharter_id: [invalid\n---\n",
    );
    assert_eq!(task_route(&p.0, None), (None, Tier::Frontier));
}

/// A Charter the typed parser rejects (here an out-of-schema effort estimate)
/// still has readable frontmatter: it must count as a parent, neither hidden
/// nor disabling inheritance for the whole project.
fn typed_invalid_charter(p: &Project, number: u8, declaration: &str) {
    p.write(
        &format!(".straymark/charters/{number:02}-typed-invalid.md"),
        &format!("---\ncharter_id: CHARTER-{number:02}-typed-invalid\nstatus: in-progress\neffort_estimate: XXL\ntrigger: synthetic\noriginating_spec: specs/001-example/spec.md\n{declaration}---\n"),
    );
}

#[test]
fn a_typed_invalid_charter_is_still_a_parent() {
    let p = Project::new();
    typed_invalid_charter(&p, 1, "work_verb: operate\n");
    assert_eq!(
        task_route(&p.0, None),
        (Some(TaskClass::Operator), Tier::Local)
    );
}

#[test]
fn a_typed_invalid_charter_can_still_compete() {
    let p = Project::new();
    p.charter(1, "specs/001-example/spec.md", "work_verb: operate\n");
    typed_invalid_charter(&p, 2, "work_verb: design\n");
    assert_eq!(task_route(&p.0, None), (None, Tier::Frontier));
}

#[test]
fn cli_says_why_an_unreadable_charter_disables_inheritance() {
    let p = Project::new();
    p.charter(1, "specs/001-example/spec.md", "work_verb: operate\n");
    p.write(
        ".straymark/charters/02-broken.md",
        "---\ncharter_id: [invalid\n---\n",
    );
    let run = std::process::Command::new(env!("CARGO_BIN_EXE_straymark-baton"))
        .args(["classify", ".", "--granularity", "task", "--out", "json"])
        .current_dir(&p.0)
        .output()
        .unwrap();
    assert!(run.status.success());
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("task inheritance disabled"), "{stderr}");
    assert!(
        stderr.contains(".straymark/charters/02-broken.md"),
        "{stderr}"
    );
    // The JSON on stdout stays machine-readable.
    let json: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap();
    assert_eq!(json[0]["class"], "undeclared");
}

#[cfg(unix)]
#[test]
fn a_spec_symlink_outside_the_project_does_not_inherit() {
    let p = Project::new();
    let outside = Project::new();
    let spec = p.0.join("specs/001-example/spec.md");
    std::fs::remove_file(&spec).unwrap();
    std::os::unix::fs::symlink(outside.0.join("specs/001-example/spec.md"), &spec).unwrap();
    p.charter(1, "specs/001-example/spec.md", "work_verb: operate\n");
    assert_eq!(task_route(&p.0, None), (None, Tier::Frontier));
}

#[test]
fn cli_inherits_without_mutation_and_still_requires_dry_run() {
    fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut files = Vec::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(dir) = pending.pop() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    pending.push(path);
                } else {
                    files.push((path.clone(), std::fs::read(path).unwrap()));
                }
            }
        }
        files.sort();
        files
    }
    let p = Project::new();
    p.charter(1, "specs/001-example/spec.md", "work_verb: implement\n");
    let before = snapshot(&p.0);
    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_straymark-baton"))
            .args(args)
            .current_dir(&p.0)
            .output()
            .unwrap()
    };
    let classified = run(&["classify", ".", "--granularity", "task", "--out", "json"]);
    assert!(classified.status.success());
    let json: serde_json::Value = serde_json::from_slice(&classified.stdout).unwrap();
    assert_eq!(json[0]["class"], "implementer");
    assert!(run(&["route", ".", "--dry-run", "--out", "json"])
        .status
        .success());
    assert!(!run(&["route", "."]).status.success());
    assert_eq!(before, snapshot(&p.0));
}
