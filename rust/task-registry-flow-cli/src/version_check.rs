use crate::model::{REGISTRY_PATH, Result, TaskRegistry, TaskRegistryArchive};
use crate::reports::{RuntimeFailure, RuntimeResult};
use crate::schema::{CheckReport, Diagnostic, FailureCode, ReportSurface, TaskStatus};
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

const VERSION_PATH: &str = "VERSION";
const ROADMAP_PATH: &str = "docs/version-roadmap.toml";
const CHANGELOG_PATH: &str = "CHANGELOG.md";
const FINAL_MANUAL_POLICY: &str = "auto_prerelease_manual_release";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Roadmap {
    schema_version: i64,
    version_model: String,
    current_version: String,
    previous_version: String,
    release_branch: String,
    remote: String,
    tag_prefix: String,
    push_policy: String,
    cutover_plan_id: String,
    #[serde(default)]
    releases: Vec<Release>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Release {
    version: String,
    date: String,
    plan_id: String,
    bump: String,
    tag: String,
    prerelease_tag: String,
    commit_subject: String,
    summary: String,
    #[serde(default)]
    covered_plan_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
}

pub(crate) fn run_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err(usage().into());
    };
    match subcommand {
        "validate" => run_validate(root, &args[1..]),
        "next" => run_next(root, &args[1..]),
        "prerelease" => run_prerelease(root, &args[1..]),
        "release-check" => run_release_check(root, &args[1..]),
        _ => Err(usage().into()),
    }
}

fn run_validate(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let json = parse_optional_json(args)?;
    let report = report(root)?;
    if json {
        let output = report.to_json()?;
        if report.has_failures() {
            Err(RuntimeFailure::json(FailureCode::DiagnosticReport, output))
        } else {
            Ok(output)
        }
    } else if report.has_failures() {
        Err(format_human(&report).into())
    } else {
        Ok("version-check validate ok".to_string())
    }
}

fn run_next(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let (plan_id, json) = parse_plan_and_json(args)?;
    let roadmap = load_roadmap(root)?;
    let release = release_for_plan(&roadmap, plan_id)?;
    if json {
        Ok(json!({
            "schema_version": 1,
            "surface": "version",
            "version": release.version,
            "tag": release.tag,
            "prerelease_tag": release.prerelease_tag,
            "commit_subject": release.commit_subject,
            "manual_final_release": true,
        })
        .to_string())
    } else {
        Ok(format_next(release))
    }
}

fn run_prerelease(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let (plan_id, rc, json) = parse_prerelease_args(args)?;
    let roadmap = load_roadmap(root)?;
    validate_report_has_no_failures(report(root)?)?;
    let release = release_for_plan(&roadmap, plan_id)?;
    let expected = format!("{}-rc.{rc}", release.tag);
    if release.prerelease_tag != expected {
        return Err(format!(
            "release {} prerelease_tag must be {expected}, got {}",
            release.version, release.prerelease_tag
        )
        .into());
    }
    if json {
        Ok(json!({
            "schema_version": 1,
            "surface": "version",
            "version": release.version,
            "prerelease_tag": release.prerelease_tag,
            "commit_subject": release.commit_subject,
            "push_branch": format!("git push {} {}", roadmap.remote, roadmap.release_branch),
            "push_prerelease_tag": format!("git push {} {}", roadmap.remote, release.prerelease_tag),
            "final_release_manual": true,
        })
        .to_string())
    } else {
        Ok(format_prerelease(&roadmap, release))
    }
}

fn run_release_check(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let json = parse_optional_json(args)?;
    let mut report = report(root)?;
    report.checks.extend(final_tag_checks(root)?);
    report = CheckReport::new(ReportSurface::Version, report.checks)?;
    if json {
        let output = report.to_json()?;
        if report.has_failures() {
            Err(RuntimeFailure::json(FailureCode::DiagnosticReport, output))
        } else {
            Ok(output)
        }
    } else if report.has_failures() {
        Err(format_human(&report).into())
    } else {
        Ok("version-check release-check ok".to_string())
    }
}

pub(crate) fn report(root: &Path) -> Result<CheckReport> {
    let roadmap = load_roadmap(root)?;
    let version = read_trimmed(root, VERSION_PATH)?;
    let mut checks = Vec::new();
    checks.extend(version_surface_checks(root, &version));
    checks.extend(roadmap_checks(root, &roadmap, &version)?);
    CheckReport::new(ReportSurface::Version, checks)
}

fn version_surface_checks(root: &Path, expected: &str) -> Vec<Diagnostic> {
    version_files()
        .into_iter()
        .map(
            |file| match extract_version(root, file.path, file.format, file.key) {
                Ok(actual) if actual == expected => Diagnostic::pass(
                    "version-surface",
                    ReportSurface::Version,
                    file.path,
                    format!("version {expected}"),
                ),
                Ok(actual) => Diagnostic::fail(
                    "version-surface",
                    ReportSurface::Version,
                    file.path,
                    format!("version {expected}"),
                    format!("version {actual}"),
                    "align the version surface with VERSION and docs/version-roadmap.toml",
                ),
                Err(error) => Diagnostic::fail(
                    "version-surface",
                    ReportSurface::Version,
                    file.path,
                    format!("version {expected}"),
                    error,
                    "restore or repair the version-bearing file",
                ),
            },
        )
        .collect()
}

fn roadmap_checks(root: &Path, roadmap: &Roadmap, version: &str) -> Result<Vec<Diagnostic>> {
    let mut checks = Vec::new();
    checks.push(equal_check(
        "version-roadmap",
        ROADMAP_PATH,
        "schema_version 1",
        format!("schema_version {}", roadmap.schema_version),
        roadmap.schema_version == 1,
    ));
    checks.push(equal_check(
        "version-roadmap",
        ROADMAP_PATH,
        "governed_semver",
        &roadmap.version_model,
        roadmap.version_model == "governed_semver",
    ));
    checks.push(equal_check(
        "version-roadmap",
        ROADMAP_PATH,
        version,
        &roadmap.current_version,
        roadmap.current_version == version,
    ));
    checks.push(equal_check(
        "version-policy",
        ROADMAP_PATH,
        FINAL_MANUAL_POLICY,
        &roadmap.push_policy,
        roadmap.push_policy == FINAL_MANUAL_POLICY,
    ));
    let Some(release) = roadmap
        .releases
        .iter()
        .find(|release| release.version == version)
    else {
        checks.push(Diagnostic::fail(
            "version-roadmap-release",
            ReportSurface::Version,
            ROADMAP_PATH,
            format!("release {version}"),
            "missing",
            "add a release row covering the current version",
        ));
        return Ok(checks);
    };
    checks.extend(release_checks(roadmap, release));
    checks.extend(registry_coverage_checks(root, roadmap)?);
    Ok(checks)
}

fn release_checks(roadmap: &Roadmap, release: &Release) -> Vec<Diagnostic> {
    let mut checks = Vec::new();
    let expected_tag = format!("{}{}", roadmap.tag_prefix, release.version);
    checks.push(equal_check(
        "version-release-tag",
        ROADMAP_PATH,
        &expected_tag,
        &release.tag,
        release.tag == expected_tag,
    ));
    let expected_prerelease = format!("{}-rc.1", release.tag);
    checks.push(equal_check(
        "version-prerelease-tag",
        ROADMAP_PATH,
        &expected_prerelease,
        &release.prerelease_tag,
        release.prerelease_tag == expected_prerelease,
    ));
    let previous = parse_semver(&roadmap.previous_version);
    let current = parse_semver(&release.version);
    let legal = previous
        .zip(current)
        .is_some_and(|(previous, current)| legal_bump(previous, current, &release.bump));
    checks.push(equal_check(
        "version-semver-bump",
        ROADMAP_PATH,
        "legal governed semver bump",
        format!(
            "{} from {} to {}",
            release.bump, roadmap.previous_version, release.version
        ),
        legal,
    ));
    for (field, value) in [
        ("date", &release.date),
        ("plan_id", &release.plan_id),
        ("commit_subject", &release.commit_subject),
        ("summary", &release.summary),
    ] {
        checks.push(equal_check(
            "version-release-field",
            ROADMAP_PATH,
            format!("non-empty {field}"),
            if value.trim().is_empty() {
                "empty"
            } else {
                "set"
            },
            !value.trim().is_empty(),
        ));
    }
    checks
}

fn registry_coverage_checks(root: &Path, roadmap: &Roadmap) -> Result<Vec<Diagnostic>> {
    let registry = load_registry_with_archives(root)?;
    let all_plan_ids = registry
        .plans
        .iter()
        .map(|plan| plan.plan_id.as_str())
        .collect::<BTreeSet<_>>();
    let covered = roadmap
        .releases
        .iter()
        .flat_map(|release| release.covered_plan_ids.iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let mut checks = Vec::new();
    checks.push(equal_check(
        "version-cutover-plan",
        REGISTRY_PATH,
        format!("plan {}", roadmap.cutover_plan_id),
        if all_plan_ids.contains(roadmap.cutover_plan_id.as_str()) {
            "present"
        } else {
            "missing"
        },
        all_plan_ids.contains(roadmap.cutover_plan_id.as_str()),
    ));
    for release in &roadmap.releases {
        checks.push(equal_check(
            "version-release-plan",
            ROADMAP_PATH,
            format!("plan {}", release.plan_id),
            if all_plan_ids.contains(release.plan_id.as_str()) {
                "present"
            } else {
                "missing"
            },
            all_plan_ids.contains(release.plan_id.as_str()),
        ));
        for plan_id in &release.covered_plan_ids {
            checks.push(equal_check(
                "version-covered-plan-exists",
                ROADMAP_PATH,
                format!("plan {plan_id}"),
                if all_plan_ids.contains(plan_id.as_str()) {
                    "present"
                } else {
                    "missing"
                },
                all_plan_ids.contains(plan_id.as_str()),
            ));
        }
    }
    let completed = completed_post_cutover_plan_ids(&registry, &roadmap.cutover_plan_id);
    for plan_id in completed {
        checks.push(equal_check(
            "version-completed-plan-covered",
            ROADMAP_PATH,
            format!("release coverage for {plan_id}"),
            if covered.contains(plan_id.as_str()) {
                "covered"
            } else {
                "missing"
            },
            covered.contains(plan_id.as_str()),
        ));
    }
    Ok(checks)
}

fn final_tag_checks(root: &Path) -> Result<Vec<Diagnostic>> {
    let roadmap = load_roadmap(root)?;
    let version = read_trimmed(root, VERSION_PATH)?;
    let release = roadmap
        .releases
        .iter()
        .find(|release| release.version == version)
        .ok_or_else(|| format!("{ROADMAP_PATH} missing release for {version}"))?;
    let branch = git(root, &["branch", "--show-current"]).unwrap_or_else(|error| error);
    let mut checks = vec![equal_check(
        "version-release-branch",
        ".",
        &roadmap.release_branch,
        &branch,
        branch == roadmap.release_branch,
    )];
    let head = git(root, &["rev-parse", "HEAD"]);
    let tag = git(root, &["rev-parse", &format!("refs/tags/{}", release.tag)]);
    let tag_at_head = matches!((&head, &tag), (Ok(head), Ok(tag)) if head == tag);
    checks.push(equal_check(
        "version-final-tag-head",
        ".",
        format!("{} at HEAD", release.tag),
        match (&head, &tag) {
            (Ok(head), Ok(tag)) if head == tag => "tag at HEAD".to_string(),
            (Ok(head), Ok(tag)) => format!("HEAD {head}, tag {tag}"),
            (_, Err(error)) => error.clone(),
            (Err(error), _) => error.clone(),
        },
        tag_at_head,
    ));
    Ok(checks)
}

fn completed_post_cutover_plan_ids(registry: &TaskRegistry, cutover_plan_id: &str) -> Vec<String> {
    let task_statuses = registry.tasks.iter().fold(
        BTreeMap::<String, Vec<TaskStatus>>::new(),
        |mut statuses, task| {
            statuses
                .entry(task.plan_id.clone())
                .or_default()
                .push(task.status);
            statuses
        },
    );
    let end_index = registry
        .plans
        .iter()
        .position(|plan| plan.plan_id == cutover_plan_id)
        .unwrap_or_else(|| registry.plans.len().saturating_sub(1));
    registry
        .plans
        .iter()
        .take(end_index + 1)
        .filter_map(|plan| {
            let statuses = task_statuses.get(&plan.plan_id)?;
            let complete = !statuses.is_empty()
                && statuses
                    .iter()
                    .all(|status| matches!(status, TaskStatus::Completed | TaskStatus::Cancelled));
            complete.then(|| plan.plan_id.clone())
        })
        .collect()
}

fn load_registry_with_archives(root: &Path) -> Result<TaskRegistry> {
    let body = fs::read_to_string(root.join(REGISTRY_PATH))
        .map_err(|error| format!("read {REGISTRY_PATH}: {error}"))?;
    let mut registry = toml::from_str::<TaskRegistry>(&body)
        .map_err(|error| format!("parse registry: {error}"))?;
    let archive_paths = registry.archive_paths.clone();
    let mut plans = registry.plans.clone();
    let mut tasks = registry.tasks.clone();
    for archive_path in archive_paths {
        let full_path = root.join(&archive_path);
        let body = fs::read_to_string(&full_path)
            .map_err(|error| format!("read {}: {error}", full_path.display()))?;
        let archive = toml::from_str::<TaskRegistryArchive>(&body)
            .map_err(|error| format!("parse {}: {error}", full_path.display()))?;
        plans.extend(archive.plans);
        tasks.extend(archive.tasks);
    }
    registry.plans = plans;
    registry.tasks = tasks;
    Ok(registry)
}

fn load_roadmap(root: &Path) -> Result<Roadmap> {
    let body = fs::read_to_string(root.join(ROADMAP_PATH))
        .map_err(|error| format!("read {ROADMAP_PATH}: {error}"))?;
    toml::from_str(&body).map_err(|error| format!("parse {ROADMAP_PATH}: {error}"))
}

fn read_trimmed(root: &Path, path: &str) -> Result<String> {
    let value =
        fs::read_to_string(root.join(path)).map_err(|error| format!("read {path}: {error}"))?;
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(format!("{path} must not be empty"))
    } else {
        Ok(value)
    }
}

struct VersionFile {
    path: &'static str,
    format: VersionFormat,
    key: Option<&'static str>,
}

enum VersionFormat {
    Plain,
    Json,
    Toml,
    MarkdownLine,
    NixVersion,
    CargoLockPackage,
}

fn version_files() -> Vec<VersionFile> {
    vec![
        VersionFile {
            path: VERSION_PATH,
            format: VersionFormat::Plain,
            key: None,
        },
        VersionFile {
            path: "plugin.json",
            format: VersionFormat::Json,
            key: Some("version"),
        },
        VersionFile {
            path: ".codex-plugin/plugin.json",
            format: VersionFormat::Json,
            key: Some("version"),
        },
        VersionFile {
            path: "MANIFEST.toml",
            format: VersionFormat::Toml,
            key: Some("plugin_version"),
        },
        VersionFile {
            path: "rust/task-registry-flow-cli/Cargo.toml",
            format: VersionFormat::Toml,
            key: Some("package.version"),
        },
        VersionFile {
            path: "rust/task-registry-flow-cli/Cargo.lock",
            format: VersionFormat::CargoLockPackage,
            key: Some("task-registry-flow-cli"),
        },
        VersionFile {
            path: "package.nix",
            format: VersionFormat::NixVersion,
            key: None,
        },
        VersionFile {
            path: "README.md",
            format: VersionFormat::MarkdownLine,
            key: Some("Current release:"),
        },
        VersionFile {
            path: "docs/releases/v2.md",
            format: VersionFormat::MarkdownLine,
            key: Some("Release version:"),
        },
        VersionFile {
            path: "REQUIREMENTS.toml",
            format: VersionFormat::Toml,
            key: Some("release_source.version"),
        },
        VersionFile {
            path: CHANGELOG_PATH,
            format: VersionFormat::MarkdownLine,
            key: Some("##"),
        },
    ]
}

fn extract_version(
    root: &Path,
    path: &str,
    format: VersionFormat,
    key: Option<&str>,
) -> Result<String> {
    let body =
        fs::read_to_string(root.join(path)).map_err(|error| format!("read {path}: {error}"))?;
    match format {
        VersionFormat::Plain => Ok(body.trim().to_string()),
        VersionFormat::Json => {
            let value = serde_json::from_str::<serde_json::Value>(&body)
                .map_err(|error| format!("parse JSON: {error}"))?;
            let key = key.ok_or_else(|| "json version key missing".to_string())?;
            json_key(&value, key)
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| format!("missing JSON string key {key}"))
        }
        VersionFormat::Toml => {
            let value = toml::from_str::<toml::Value>(&body)
                .map_err(|error| format!("parse TOML: {error}"))?;
            let key = key.ok_or_else(|| "toml version key missing".to_string())?;
            toml_key(&value, key)
                .and_then(toml::Value::as_str)
                .map(str::to_string)
                .ok_or_else(|| format!("missing TOML string key {key}"))
        }
        VersionFormat::MarkdownLine => {
            let key = key.ok_or_else(|| "markdown version key missing".to_string())?;
            markdown_line_version(&body, key)
                .ok_or_else(|| format!("missing Markdown version line with prefix {key}"))
        }
        VersionFormat::NixVersion => {
            nix_version(&body).ok_or_else(|| "missing Nix version".to_string())
        }
        VersionFormat::CargoLockPackage => {
            let package = key.ok_or_else(|| "lock package name missing".to_string())?;
            cargo_lock_package_version(&body, package)
                .ok_or_else(|| format!("missing Cargo.lock package {package} version"))
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

fn markdown_line_version(body: &str, key: &str) -> Option<String> {
    if key == "##" {
        return body.lines().find_map(|line| {
            let rest = line.trim().strip_prefix("## ")?;
            let version = rest.trim_matches('[').split([' ', ']']).next()?;
            (version != "Unreleased").then(|| version.to_string())
        });
    }
    body.lines().find_map(|line| {
        let rest = line.trim().strip_prefix(key)?.trim();
        if let Some(after_open) = rest.strip_prefix('`') {
            return Some(after_open.split('`').next()?.trim().to_string());
        }
        Some(rest.to_string())
    })
}

fn nix_version(body: &str) -> Option<String> {
    body.lines().find_map(|line| {
        let trimmed = line.trim();
        let rest = trimmed.strip_prefix("version = \"")?;
        Some(rest.split('"').next()?.to_string())
    })
}

fn cargo_lock_package_version(body: &str, package: &str) -> Option<String> {
    let mut in_package = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            in_package = false;
        } else if trimmed == format!("name = \"{package}\"") {
            in_package = true;
        } else if in_package && trimmed.starts_with("version = \"") {
            return Some(
                trimmed
                    .trim_start_matches("version = \"")
                    .split('"')
                    .next()?
                    .to_string(),
            );
        }
    }
    None
}

fn parse_semver(value: &str) -> Option<SemVer> {
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() != 3 {
        return None;
    }
    Some(SemVer {
        major: parse_part(parts[0])?,
        minor: parse_part(parts[1])?,
        patch: parse_part(parts[2])?,
    })
}

fn parse_part(value: &str) -> Option<u64> {
    if value.is_empty() || (value.len() > 1 && value.starts_with('0')) {
        return None;
    }
    value.parse().ok()
}

fn legal_bump(previous: SemVer, current: SemVer, bump: &str) -> bool {
    match bump {
        "major" => current.major == previous.major + 1 && current.minor == 0 && current.patch == 0,
        "minor" => {
            current.major == previous.major
                && current.minor == previous.minor + 1
                && current.patch == 0
        }
        "patch" => {
            current.major == previous.major
                && current.minor == previous.minor
                && current.patch == previous.patch + 1
        }
        _ => false,
    }
}

fn equal_check(
    check_id: &str,
    path: &str,
    expected: impl Into<String>,
    actual: impl Into<String>,
    pass: bool,
) -> Diagnostic {
    let expected = expected.into();
    let actual = actual.into();
    if pass {
        Diagnostic::pass(check_id, ReportSurface::Version, path, expected)
    } else {
        Diagnostic::fail(
            check_id,
            ReportSurface::Version,
            path,
            expected,
            actual,
            "repair docs/version-roadmap.toml or the referenced release surface",
        )
    }
}

fn release_for_plan<'a>(roadmap: &'a Roadmap, plan_id: &str) -> Result<&'a Release> {
    roadmap
        .releases
        .iter()
        .find(|release| {
            release.plan_id == plan_id
                || release
                    .covered_plan_ids
                    .iter()
                    .any(|covered| covered == plan_id)
        })
        .ok_or_else(|| format!("no release entry covers plan {plan_id}"))
}

fn validate_report_has_no_failures(report: CheckReport) -> RuntimeResult<()> {
    if report.has_failures() {
        Err(RuntimeFailure::json(
            FailureCode::DiagnosticReport,
            report.to_json()?,
        ))
    } else {
        Ok(())
    }
}

fn parse_optional_json(args: &[String]) -> RuntimeResult<bool> {
    match args {
        [] => Ok(false),
        [flag, format] if flag == "--format" && format == "json" => Ok(true),
        _ => Err(usage().into()),
    }
}

fn parse_plan_and_json(args: &[String]) -> RuntimeResult<(&str, bool)> {
    match args {
        [plan_id] => Ok((plan_id, false)),
        [plan_id, flag, format] if flag == "--format" && format == "json" => Ok((plan_id, true)),
        _ => Err(usage().into()),
    }
}

fn parse_prerelease_args(args: &[String]) -> RuntimeResult<(&str, u64, bool)> {
    match args {
        [plan_id, flag, rc] if flag == "--rc" => Ok((plan_id, parse_rc(rc)?, false)),
        [plan_id, flag, rc, format_flag, format]
            if flag == "--rc" && format_flag == "--format" && format == "json" =>
        {
            Ok((plan_id, parse_rc(rc)?, true))
        }
        _ => Err(usage().into()),
    }
}

fn parse_rc(value: &str) -> RuntimeResult<u64> {
    value
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| RuntimeFailure::from("rc must be a positive integer"))
}

fn format_next(release: &Release) -> String {
    [
        format!("version={}", release.version),
        format!("tag={}", release.tag),
        format!("prerelease_tag={}", release.prerelease_tag),
        format!("commit_subject={}", release.commit_subject),
        "manual_final_release=true".to_string(),
    ]
    .join("\n")
}

fn format_prerelease(roadmap: &Roadmap, release: &Release) -> String {
    [
        format!("version={}", release.version),
        format!("prerelease_tag={}", release.prerelease_tag),
        format!("commit_subject={}", release.commit_subject),
        format!(
            "push_branch=git push {} {}",
            roadmap.remote, roadmap.release_branch
        ),
        format!(
            "push_prerelease_tag=git push {} {}",
            roadmap.remote, release.prerelease_tag
        ),
        "final_release_manual=true".to_string(),
    ]
    .join("\n")
}

fn format_human(report: &CheckReport) -> String {
    let mut lines = report
        .checks
        .iter()
        .map(|check| {
            format!(
                "{} {} {}: expected {}, actual {}",
                check.status, check.check_id, check.path, check.expected, check.actual
            )
        })
        .collect::<Vec<_>>();
    lines.push(format!(
        "version check summary: {} pass, {} warn, {} fail, {} skip",
        report.summary.pass, report.summary.warn, report.summary.fail, report.summary.skip
    ));
    lines.join("\n")
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("git {}: {error}", args.join(" ")))?;
    if !output.status.success() {
        return Err(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn usage() -> String {
    "usage: task-registry-flow version-check {validate|next <plan_id>|prerelease <plan_id> --rc <n>|release-check} [--format json]".to_string()
}
