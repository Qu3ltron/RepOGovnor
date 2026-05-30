use crate::model::*;
use crate::{append_event, load_registry, normalize_relative_path, timestamp, truncate_detail};
use serde_json::Value;
use std::env;
use std::io::{self, Read};
use std::path::Path;

#[derive(Default)]
pub(crate) struct HookInspection {
    pub(crate) paths: Vec<String>,
    pub(crate) tool_names: Vec<String>,
    command_write_without_path: bool,
}

impl HookInspection {
    pub(crate) fn uncertain_write(&self) -> bool {
        if self.command_write_without_path {
            return true;
        }
        self.paths.is_empty()
            && self
                .tool_names
                .iter()
                .any(|name| is_write_hook_tool(name.as_str()))
    }
}

pub(crate) fn verify_mutation_hook(root: &Path, _format: &str) -> Result<()> {
    let mut stdin = String::new();
    io::stdin()
        .read_to_string(&mut stdin)
        .map_err(|error| format!("read hook stdin: {error}"))?;
    verify_mutation_payload(root, &stdin)
}

pub(crate) fn verify_mutation_payload(root: &Path, stdin: &str) -> Result<()> {
    if stdin.trim().is_empty() {
        return Ok(());
    }

    let value = serde_json::from_str::<Value>(stdin)
        .map_err(|error| format!("parse hook JSON: {error}"))?;
    let inspection = inspect_hook_payload(&value);
    if inspection.uncertain_write() {
        return Err(
            "write-intent hook payload did not expose a deterministic target path".to_string(),
        );
    }
    if inspection.paths.is_empty() {
        return Ok(());
    }

    let mut implementation_paths = Vec::new();
    for raw_path in inspection.paths {
        let path = normalize_hook_path(root, &raw_path)?;
        if !is_governance_write(&path) {
            implementation_paths.push(path);
        }
    }
    if implementation_paths.is_empty() {
        return Ok(());
    }

    let registry = load_registry(root)?;
    let allowed_targets = registry
        .tasks
        .iter()
        .filter(|task| ACTIVE_TARGET_STATUSES.contains(&task.status.as_str()))
        .flat_map(|task| task.targets.iter().map(|target| target.file.as_str()))
        .collect::<Vec<_>>();

    for path in implementation_paths {
        if target_allows(&path, &allowed_targets) {
            continue;
        }
        let _ = append_event(
            root,
            EventRecord {
                timestamp: timestamp(),
                command: "verify-mutation-hook".to_string(),
                outcome: "mutation-denied".to_string(),
                duration_ms: 0,
                detail: truncate_detail(&path),
            },
        );
        return Err(format!(
            "{path} is not bound to an active registry task target"
        ));
    }
    Ok(())
}

pub(crate) fn inspect_hook_payload(value: &Value) -> HookInspection {
    let mut inspection = HookInspection::default();
    collect_hook_signals(value, &mut inspection);
    inspection
}

fn collect_hook_signals(value: &Value, inspection: &mut HookInspection) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let normalized_key = normalize_hook_key(key);
                if is_hook_path_key(&normalized_key)
                    && let Some(path) = child.as_str()
                {
                    inspection.paths.push(path.to_string());
                }
                if is_hook_tool_key(&normalized_key)
                    && let Some(name) = child.as_str()
                {
                    inspection.tool_names.push(name.to_string());
                }
                if is_hook_command_key(&normalized_key)
                    && let Some(command) = child.as_str()
                {
                    let command_paths = collect_command_paths(command);
                    if command_paths.is_empty() && command_has_write_intent(command) {
                        inspection.command_write_without_path = true;
                    }
                    inspection.paths.extend(command_paths);
                }
                collect_hook_signals(child, inspection);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_hook_signals(item, inspection);
            }
        }
        _ => {}
    }
}

fn normalize_hook_key(key: &str) -> String {
    key.chars()
        .filter(|ch| *ch != '_' && *ch != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_hook_path_key(key: &str) -> bool {
    matches!(
        key,
        "path"
            | "file"
            | "filepath"
            | "target"
            | "targetpath"
            | "targetfile"
            | "absolutepath"
            | "relativepath"
            | "filename"
    )
}

fn is_hook_tool_key(key: &str) -> bool {
    matches!(key, "toolname" | "name")
}

fn is_hook_command_key(key: &str) -> bool {
    matches!(key, "command" | "commandline")
}

fn is_write_hook_tool(name: &str) -> bool {
    matches!(
        name,
        "apply_patch"
            | "Edit"
            | "Write"
            | "edit_file"
            | "write_to_file"
            | "replace_file_content"
            | "multi_replace_file_content"
    )
}

fn collect_command_paths(command: &str) -> Vec<String> {
    let mut paths = collect_patch_paths(command);
    let tokens = shell_like_tokens(command);
    for (index, token) in tokens.iter().enumerate() {
        if matches!(
            token.as_str(),
            ">" | ">>" | "2>" | "1>" | "--path" | "--file"
        ) {
            if let Some(next) = tokens.get(index + 1) {
                push_candidate_path(&mut paths, next);
            }
            continue;
        }
        if matches!(token.as_str(), "cp" | "mv" | "rm" | "touch" | "tee") {
            if let Some(last) = tokens.last() {
                push_candidate_path(&mut paths, last);
            }
            continue;
        }
        if token == "sed"
            && tokens.iter().any(|part| part.starts_with("-i"))
            && let Some(last) = tokens.last()
        {
            push_candidate_path(&mut paths, last);
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn collect_patch_paths(text: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for line in text.lines() {
        for marker in ["*** Add File:", "*** Update File:", "*** Delete File:"] {
            if let Some(path) = line.trim().strip_prefix(marker) {
                push_candidate_path(&mut paths, path.trim());
            }
        }
    }
    paths
}

fn command_has_write_intent(command: &str) -> bool {
    let tokens = shell_like_tokens(command);
    tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            ">" | ">>" | "2>" | "1>" | "tee" | "cp" | "mv" | "rm" | "touch"
        )
    }) || tokens.iter().any(|token| token == "sed")
        && tokens.iter().any(|token| token.starts_with("-i"))
        || tokens.iter().any(|token| token == "perl")
            && tokens
                .iter()
                .any(|token| token.starts_with("-i") || token.starts_with("-pi"))
        || command.contains("*** Begin Patch")
}

fn push_candidate_path(paths: &mut Vec<String>, raw: &str) {
    let candidate = raw
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_end_matches(',')
        .trim_end_matches(';');
    if candidate.is_empty()
        || candidate.starts_with('-')
        || candidate.starts_with("http://")
        || candidate.starts_with("https://")
        || candidate.starts_with('$')
    {
        return;
    }
    if candidate.contains('/') || candidate.contains('.') {
        paths.push(candidate.to_string());
    }
}

fn shell_like_tokens(command: &str) -> Vec<String> {
    command
        .split_whitespace()
        .map(|token| {
            token
                .trim_matches(|ch| matches!(ch, '"' | '\'' | '`'))
                .to_string()
        })
        .filter(|token| !token.is_empty())
        .collect()
}

fn normalize_hook_path(root: &Path, raw_path: &str) -> Result<String> {
    if raw_path.trim().is_empty() || raw_path.starts_with("file://") {
        return Err(format!("unsupported hook path: {raw_path}"));
    }
    let raw = Path::new(raw_path);
    let relative = if raw.is_absolute() {
        let cwd = env::current_dir().map_err(|error| format!("current_dir: {error}"))?;
        raw.strip_prefix(cwd.join(root))
            .or_else(|_| raw.strip_prefix(&cwd))
            .map_err(|_| format!("{raw_path} is outside the repo root"))?
            .to_path_buf()
    } else {
        raw.to_path_buf()
    };
    normalize_relative_path(&relative.to_string_lossy())
}

fn is_governance_write(path: &str) -> bool {
    GOVERNANCE_WRITE_FILES.contains(&path)
        || GOVERNANCE_WRITE_PREFIXES
            .iter()
            .any(|prefix| path.starts_with(prefix))
}

pub(crate) fn target_allows(path: &str, targets: &[&str]) -> bool {
    targets.iter().any(|target| {
        let Ok(target) = normalize_relative_path(target) else {
            return false;
        };
        path == target || target.ends_with('/') && path.starts_with(&target)
    })
}
