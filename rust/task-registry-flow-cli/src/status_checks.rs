use crate::model::Result;
use crate::reports::{RuntimeFailure, RuntimeResult};
use crate::schema::{CheckReport, Diagnostic};
use std::fs;
use std::path::Path;

const MARKER_BEGIN: &str = "<!-- agent-governance:begin -->";
const MARKER_END: &str = "<!-- agent-governance:end -->";
const MARKER_HEADING: &str = "## Agent governance (portable plugin)";
const MARKER_SOURCE_LIMIT: &str = "Source limit";

pub(crate) fn run_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let json = match args {
        [] => false,
        [flag, format] if flag == "--format" && format == "json" => true,
        _ => return Err("usage: task-registry-flow status-check [--format json]".into()),
    };
    let skill_path = ".agents/skills/task-registry-flow";
    let path = root.join(skill_path);
    let report = report(
        "status",
        vec![
            marker_check(root, "AGENTS.md"),
            marker_check(root, "GEMINI.md"),
            native_skill_check(skill_path, path.is_dir() && !path.is_symlink()),
        ],
    )?;
    if json {
        let output = report.to_json()?;
        if report.has_failures() {
            Err(RuntimeFailure::json(output))
        } else {
            Ok(output)
        }
    } else if report.has_failures() {
        Err(report.to_json()?.into())
    } else {
        Ok(format!(
            "status-check ok: {} pass, {} fail",
            report.summary.pass, report.summary.fail
        ))
    }
}

pub(crate) fn report(surface: &str, checks: Vec<Diagnostic>) -> Result<CheckReport> {
    CheckReport::new(surface, checks)
}

pub(crate) fn marker_check(root: &Path, path: &str) -> Diagnostic {
    let full_path = root.join(path);
    let Ok(body) = fs::read_to_string(&full_path) else {
        return Diagnostic::fail(
            "governance-marker",
            "status",
            path,
            "single governance marker block",
            "missing file",
            "install or refresh the governance projection",
        );
    };
    let begin_lines = marker_lines(&body, MARKER_BEGIN);
    let end_lines = marker_lines(&body, MARKER_END);
    if begin_lines.len() == 1 && end_lines.len() == 1 && begin_lines[0] < end_lines[0] {
        let block = marker_block(&body, begin_lines[0], end_lines[0]);
        if !marker_content_valid(path, &block) {
            return Diagnostic::fail(
                "governance-marker",
                "status",
                path,
                "current managed marker block",
                "stale marker content",
                "run install-to-workspace --merge or --force to refresh marker provenance",
            );
        }
        Diagnostic::pass(
            "governance-marker",
            "status",
            path,
            "single governance marker block",
        )
    } else {
        let actual = if begin_lines.is_empty() && end_lines.is_empty() {
            "missing marker block".to_string()
        } else if begin_lines.len() == 1 && end_lines.len() == 1 {
            "marker block out of order".to_string()
        } else {
            format!("begin={} end={}", begin_lines.len(), end_lines.len())
        };
        Diagnostic::fail(
            "governance-marker",
            "status",
            path,
            "single governance marker block",
            actual,
            "run install-to-workspace --merge or --force to restore marker provenance",
        )
    }
}

fn marker_lines(body: &str, marker: &str) -> Vec<usize> {
    body.lines()
        .enumerate()
        .filter_map(|(line_number, line)| {
            (line.trim_end_matches('\r') == marker).then_some(line_number)
        })
        .collect()
}

fn marker_block(body: &str, begin_line: usize, end_line: usize) -> String {
    body.lines()
        .skip(begin_line + 1)
        .take(end_line.saturating_sub(begin_line + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn marker_content_valid(path: &str, block: &str) -> bool {
    block.contains(MARKER_HEADING)
        && block.contains(MARKER_SOURCE_LIMIT)
        && match path {
            "AGENTS.md" => block.contains("| Registry CLI |"),
            "GEMINI.md" => block.contains("Antigravity hook:"),
            _ => true,
        }
}

pub(crate) fn native_skill_check(path: &str, is_native_directory: bool) -> Diagnostic {
    if is_native_directory {
        Diagnostic::pass("native-skill", "status", path, "native directory")
    } else {
        Diagnostic::fail(
            "native-skill",
            "status",
            path,
            "native directory",
            "missing or symlink",
            "replace legacy skill symlink with native directory",
        )
    }
}
