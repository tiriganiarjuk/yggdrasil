use saga_core::qa_path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub tool: String,
    pub code: String,
    pub severity: String,
    pub file: String,
    pub line: usize,
    pub column: Option<usize>,
    pub message: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub signal: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub canary: Option<String>,
}

/// A file that was not fully examined: an instrument could not do its work on it,
/// or its sidecar could not be used. It is not known to be clean.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fault {
    pub file: String,
    pub tool: String,
    pub mode: String,
    pub class: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub directory: String,
    pub files_scanned: usize,
    pub issues: Vec<Issue>,
    pub faults: Vec<Fault>,
    pub by_tool: HashMap<String, usize>,
    pub by_severity: HashMap<String, usize>,
    pub by_file: HashMap<String, usize>,
    pub by_code: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SagaResult {
    pub success: bool,
    pub files_analyzed: usize,
    pub not_fully_examined: usize,
    pub output: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SvalFileTreeEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub has_sidecar: bool,
    pub issue_count: usize,
}

// ============================================================================
// Sidecar Reading — the .qa format is saga_core's; this only flattens it
// ============================================================================

fn non_empty(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

/// Every sidecar under a directory, by saga's own walk — the one saga writes them
/// with and syn finds them with. A tree that cannot be listed whole is an error
/// naming what could not be read.
fn find_sidecars(directory: &str, include_tests: bool) -> Result<Vec<PathBuf>, String> {
    let left_out: &[&str] = if include_tests { &[] } else { &["tests"] };
    saga_runner::find_files(Path::new(directory), left_out, &|name| name.starts_with('.') && name.ends_with(".qa"))
        .map_err(|failure| failure.standard_message())
}

/// One sidecar's issues and faults. The sidecar is read by saga's own loader, so a
/// sidecar that cannot be read, or does not match the sidecar schema, comes back
/// as a fault naming it.
fn read_sidecar(path: &Path) -> (Vec<Issue>, Vec<Fault>) {
    let report = match saga_runner::load_qa_file(path) {
        Ok(report) => report,
        Err(failure) => {
            let unusable = Fault {
                file: path.display().to_string(),
                tool: "svalinn".to_string(),
                mode: failure.mode.to_string(),
                class: failure.class.to_string(),
                message: failure.standard_message(),
            };
            return (Vec::new(), vec![unusable]);
        }
    };

    let faults = report
        .faults
        .into_iter()
        .map(|fault| Fault {
            file: report.file.clone(),
            tool: fault.tool,
            mode: fault.mode,
            class: fault.class,
            message: fault.message,
        })
        .collect();
    let issues = report
        .issues
        .into_iter()
        .map(|i| Issue {
            tool: i.tool,
            code: i.code,
            severity: i.severity,
            file: report.file.clone(),
            line: i.line,
            column: i.column,
            message: i.message,
            detail: non_empty(i.detail),
            signal: non_empty(i.signal),
            direction: non_empty(i.direction),
            canary: non_empty(i.canary),
        })
        .collect();
    (issues, faults)
}

// ============================================================================
// Public Functions
// ============================================================================

pub fn scan_directory(directory: &str, include_tests: bool) -> Result<ScanResult, String> {
    let sidecars = find_sidecars(directory, include_tests)?;

    let mut all_issues: Vec<Issue> = vec![];
    let mut all_faults: Vec<Fault> = vec![];
    let files_scanned = sidecars.len();

    for sidecar in sidecars {
        let (issues, faults) = read_sidecar(&sidecar);
        all_issues.extend(issues);
        all_faults.extend(faults);
    }

    let mut by_tool: HashMap<String, usize> = HashMap::new();
    let mut by_severity: HashMap<String, usize> = HashMap::new();
    let mut by_file: HashMap<String, usize> = HashMap::new();
    let mut by_code: HashMap<String, usize> = HashMap::new();

    for issue in &all_issues {
        *by_tool.entry(issue.tool.clone()).or_insert(0) += 1;
        *by_severity.entry(issue.severity.clone()).or_insert(0) += 1;
        *by_file.entry(issue.file.clone()).or_insert(0) += 1;
        *by_code.entry(issue.code.clone()).or_insert(0) += 1;
    }

    Ok(ScanResult {
        directory: directory.to_string(),
        files_scanned,
        issues: all_issues,
        faults: all_faults,
        by_tool,
        by_severity,
        by_file,
        by_code,
    })
}

pub use common_core::open_in_editor;

pub fn run_saga(directory: &str) -> Result<SagaResult, String> {
    let saga_path = dirs::home_dir()
        .map(|h| h.join("ai/tools/bin/saga"))
        .ok_or("Could not determine home directory")?;

    if !saga_path.exists() {
        return Err(format!("Saga not found at {}", saga_path.display()));
    }

    let home = dirs::home_dir().map(|h| h.display().to_string()).unwrap_or_default();
    let path = std::env::var("PATH").unwrap_or_default();
    let extended_path = format!(
        "{}/.local/bin:/opt/homebrew/bin:/usr/local/bin:{}",
        home, path
    );

    let output = std::process::Command::new(&saga_path)
        .arg("--force")
        .arg(directory)
        .env("PATH", &extended_path)
        .output()
        .map_err(|e| format!("Failed to run saga: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Saga writes its summary to stderr.
    let (files_analyzed, not_fully_examined) = parse_saga_output(&stderr);

    let combined = if stderr.is_empty() {
        stdout
    } else {
        format!("{}\n{}", stdout, stderr)
    };

    Ok(SagaResult {
        success: output.status.success(),
        files_analyzed,
        not_fully_examined,
        output: combined,
    })
}

/// The count saga's summary line gives before a label, e.g. the 12 in "12 generated".
fn summary_count(summary: &str, label: &str) -> usize {
    summary
        .split(", ")
        .find_map(|part| part.trim().strip_suffix(label))
        .and_then(|count| count.trim().rsplit(' ').next())
        .and_then(|count| count.parse().ok())
        .unwrap_or(0)
}

/// From saga's summary line — "[saga] N generated, N skipped, N failed, N not fully
/// examined (of N ... files)" — the files it wrote sidecars for, and those it could
/// not fully examine.
fn parse_saga_output(output: &str) -> (usize, usize) {
    let Some(summary) = output.lines().find(|line| line.starts_with("[saga] ") && line.contains(" generated")) else {
        return (0, 0);
    };
    let before_total = summary.split(" (of ").next().unwrap_or(summary);
    (summary_count(before_total, " generated"), summary_count(before_total, " not fully examined"))
}

/// What a sidecar gives the tree to count: its issues and its faults, so a file
/// that was not examined never shows as having nothing.
fn findings_count(sidecar: &Path) -> usize {
    let (issues, faults) = read_sidecar(sidecar);
    issues.len() + faults.len()
}

/// Whether a file has a sidecar, and what the tree counts for the entry: a file's
/// own findings, or those of every sidecar under a directory.
fn sidecar_status(path: &Path, is_dir: bool) -> Result<(bool, usize), String> {
    if is_dir {
        let sidecars = find_sidecars(&path.to_string_lossy(), true)?;
        return Ok((false, sidecars.iter().map(|sidecar| findings_count(sidecar)).sum()));
    }
    let sidecar_path = qa_path(path);
    if sidecar_path.exists() {
        Ok((true, findings_count(&sidecar_path)))
    } else {
        Ok((false, 0))
    }
}

/// A directory's own entries, each with what its sidecars hold. The listing is
/// `common_core::list_directory`'s, already directories first.
pub fn list_qa_tree(directory: &str) -> Result<Vec<SvalFileTreeEntry>, String> {
    let mut entries = Vec::new();
    for listed in common_core::list_directory(directory, false)? {
        let (has_sidecar, issue_count) = sidecar_status(Path::new(&listed.path), listed.is_dir)?;
        entries.push(SvalFileTreeEntry {
            name: listed.name,
            path: listed.path,
            is_dir: listed.is_dir,
            has_sidecar,
            issue_count,
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saga_s_summary_gives_the_files_written_and_those_not_fully_examined() {
        let stderr = "  src/.a.py.qa\n  FAULT /p/src/b.py: ruff: why\n\
                      [saga] 12 generated, 3 skipped, 1 failed, 2 not fully examined (of 16 .py/.rs/.svelte files)\n";
        assert_eq!(parse_saga_output(stderr), (12, 2));
    }

    #[test]
    fn output_without_a_summary_gives_no_counts() {
        assert_eq!(parse_saga_output("[saga] no .py files found in /p\n"), (0, 0));
    }

    #[test]
    fn a_sidecar_that_cannot_be_used_is_a_fault_naming_it() {
        let dir = std::env::temp_dir().join(format!("svalinn_core_unusable_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sidecar = dir.join(".a.py.qa");
        std::fs::write(&sidecar, r#"{"file":"/p/a.py","issues":[]}"#).unwrap();

        let (issues, faults) = read_sidecar(&sidecar);
        assert!(issues.is_empty());
        assert_eq!(faults.len(), 1);
        assert_eq!(faults[0].tool, "svalinn");
        assert_eq!(faults[0].file, sidecar.display().to_string());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
