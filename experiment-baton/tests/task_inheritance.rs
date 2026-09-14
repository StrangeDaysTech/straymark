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
        std::fs::remove_dir_all(&self.0).unwrap();
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
fn multiple_parents_remain_undeclared_even_when_their_verbs_agree() {
    for second in ["operate", "design"] {
        let p = Project::new();
        p.charter(1, "specs/001-example/spec.md", "work_verb: operate\n");
        p.charter(
            2,
            "specs/001-example/spec.md",
            &format!("work_verb: {second}\n"),
        );
        assert_eq!(task_route(&p.0, None), (None, Tier::Frontier));
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
