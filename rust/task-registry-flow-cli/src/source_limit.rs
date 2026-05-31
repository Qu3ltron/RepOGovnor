use crate::model::{Result, SOURCE_LINE_LIMIT};
use crate::reports::{RuntimeFailure, RuntimeResult};
use crate::schema::{CheckReport, Diagnostic};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct Violation {
    pub(crate) path: String,
    pub(crate) lines: usize,
    pub(crate) limit: usize,
}

#[derive(Debug, Clone, Serialize)]
struct SplitPlan {
    path: String,
    lines: usize,
    limit: usize,
    targets: Vec<SplitTarget>,
}

#[derive(Debug, Clone, Serialize)]
struct SplitPlanReport {
    schema_version: i64,
    surface: String,
    plans: Vec<SplitPlan>,
}

#[derive(Debug, Clone, Serialize)]
struct SplitTarget {
    destination: String,
    reason: String,
    ranges: Vec<String>,
}

#[derive(Debug, Clone)]
struct LineRange {
    start: usize,
    end: usize,
}

pub(crate) fn run_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err(source_limit_usage().into());
    };
    match subcommand {
        "check" => {
            let mut json = false;
            if args.len() == 3 && args[1] == "--format" && args[2] == "json" {
                json = true;
            } else if args.len() != 1 {
                return Err(source_limit_usage().into());
            }
            if json {
                let report = check_report(root)?;
                let output = report.to_json()?;
                if report.has_failures() {
                    Err(RuntimeFailure::json(output))
                } else {
                    Ok(output)
                }
            } else {
                check(root)?;
                Ok(format!(
                    "source file limit ok: max {SOURCE_LINE_LIMIT} lines"
                ))
            }
        }
        "plan" => {
            let mut path = None;
            let mut json = false;
            let mut index = 1;
            while index < args.len() {
                match args[index].as_str() {
                    "--path" => {
                        index += 1;
                        path = args.get(index).map(String::as_str);
                    }
                    "--all" => path = None,
                    "--format" => {
                        index += 1;
                        json = args.get(index).is_some_and(|value| value == "json");
                    }
                    _ => return Err(source_limit_usage().into()),
                }
                index += 1;
            }
            let output = plan(root, path, json)?;
            Ok(output)
        }
        _ => Err(source_limit_usage().into()),
    }
}

pub(crate) fn check(root: &Path) -> Result<()> {
    let violations = violations(root)?;
    if violations.is_empty() {
        return Ok(());
    }
    let mut message =
        format!("source file limit exceeded: max {SOURCE_LINE_LIMIT} lines. Violations:");
    for violation in violations {
        message.push_str(&format!(
            "\n- {}: {} lines; split plan: .codex/scripts/task-registry source-limit plan --path {}",
            violation.path, violation.lines, violation.path
        ));
    }
    Err(message)
}

pub(crate) fn check_report(root: &Path) -> Result<CheckReport> {
    let violations = violations(root)?;
    let checks = if violations.is_empty() {
        vec![Diagnostic::pass(
            "source-limit",
            "source-limit",
            ".",
            format!("all source files at or below {SOURCE_LINE_LIMIT} lines"),
        )]
    } else {
        violations
            .into_iter()
            .map(|violation| {
                Diagnostic::fail(
                    "source-limit",
                    "source-limit",
                    violation.path,
                    format!("at most {} lines", violation.limit),
                    format!("{} lines", violation.lines),
                    "split the file or run source-limit plan for a focused split plan",
                )
            })
            .collect()
    };
    CheckReport::new("source-limit", checks)
}

pub(crate) fn plan(root: &Path, path: Option<&str>, json: bool) -> Result<String> {
    let plans = if let Some(path) = path {
        let rel = normalize_relative_path(path)?;
        let line_count = count_lines(&root.join(&rel))?;
        if line_count <= SOURCE_LINE_LIMIT {
            Vec::new()
        } else {
            vec![split_plan(root, &rel, line_count)?]
        }
    } else {
        violations(root)?
            .into_iter()
            .map(|violation| split_plan(root, &violation.path, violation.lines))
            .collect::<Result<Vec<_>>>()?
    };

    if json {
        return serde_json::to_string_pretty(&SplitPlanReport {
            schema_version: 1,
            surface: "source-limit-plan".to_string(),
            plans,
        })
        .map_err(|error| format!("serialize source-limit plan: {error}"));
    }
    if plans.is_empty() {
        return Ok("No source file split needed.".to_string());
    }
    let mut output = String::new();
    for plan in plans {
        output.push_str(&format!(
            "# Split plan for `{}`\n\n{} lines; hard limit {}.\n\n",
            plan.path, plan.lines, plan.limit
        ));
        for target in plan.targets {
            output.push_str(&format!(
                "- `{}` - {}; move {}.\n",
                target.destination,
                target.reason,
                target.ranges.join(", ")
            ));
        }
        output.push('\n');
    }
    Ok(output.trim_end().to_string())
}

fn violations(root: &Path) -> Result<Vec<Violation>> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    let mut violations = Vec::new();
    for path in files {
        let rel = relative_path(root, &path)?;
        let lines = count_lines(&path)?;
        if lines > SOURCE_LINE_LIMIT {
            violations.push(Violation {
                path: rel,
                lines,
                limit: SOURCE_LINE_LIMIT,
            });
        }
    }
    violations.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(violations)
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|error| format!("read {}: {error}", dir.display()))? {
        let entry = entry.map_err(|error| format!("read {} entry: {error}", dir.display()))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("read {} file type: {error}", entry.path().display()))?;
        if file_type.is_symlink() {
            continue;
        }
        let path = entry.path();
        let rel = relative_path(root, &path)?;
        if file_type.is_dir() {
            if !skip_dir(&rel) {
                collect_files(root, &path, out)?;
            }
            continue;
        }
        if file_type.is_file() && should_check_file(&rel) {
            out.push(path);
        }
    }
    Ok(())
}

fn split_plan(root: &Path, path: &str, lines: usize) -> Result<SplitPlan> {
    let body =
        fs::read_to_string(root.join(path)).map_err(|error| format!("read {path}: {error}"))?;
    let targets = if path.ends_with(".rs") {
        rust_split_targets(path, &body)
    } else {
        chunk_split_targets(path, lines)
    };
    Ok(SplitPlan {
        path: path.to_string(),
        lines,
        limit: SOURCE_LINE_LIMIT,
        targets,
    })
}

fn rust_split_targets(path: &str, body: &str) -> Vec<SplitTarget> {
    let mut buckets: BTreeMap<&str, Vec<LineRange>> = BTreeMap::new();
    let mut current_start = 1usize;
    let mut current_bucket = "module_support";
    for (index, line) in body.lines().enumerate() {
        let line_number = index + 1;
        if let Some(bucket) = rust_bucket(line.trim_start()) {
            if line_number > current_start {
                buckets.entry(current_bucket).or_default().push(LineRange {
                    start: current_start,
                    end: line_number - 1,
                });
            }
            current_bucket = bucket;
            current_start = line_number;
        }
    }
    let total = body.lines().count();
    buckets.entry(current_bucket).or_default().push(LineRange {
        start: current_start,
        end: total,
    });

    let stem = path.trim_end_matches(".rs");
    buckets
        .into_iter()
        .flat_map(|(bucket, ranges)| rust_bucket_targets(stem, bucket, ranges))
        .collect()
}

fn rust_bucket_targets(stem: &str, bucket: &str, ranges: Vec<LineRange>) -> Vec<SplitTarget> {
    let chunks = chunk_ranges(ranges, SOURCE_LINE_LIMIT - 200);
    let multiple = chunks.len() > 1;
    chunks
        .into_iter()
        .enumerate()
        .map(|(index, ranges)| {
            let destination = if multiple {
                format!("{stem}/{bucket}_part_{:03}.rs", index + 1)
            } else {
                format!("{stem}/{bucket}.rs")
            };
            SplitTarget {
                destination,
                reason: rust_bucket_reason(bucket).to_string(),
                ranges: ranges
                    .into_iter()
                    .map(|range| format!("lines {}-{}", range.start, range.end))
                    .collect(),
            }
        })
        .collect()
}

fn chunk_ranges(ranges: Vec<LineRange>, chunk_size: usize) -> Vec<Vec<LineRange>> {
    let mut chunks = Vec::new();
    let mut current = Vec::new();
    let mut current_lines = 0usize;

    for range in ranges {
        let mut start = range.start;
        while start <= range.end {
            if current_lines == chunk_size {
                chunks.push(current);
                current = Vec::new();
                current_lines = 0;
            }
            let remaining = chunk_size - current_lines;
            let available = range.end - start + 1;
            let take = remaining.min(available);
            current.push(LineRange {
                start,
                end: start + take - 1,
            });
            current_lines += take;
            start += take;
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn rust_bucket(line: &str) -> Option<&'static str> {
    if line.starts_with("#[cfg(test)]") {
        return Some("tests");
    }
    if line.starts_with("const ")
        || line.starts_with("type ")
        || line.starts_with("struct ")
        || line.starts_with("pub struct ")
        || line.starts_with("enum ")
        || line.starts_with("pub enum ")
    {
        return Some("model");
    }
    if line.starts_with("fn main") || line.starts_with("fn run") || line.starts_with("fn usage") {
        return Some("cli");
    }
    if line.contains("registry")
        || line.starts_with("fn activate")
        || line.starts_with("fn update_task")
        || line.starts_with("fn defer")
        || line.starts_with("fn report")
        || line.starts_with("fn archive")
    {
        return Some("registry");
    }
    if line.starts_with("fn validate")
        || line.starts_with("fn assert")
        || line.starts_with("fn reject")
    {
        return Some("validation");
    }
    if line.contains("manifest")
        || line.contains("behavior")
        || line.starts_with("fn run_confirmation")
    {
        return Some("manifest");
    }
    if line.contains("hook") || line.contains("metrics") || line.starts_with("fn collect_hook") {
        return Some("hooks_metrics");
    }
    if line.starts_with("fn normalized")
        || line.starts_with("fn today")
        || line.starts_with("fn timestamp")
        || line.starts_with("fn truncate")
        || line.starts_with("fn relative")
    {
        return Some("util");
    }
    None
}

fn rust_bucket_reason(bucket: &str) -> &str {
    match bucket {
        "cli" => "command dispatch and user-facing CLI output",
        "model" => "shared constants and typed registry structures",
        "registry" => "registry lifecycle, persistence, and reporting",
        "validation" => "typed validation and invariant checks",
        "manifest" => "Task Manifest parsing and behavior confirmation",
        "hooks_metrics" => "mutation-hook and local receipt metrics",
        "tests" => "unit and integration tests",
        _ => "module support and utilities",
    }
}

fn chunk_split_targets(path: &str, lines: usize) -> Vec<SplitTarget> {
    let mut targets = Vec::new();
    let chunk = SOURCE_LINE_LIMIT - 200;
    let mut start = 1usize;
    let mut index = 1usize;
    while start <= lines {
        let end = (start + chunk - 1).min(lines);
        let destination = numbered_destination(path, index);
        targets.push(SplitTarget {
            destination,
            reason: "stable range chunk below the hard line limit".to_string(),
            ranges: vec![format!("lines {start}-{end}")],
        });
        start = end + 1;
        index += 1;
    }
    targets
}

fn numbered_destination(path: &str, index: usize) -> String {
    let original = Path::new(path);
    let stem = original
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("part");
    let extension = original
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let parent = original.parent().unwrap_or_else(|| Path::new(""));
    let file = if extension.is_empty() {
        format!("{stem}_part_{index:03}")
    } else {
        format!("{stem}_part_{index:03}.{extension}")
    };
    parent.join(file).to_string_lossy().replace('\\', "/")
}

fn count_lines(path: &Path) -> Result<usize> {
    let body =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(body.lines().count())
}

fn should_check_file(path: &str) -> bool {
    if skip_file(path) {
        return false;
    }
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let known_source_extension = matches!(
        extension,
        "rs" | "go"
            | "java"
            | "kt"
            | "kts"
            | "swift"
            | "c"
            | "cc"
            | "cpp"
            | "cxx"
            | "h"
            | "hh"
            | "hpp"
            | "cs"
            | "rb"
            | "php"
            | "lua"
            | "scala"
            | "clj"
            | "cljs"
            | "ex"
            | "exs"
            | "erl"
            | "hrl"
            | "sh"
            | "bash"
            | "zsh"
            | "py"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "css"
            | "scss"
            | "sass"
            | "html"
            | "htm"
            | "xml"
            | "sql"
            | "graphql"
            | "gql"
            | "proto"
            | "nix"
            | "ini"
            | "cfg"
            | "conf"
            | "env"
            | "toml"
            | "json"
            | "yaml"
            | "yml"
            | "md"
            | "template"
    );
    known_source_extension
        || path == "AGENTS.md"
        || path == "GEMINI.md"
        || is_known_config_name(path)
        || is_extensionless_script_or_hook(path, extension)
}

fn is_known_config_name(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    matches!(
        name,
        ".gitignore"
            | ".gitattributes"
            | ".editorconfig"
            | ".env"
            | ".env.example"
            | "Dockerfile"
            | "Containerfile"
    )
}

fn is_extensionless_script_or_hook(path: &str, extension: &str) -> bool {
    if !extension.is_empty() {
        return false;
    }
    let name = Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    matches!(
        name,
        "Dockerfile" | "Makefile" | "Justfile" | "Taskfile" | "Rakefile" | "Gemfile" | "Procfile"
    ) || path
        .split('/')
        .any(|part| matches!(part, "scripts" | "hooks" | "tools"))
}

fn skip_dir(path: &str) -> bool {
    path.split('/').any(|part| {
        matches!(
            part,
            ".git"
                | "target"
                | "node_modules"
                | ".cache"
                | ".next"
                | ".nuxt"
                | ".pytest_cache"
                | ".mypy_cache"
                | ".ruff_cache"
                | ".svelte-kit"
                | ".venv"
                | ".workspace-render"
                | "__pycache__"
                | "coverage"
                | "dist"
                | "build"
                | "out"
                | "venv"
                | "vendor"
        )
    })
}

fn skip_file(path: &str) -> bool {
    let name = Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    matches!(
        name,
        "Cargo.lock"
            | "package-lock.json"
            | "pnpm-lock.yaml"
            | "yarn.lock"
            | "bun.lockb"
            | "Gemfile.lock"
            | "poetry.lock"
            | "uv.lock"
            | "Pipfile.lock"
            | "composer.lock"
            | "deno.lock"
            | "flake.lock"
            | "go.sum"
    ) || path == "docs/task-registry/events.jsonl"
        || path.starts_with("docs/task-registry/archive/")
}

fn normalize_relative_path(path: &str) -> Result<String> {
    let path = path.replace('\\', "/");
    let mut parts = Vec::new();
    for component in Path::new(&path).components() {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => return Err(format!("path must not contain '..': {path}")),
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("path must be relative: {path}"));
            }
        }
    }
    if parts.is_empty() {
        return Err("path must not be empty".to_string());
    }
    Ok(parts.join("/"))
}

fn relative_path(root: &Path, path: &Path) -> Result<String> {
    path.strip_prefix(root)
        .map_err(|error| format!("strip prefix {}: {error}", path.display()))
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

fn source_limit_usage() -> String {
    "usage: task-registry-flow source-limit {check [--format json]|plan [--all|--path <file>] [--format json]}"
        .to_string()
}
