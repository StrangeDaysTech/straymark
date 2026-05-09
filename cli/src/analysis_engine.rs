use anyhow::Result;
use arborist::AnalysisConfig;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Directories to skip during source file walking
const EXCLUDED_DIRS: &[&str] = &[
    ".straymark",
    ".git",
    "node_modules",
    "target",
    "vendor",
    "dist",
    "build",
    ".venv",
    "__pycache__",
];

/// File extensions recognized for analysis (matches arborist "all" features)
const SOURCE_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "ts", "jsx", "tsx", "java", "go",
    "cs", "cpp", "cc", "cxx", "c", "h", "php", "kt", "swift",
];

/// Full analysis report
#[derive(Debug, Serialize)]
pub struct AnalysisReport {
    pub path: String,
    pub threshold: u32,
    pub functions: Vec<FunctionEntry>,
    pub summary: AnalysisSummary,
    pub warnings: Vec<String>,
}

/// A single function's metrics
#[derive(Debug, Serialize)]
pub struct FunctionEntry {
    pub file: String,
    pub name: String,
    pub line: u32,
    pub cognitive: u32,
    pub cyclomatic: u32,
    pub sloc: u32,
}

/// Aggregated summary statistics
#[derive(Debug, Serialize)]
pub struct AnalysisSummary {
    pub files_analyzed: usize,
    pub total_functions: usize,
    pub above_threshold: usize,
    pub above_threshold_pct: f64,
    pub max_cognitive: u32,
    pub max_cognitive_location: String,
    pub avg_cognitive: f64,
}

/// Walk source files recursively, excluding known non-source directories
fn walk_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_recursive(root, &mut files);
    files.sort();
    files
}

fn walk_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !EXCLUDED_DIRS.contains(&name) {
                    walk_recursive(&path, files);
                }
            }
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if SOURCE_EXTENSIONS.contains(&ext) {
                    files.push(path);
                }
            }
        }
    }
}

/// Analyze all source files under `root` and produce a report
pub fn analyze_path(root: &Path, threshold: u32) -> Result<AnalysisReport> {
    let files = walk_source_files(root);
    let config = AnalysisConfig {
        cognitive_threshold: Some(threshold as u64),
        include_methods: true,
    };

    let mut all_functions: Vec<FunctionEntry> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut files_analyzed: usize = 0;

    for file_path in &files {
        match arborist::analyze_file_with_config(file_path, &config) {
            Ok(report) => {
                files_analyzed += 1;
                let relative = file_path
                    .strip_prefix(root)
                    .unwrap_or(file_path)
                    .to_string_lossy()
                    .to_string();

                for func in report.functions {
                    all_functions.push(FunctionEntry {
                        file: relative.clone(),
                        name: func.name,
                        line: func.start_line as u32,
                        cognitive: func.cognitive as u32,
                        cyclomatic: func.cyclomatic as u32,
                        sloc: func.sloc as u32,
                    });
                }
            }
            Err(e) => {
                let relative = file_path
                    .strip_prefix(root)
                    .unwrap_or(file_path)
                    .to_string_lossy();
                warnings.push(format!("{}: {}", relative, e));
            }
        }
    }

    let total_functions = all_functions.len();
    let above_threshold: Vec<&FunctionEntry> = all_functions
        .iter()
        .filter(|f| f.cognitive > threshold)
        .collect();
    let above_count = above_threshold.len();
    let above_pct = if total_functions > 0 {
        (above_count as f64 / total_functions as f64) * 100.0
    } else {
        0.0
    };

    let (max_cog, max_loc) = all_functions
        .iter()
        .max_by_key(|f| f.cognitive)
        .map(|f| (f.cognitive, format!("{}:{}", f.file, f.name)))
        .unwrap_or((0, String::new()));

    let avg_cog = if total_functions > 0 {
        all_functions.iter().map(|f| f.cognitive as f64).sum::<f64>() / total_functions as f64
    } else {
        0.0
    };

    Ok(AnalysisReport {
        path: root.to_string_lossy().to_string(),
        threshold,
        functions: all_functions,
        summary: AnalysisSummary {
            files_analyzed,
            total_functions,
            above_threshold: above_count,
            above_threshold_pct: above_pct,
            max_cognitive: max_cog,
            max_cognitive_location: max_loc,
            avg_cognitive: avg_cog,
        },
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_empty_directory() {
        let dir = TempDir::new().unwrap();
        let report = analyze_path(dir.path(), 8).unwrap();
        assert_eq!(report.summary.files_analyzed, 0);
        assert_eq!(report.summary.total_functions, 0);
        assert!(report.functions.is_empty());
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn test_walk_excludes_ignored_dirs() {
        let dir = TempDir::new().unwrap();

        // Create files in excluded dirs
        for excluded in &["node_modules", "target", ".git"] {
            let sub = dir.path().join(excluded);
            fs::create_dir_all(&sub).unwrap();
            fs::write(sub.join("lib.rs"), "fn excluded() {}").unwrap();
        }

        // Create a valid source file at root
        fs::write(dir.path().join("main.rs"), "fn included() {}").unwrap();

        let files = walk_source_files(dir.path());
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("main.rs"));
    }

    #[test]
    fn test_single_rust_file() {
        let dir = TempDir::new().unwrap();
        let code = r#"
fn simple_add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
        fs::write(dir.path().join("lib.rs"), code).unwrap();

        let report = analyze_path(dir.path(), 8).unwrap();
        assert_eq!(report.summary.files_analyzed, 1);
        assert_eq!(report.summary.total_functions, 1);
        assert_eq!(report.functions[0].name, "simple_add");
        assert_eq!(report.functions[0].cognitive, 0);
    }

    #[test]
    fn test_threshold_filtering() {
        let dir = TempDir::new().unwrap();
        // A function with some complexity (nested ifs)
        let code = r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 100 {
                return x * 2;
            }
            return x + 1;
        }
        return x;
    }
    0
}

fn simple() -> i32 {
    42
}
"#;
        fs::write(dir.path().join("test.rs"), code).unwrap();

        let report = analyze_path(dir.path(), 2).unwrap();
        assert_eq!(report.summary.total_functions, 2);
        // complex should exceed threshold of 2, simple should not
        assert!(report.summary.above_threshold >= 1);
    }

    #[test]
    fn test_summary_statistics() {
        let dir = TempDir::new().unwrap();
        let code = r#"
fn a() -> i32 { 1 }
fn b() -> i32 { 2 }
fn c() -> i32 { 3 }
"#;
        fs::write(dir.path().join("funcs.rs"), code).unwrap();

        let report = analyze_path(dir.path(), 8).unwrap();
        assert_eq!(report.summary.total_functions, 3);
        assert_eq!(report.summary.above_threshold, 0);
        assert_eq!(report.summary.above_threshold_pct, 0.0);
        assert_eq!(report.summary.avg_cognitive, 0.0);
    }

    #[test]
    fn test_unsupported_extension_skipped() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("notes.txt"), "not code").unwrap();
        fs::write(dir.path().join("data.csv"), "a,b,c").unwrap();

        let files = walk_source_files(dir.path());
        assert!(files.is_empty());
    }
}
