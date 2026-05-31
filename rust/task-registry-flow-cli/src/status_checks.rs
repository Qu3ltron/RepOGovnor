use crate::model::Result;
use crate::reports::{RuntimeFailure, RuntimeResult};
use crate::schema::{CheckReport, Diagnostic};
use std::path::Path;

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
        vec![native_skill_check(
            skill_path,
            path.is_dir() && !path.is_symlink(),
        )],
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
