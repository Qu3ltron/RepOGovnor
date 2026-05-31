use crate::model::Result;
use crate::reports::{RuntimeFailure, RuntimeResult};
use crate::schema::{CheckReport, Diagnostic, ReleaseCheckId, VersionFileFormat};
use serde::Deserialize;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct Requirements {
    schema_version: i64,
    plugin_name: String,
    release_source: ReleaseSource,
    #[serde(default)]
    tracked_for_ci: Option<TrackedForCi>,
    #[serde(default)]
    post_install: Option<toml::Value>,
    #[serde(default)]
    source_file_governance: Option<toml::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct ReleaseSource {
    version: String,
    #[serde(default)]
    status_command: Option<String>,
    #[serde(default)]
    version_check_command: Option<String>,
    #[serde(default)]
    audit_command: Option<String>,
    required: Vec<String>,
    #[serde(default)]
    executable: Vec<String>,
    #[serde(default)]
    stale_absent: Vec<String>,
    #[serde(default)]
    check_ids: Vec<String>,
    #[serde(default)]
    version_files: Vec<VersionFile>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
struct TrackedForCi {
    #[serde(default)]
    design_rule: Option<String>,
    required: Vec<String>,
    #[serde(default)]
    plugin_link: Option<toml::Value>,
    #[serde(default)]
    overlay_markers: Option<toml::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionFile {
    path: String,
    format: VersionFileFormat,
    #[serde(default)]
    key: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Required,
    Version,
    Tracked,
    All,
}

pub(crate) fn run_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let Some(mode) = args.first().map(String::as_str) else {
        return Err(release_usage().into());
    };
    let mode = match mode {
        "required" => Mode::Required,
        "version" => Mode::Version,
        "tracked" => Mode::Tracked,
        "all" => Mode::All,
        _ => return Err(release_usage().into()),
    };
    let mut json = false;
    let mut index = 1usize;
    while index < args.len() {
        match args[index].as_str() {
            "--format" => {
                index += 1;
                if args.get(index).map(String::as_str) != Some("json") {
                    return Err(release_usage().into());
                }
                json = true;
            }
            _ => return Err(release_usage().into()),
        }
        index += 1;
    }

    let report = report(root, mode)?;
    if json {
        let output = report.to_json()?;
        if report.has_failures() {
            return Err(RuntimeFailure::json(output));
        }
        Ok(output)
    } else {
        let output = format_human(&report);
        if report.has_failures() {
            return Err(output.into());
        }
        Ok(output)
    }
}

pub(crate) fn report(root: &Path, mode: Mode) -> Result<CheckReport> {
    let requirements = load_requirements(root)?;
    let mut checks = Vec::new();
    if matches!(mode, Mode::Required | Mode::All) {
        checks.extend(required_checks(root, &requirements));
        checks.extend(executable_checks(root, &requirements));
        checks.extend(stale_absent_checks(root, &requirements));
        checks.extend(schema_checks(&requirements));
    }
    if matches!(mode, Mode::Version | Mode::All) {
        checks.extend(version_checks(root, &requirements));
    }
    if matches!(mode, Mode::Tracked) {
        checks.extend(tracked_checks(root, &requirements));
    }
    CheckReport::new("release-source", checks)
}

fn load_requirements(root: &Path) -> Result<Requirements> {
    let path = root.join("REQUIREMENTS.toml");
    let body =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    toml::from_str(&body).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn required_checks(root: &Path, requirements: &Requirements) -> Vec<Diagnostic> {
    requirements
        .release_source
        .required
        .iter()
        .map(|path| {
            if root.join(path).is_file() {
                Diagnostic::pass(
                    ReleaseCheckId::ReleaseFilePresent.as_str(),
                    "release-source",
                    path,
                    "file present",
                )
            } else {
                Diagnostic::fail(
                    ReleaseCheckId::ReleaseFilePresent.as_str(),
                    "release-source",
                    path,
                    "file present",
                    "missing",
                    "restore the required release artifact or update REQUIREMENTS.toml through an approved plan",
                )
            }
        })
        .collect()
}

fn executable_checks(root: &Path, requirements: &Requirements) -> Vec<Diagnostic> {
    requirements
        .release_source
        .executable
        .iter()
        .map(|path| {
            let full_path = root.join(path);
            match fs::metadata(&full_path) {
                Ok(metadata) if metadata.is_file() && metadata_is_executable(&metadata) => {
                    Diagnostic::pass(
                        ReleaseCheckId::ReleaseFileExecutable.as_str(),
                        "release-source",
                        path,
                        "executable file",
                    )
                }
                Ok(metadata) if metadata.is_file() => Diagnostic::fail(
                    ReleaseCheckId::ReleaseFileExecutable.as_str(),
                    "release-source",
                    path,
                    "executable file",
                    "not executable",
                    "chmod +x the release script or remove it from release_source.executable through an approved plan",
                ),
                Ok(_) => Diagnostic::fail(
                    ReleaseCheckId::ReleaseFileExecutable.as_str(),
                    "release-source",
                    path,
                    "executable file",
                    "not a file",
                    "replace the path with an executable file or update REQUIREMENTS.toml through an approved plan",
                ),
                Err(_) => Diagnostic::fail(
                    ReleaseCheckId::ReleaseFileExecutable.as_str(),
                    "release-source",
                    path,
                    "executable file",
                    "missing",
                    "restore the executable release artifact or update REQUIREMENTS.toml through an approved plan",
                ),
            }
        })
        .collect()
}

#[cfg(unix)]
fn metadata_is_executable(metadata: &fs::Metadata) -> bool {
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn metadata_is_executable(metadata: &fs::Metadata) -> bool {
    metadata.is_file()
}

fn stale_absent_checks(root: &Path, requirements: &Requirements) -> Vec<Diagnostic> {
    requirements
        .release_source
        .stale_absent
        .iter()
        .map(|path| {
            if !root.join(path).exists() {
                Diagnostic::pass(
                    ReleaseCheckId::StalePathAbsent.as_str(),
                    "release-source",
                    path,
                    "path absent",
                )
            } else {
                Diagnostic::fail(
                    ReleaseCheckId::StalePathAbsent.as_str(),
                    "release-source",
                    path,
                    "path absent",
                    "present",
                    "remove stale legacy release-incompatible artifact",
                )
            }
        })
        .collect()
}

fn schema_checks(requirements: &Requirements) -> Vec<Diagnostic> {
    let mut checks = Vec::new();
    if requirements.schema_version == 1 {
        checks.push(Diagnostic::pass(
            ReleaseCheckId::ReleaseSchemaValid.as_str(),
            "release-source",
            "schema_version",
            "schema_version 1",
        ));
    } else {
        checks.push(Diagnostic::fail(
            ReleaseCheckId::ReleaseSchemaValid.as_str(),
            "release-source",
            "schema_version",
            "schema_version 1",
            format!("schema_version {}", requirements.schema_version),
            "update REQUIREMENTS.toml to the supported schema version",
        ));
    }
    if requirements.plugin_name.trim().is_empty() {
        checks.push(Diagnostic::fail(
            ReleaseCheckId::ReleaseSchemaValid.as_str(),
            "release-source",
            "plugin_name",
            "non-empty plugin name",
            "empty",
            "set plugin_name in REQUIREMENTS.toml",
        ));
    }
    for check_id in &requirements.release_source.check_ids {
        if ReleaseCheckId::from_str(check_id).is_ok() {
            checks.push(Diagnostic::pass(
                ReleaseCheckId::ReleaseSchemaValid.as_str(),
                "release-source",
                check_id,
                "known release check id",
            ));
        } else {
            checks.push(Diagnostic::fail(
                ReleaseCheckId::ReleaseSchemaValid.as_str(),
                "release-source",
                check_id,
                "known release check id",
                "unknown",
                "replace with a supported release check id",
            ));
        }
    }
    if requirements.release_source.version_files.is_empty() {
        checks.push(Diagnostic::fail(
            ReleaseCheckId::ReleaseSchemaValid.as_str(),
            "release-source",
            "release_source.version_files",
            "at least one version-bearing file",
            "missing",
            "declare version-bearing files in REQUIREMENTS.toml",
        ));
    }
    checks
}

fn version_checks(root: &Path, requirements: &Requirements) -> Vec<Diagnostic> {
    let expected = requirements.release_source.version.trim();
    requirements
        .release_source
        .version_files
        .iter()
        .map(|version_file| match extract_version(root, version_file) {
            Ok(actual) if actual == expected => Diagnostic::pass(
                ReleaseCheckId::ReleaseVersionConsistent.as_str(),
                "release-source",
                &version_file.path,
                format!("version {expected}"),
            ),
            Ok(actual) => Diagnostic::fail(
                ReleaseCheckId::ReleaseVersionConsistent.as_str(),
                "release-source",
                &version_file.path,
                format!("version {expected}"),
                format!("version {actual}"),
                "update the version-bearing file or REQUIREMENTS.toml release_source.version",
            ),
            Err(error) => Diagnostic::fail(
                ReleaseCheckId::ReleaseVersionConsistent.as_str(),
                "release-source",
                &version_file.path,
                format!("version {expected}"),
                error,
                "restore the version-bearing file or repair its schema selector",
            ),
        })
        .collect()
}

fn tracked_checks(root: &Path, requirements: &Requirements) -> Vec<Diagnostic> {
    requirements
        .tracked_for_ci
        .as_ref()
        .map(|tracked| tracked.required.as_slice())
        .unwrap_or_default()
        .iter()
        .map(|path| {
            if root.join(path).exists() {
                Diagnostic::pass(
                    ReleaseCheckId::TrackedForCiPresent.as_str(),
                    "tracked-for-ci",
                    path,
                    "path present",
                )
            } else {
                Diagnostic::fail(
                    ReleaseCheckId::TrackedForCiPresent.as_str(),
                    "tracked-for-ci",
                    path,
                    "path present",
                    "missing",
                    "run install --merge/--force and commit the required governance artifact",
                )
            }
        })
        .collect()
}

fn extract_version(root: &Path, version_file: &VersionFile) -> Result<String> {
    let path = root.join(&version_file.path);
    let body =
        fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
    match version_file.format {
        VersionFileFormat::Plain => Ok(body.trim().to_string()),
        VersionFileFormat::Json => {
            let value = serde_json::from_str::<serde_json::Value>(&body)
                .map_err(|error| format!("parse JSON: {error}"))?;
            let key = version_file
                .key
                .as_deref()
                .ok_or_else(|| "json version file requires key".to_string())?;
            json_key(&value, key)
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| format!("missing JSON string key {key}"))
        }
        VersionFileFormat::Toml => {
            let value = toml::from_str::<toml::Value>(&body)
                .map_err(|error| format!("parse TOML: {error}"))?;
            let key = version_file
                .key
                .as_deref()
                .ok_or_else(|| "toml version file requires key".to_string())?;
            toml_key(&value, key)
                .and_then(toml::Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| format!("missing TOML string key {key}"))
        }
    }
}

fn json_key<'a>(value: &'a serde_json::Value, key: &str) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for part in key.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

fn toml_key<'a>(value: &'a toml::Value, key: &str) -> Option<&'a toml::Value> {
    let mut current = value;
    for part in key.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

fn format_human(report: &CheckReport) -> String {
    let mut lines = Vec::new();
    for check in &report.checks {
        lines.push(format!(
            "{} {} {}: expected {}, actual {}",
            check.status, check.check_id, check.path, check.expected, check.actual
        ));
    }
    lines.push(format!(
        "release check summary: {} pass, {} warn, {} fail, {} skip",
        report.summary.pass, report.summary.warn, report.summary.fail, report.summary.skip
    ));
    lines.join("\n")
}

fn release_usage() -> String {
    "usage: task-registry-flow release-check {required|version|tracked|all} [--format json]"
        .to_string()
}
