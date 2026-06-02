use crate::hook_io;
use crate::model::*;
use crate::schema::{
    AgentModelAttribution, EventOutcome, HookFormat, ModelIdentityStatus, MutationAttribution,
    MutationAttributionDecision, MutationScope,
};
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
    command_write_with_path: bool,
    pub(crate) model: Option<String>,
    pub(crate) session_id: Option<String>,
    pub(crate) turn_id: Option<String>,
    pub(crate) tool_use_id: Option<String>,
    pub(crate) hook_event_name: Option<String>,
}

impl HookInspection {
    pub(crate) fn has_write_intent(&self) -> bool {
        self.command_write_without_path
            || self.command_write_with_path
            || self
                .tool_names
                .iter()
                .any(|name| is_write_hook_tool(name.as_str()))
    }

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

pub(crate) fn verify_mutation_hook(root: &Path, format: HookFormat) -> Result<()> {
    let mut stdin = String::new();
    io::stdin()
        .read_to_string(&mut stdin)
        .map_err(|error| format!("read hook stdin: {error}"))?;
    verify_mutation_payload_for_format(root, format, &stdin)
}

pub(crate) fn verify_mutation_payload_for_format(
    root: &Path,
    format: HookFormat,
    stdin: &str,
) -> Result<()> {
    verify_mutation_payload_inner(root, Some(format), stdin)
}

fn verify_mutation_payload_inner(
    root: &Path,
    format: Option<HookFormat>,
    stdin: &str,
) -> Result<()> {
    if stdin.trim().is_empty() {
        return Ok(());
    }

    let value = serde_json::from_str::<Value>(stdin)
        .map_err(|error| format!("parse hook JSON: {error}"))?;
    if let Some(format) = format {
        hook_io::validate_payload_shape(format, &value)?;
    }
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
    for raw_path in &inspection.paths {
        let path = normalize_hook_path(root, raw_path)?;
        if !is_plan_bootstrap_write(&path) {
            implementation_paths.push(path);
        }
    }
    if implementation_paths.is_empty() {
        return Ok(());
    }

    let registry = load_registry(root)?;
    let allowed_scopes = registry
        .tasks
        .iter()
        .filter(|task| ACTIVE_TARGET_STATUSES.contains(&task.status))
        .flat_map(|task| task.targets.iter())
        .map(|target| MutationScope::from_task_target(&target.file))
        .collect::<Result<Vec<_>>>()?;

    for path in &implementation_paths {
        if allowed_scopes.iter().any(|scope| scope.allows(path)) {
            continue;
        }
        let _ = append_event(
            root,
            EventRecord::mutation_denied(
                timestamp(),
                0,
                path.clone(),
                truncate_detail(&format!(
                    "{path} is not bound to an active registry task target"
                )),
            ),
        );
        return Err(format!(
            "{path} is not bound to an active registry task target"
        ));
    }
    if let Some(format) = format
        && inspection.has_write_intent()
    {
        record_mutation_attribution(root, format, &inspection, &implementation_paths)?;
    }
    Ok(())
}

pub(crate) fn inspect_hook_payload(value: &Value) -> HookInspection {
    let mut inspection = HookInspection::default();
    collect_top_level_hook_metadata(value, &mut inspection);
    collect_hook_signals(value, &mut inspection);
    inspection
}

fn collect_top_level_hook_metadata(value: &Value, inspection: &mut HookInspection) {
    let Some(map) = value.as_object() else {
        return;
    };
    inspection.model = string_field(map.get("model"));
    inspection.session_id = string_field(map.get("session_id"));
    inspection.turn_id = string_field(map.get("turn_id"));
    inspection.tool_use_id = string_field(map.get("tool_use_id"));
    inspection.hook_event_name =
        string_field(map.get("hook_event_name")).or_else(|| string_field(map.get("hookEventName")));
}

fn string_field(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
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
                    let has_write_intent = command_has_write_intent(command);
                    if has_write_intent && !command_paths.is_empty() {
                        inspection.command_write_with_path = true;
                    }
                    if command_has_ambiguous_write_intent(command)
                        || command_paths.is_empty() && has_write_intent
                    {
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

fn record_mutation_attribution(
    root: &Path,
    format: HookFormat,
    inspection: &HookInspection,
    target_paths: &[String],
) -> Result<()> {
    let pre_tool_use = inspection
        .hook_event_name
        .as_deref()
        .map(|name| name == "PreToolUse")
        .unwrap_or(true);
    let measured_codex = format == HookFormat::Codex
        && inspection.model.is_some()
        && inspection.session_id.is_some()
        && inspection.turn_id.is_some()
        && inspection.tool_use_id.is_some();
    if format == HookFormat::Codex && pre_tool_use && !measured_codex {
        let summary = "Codex mutation model identity missing".to_string();
        let _ = append_event(
            root,
            EventRecord::mutation_attribution(
                timestamp(),
                0,
                summary.clone(),
                EventOutcome::MutationDenied,
                target_paths
                    .first()
                    .cloned()
                    .unwrap_or_else(|| ".".to_string()),
                agent_attribution(format, inspection, ModelIdentityStatus::Unmeasured),
                mutation_attribution(
                    format,
                    MutationAttributionDecision::Denied,
                    target_paths.to_vec(),
                ),
            ),
        );
        return Err(summary);
    }

    let status = if measured_codex {
        ModelIdentityStatus::Measured
    } else {
        ModelIdentityStatus::Unmeasured
    };
    let decision = if pre_tool_use {
        MutationAttributionDecision::Allowed
    } else {
        MutationAttributionDecision::Observed
    };
    append_event(
        root,
        EventRecord::mutation_attribution(
            timestamp(),
            0,
            format!("{} mutation attribution {}", format, decision.as_str()),
            EventOutcome::Ok,
            target_paths
                .first()
                .cloned()
                .unwrap_or_else(|| ".".to_string()),
            agent_attribution(format, inspection, status),
            mutation_attribution(format, decision, target_paths.to_vec()),
        ),
    )
}

fn agent_attribution(
    format: HookFormat,
    inspection: &HookInspection,
    identity_status: ModelIdentityStatus,
) -> AgentModelAttribution {
    AgentModelAttribution {
        provider: format.to_string(),
        adapter: format.to_string(),
        identity_status,
        evidence_source: "hook-payload".to_string(),
        model_slug: inspection.model.clone(),
        session_id: inspection.session_id.clone(),
        turn_id: inspection.turn_id.clone(),
        tool_use_id: inspection.tool_use_id.clone(),
        hook_event_name: inspection.hook_event_name.clone(),
    }
}

fn mutation_attribution(
    format: HookFormat,
    decision: MutationAttributionDecision,
    target_paths: Vec<String>,
) -> MutationAttribution {
    MutationAttribution {
        decision,
        hook_format: format,
        target_paths,
    }
}

fn collect_command_paths(command: &str) -> Vec<String> {
    let mut paths = collect_patch_paths(command);
    paths.extend(inspect_compact_redirections(command).paths);
    paths.extend(collect_inline_write_paths(command));
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
        || inspect_compact_redirections(command).has_file_write
        || command.contains("*** Begin Patch")
        || inline_command_has_write_intent(command)
}

fn command_has_ambiguous_write_intent(command: &str) -> bool {
    inspect_compact_redirections(command).ambiguous_write
        || inspect_inline_open_writes(command).ambiguous_write
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

fn inline_command_has_write_intent(command: &str) -> bool {
    let lowered = command.to_ascii_lowercase();
    [
        ".write_text(",
        ".write_bytes(",
        ".write_all(",
        ".writestr(",
        "writefilesync(",
        "writefile(",
        "appendfilesync(",
        "appendfile(",
        "fs.write",
        "file.write(",
        "file.openwrite(",
        "createwriter(",
        "createtext(",
        "truncate(",
        "removefile(",
        "deletefile(",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
        || inline_open_has_write_mode(&lowered)
}

fn inline_open_has_write_mode(command: &str) -> bool {
    inspect_inline_open_writes(command).has_write
}

fn collect_inline_write_paths(command: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for marker in ["Path(", "pathlib.Path("] {
        collect_quoted_arg_after(command, marker, &mut paths);
    }
    paths.extend(inspect_inline_open_writes(command).paths);
    paths.sort();
    paths.dedup();
    paths
}

#[derive(Default)]
struct CompactRedirectionInspection {
    paths: Vec<String>,
    has_file_write: bool,
    ambiguous_write: bool,
}

fn inspect_compact_redirections(command: &str) -> CompactRedirectionInspection {
    let mut inspection = CompactRedirectionInspection::default();
    for token in shell_like_tokens(command) {
        let Some(target) = compact_redirection_target(&token) else {
            continue;
        };
        if target.is_empty() || target.starts_with('&') {
            continue;
        }
        inspection.has_file_write = true;
        if target.starts_with('$') || target.contains('$') {
            inspection.ambiguous_write = true;
        } else {
            push_candidate_path(&mut inspection.paths, target);
        }
    }
    inspection.paths.sort();
    inspection.paths.dedup();
    inspection
}

fn compact_redirection_target(token: &str) -> Option<&str> {
    for prefix in ["1>>", "2>>", ">>", "1>", "2>", ">"] {
        if let Some(target) = token.strip_prefix(prefix) {
            return Some(target);
        }
    }
    None
}

#[derive(Default)]
struct InlineOpenWriteInspection {
    paths: Vec<String>,
    has_write: bool,
    ambiguous_write: bool,
}

fn inspect_inline_open_writes(command: &str) -> InlineOpenWriteInspection {
    let lowered = command.to_ascii_lowercase();
    let mut inspection = InlineOpenWriteInspection::default();
    let mut offset = 0usize;
    while let Some(index) = lowered[offset..].find("open(") {
        let open_index = offset + index;
        let args_start = open_index + "open(".len();
        let after_lowered = &lowered[args_start..];
        let after_original = &command[args_start..];
        let Some(close_index) = after_lowered.find(')') else {
            inspection.has_write = true;
            inspection.ambiguous_write = true;
            break;
        };
        let args_lowered = &after_lowered[..close_index];
        let args_original = &after_original[..close_index];
        if inline_open_args_have_write_mode(args_lowered) {
            inspection.has_write = true;
            if let Some(path) = first_quoted_arg(args_original) {
                let before = inspection.paths.len();
                push_candidate_path(&mut inspection.paths, path);
                if inspection.paths.len() == before {
                    inspection.ambiguous_write = true;
                }
            } else {
                inspection.ambiguous_write = true;
            }
        }
        offset = args_start + close_index + 1;
    }
    inspection.paths.sort();
    inspection.paths.dedup();
    inspection
}

fn inline_open_args_have_write_mode(args: &str) -> bool {
    let Some(comma_index) = args.find(',') else {
        return false;
    };
    let mode_args = &args[comma_index + 1..];
    ["'w", "\"w", "'a", "\"a", "'x", "\"x", "'r+", "\"r+"]
        .iter()
        .any(|mode| mode_args.contains(mode))
}

fn first_quoted_arg(args: &str) -> Option<&str> {
    let trimmed = args.trim_start();
    let quote = trimmed
        .chars()
        .next()
        .filter(|ch| *ch == '\'' || *ch == '"')?;
    let body = &trimmed[quote.len_utf8()..];
    body.find(quote).map(|end| &body[..end])
}

fn collect_quoted_arg_after(command: &str, marker: &str, paths: &mut Vec<String>) {
    let mut rest = command;
    while let Some(index) = rest.find(marker) {
        let after_marker = &rest[index + marker.len()..];
        let trimmed = after_marker.trim_start();
        if let Some(quote) = trimmed
            .chars()
            .next()
            .filter(|ch| *ch == '\'' || *ch == '"')
        {
            let body = &trimmed[quote.len_utf8()..];
            if let Some(end) = body.find(quote) {
                push_candidate_path(paths, &body[..end]);
            }
        }
        rest = &after_marker[marker.len().min(after_marker.len())..];
    }
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

fn is_plan_bootstrap_write(path: &str) -> bool {
    path.starts_with(PLAN_BOOTSTRAP_PREFIX) && path.ends_with(".md")
}

#[cfg(test)]
pub(crate) fn target_allows(path: &str, targets: &[&str]) -> bool {
    targets
        .iter()
        .filter_map(|target| MutationScope::from_task_target(target).ok())
        .any(|scope| scope.allows(path))
}
