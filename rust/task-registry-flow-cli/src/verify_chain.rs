use std::fs;
use std::path::Path;

use fs2::FileExt;
use serde_json::Value;

use crate::metrics::receipt_value_hash;
use crate::model::{EVENTS_PATH, Result};
use crate::reports::{RuntimeFailure, RuntimeResult};
use crate::schema::{CheckReport, CheckStatus, Diagnostic, FailureCode, ReportSurface};

pub(crate) fn run_verify_chain(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let (repair, json) = parse_verify_chain_args(args)?;

    let report = check_chain(root)?;

    if repair && report.has_failures() {
        let fixed = repair_chain(root, &report)?;
        if json {
            return fixed
                .to_json()
                .map_err(|e| RuntimeFailure::text(FailureCode::Serialization, e.to_string()));
        }
        return Ok(format_chain_report(&fixed));
    }

    if report.has_failures() {
        if json {
            return Err(RuntimeFailure::json(
                FailureCode::DiagnosticReport,
                report
                    .to_json()
                    .map_err(|e| RuntimeFailure::text(FailureCode::Serialization, e.to_string()))?,
            ));
        }
        return Err(RuntimeFailure::text(
            FailureCode::DiagnosticReport,
            format_chain_report(&report),
        ));
    }

    if json {
        return report
            .to_json()
            .map_err(|e| RuntimeFailure::text(FailureCode::Serialization, e.to_string()));
    }
    Ok(format_chain_report(&report))
}

/// Parse verify-chain arguments strictly. Unknown args, trailing values,
/// or duplicate flags produce an error so the user gets a usage message
/// instead of silent success.
fn parse_verify_chain_args(args: &[String]) -> Result<(bool, bool)> {
    let mut repair = false;
    let mut json = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repair" => {
                if repair {
                    return Err("duplicate flag: --repair".to_string());
                }
                repair = true;
            }
            "--format" => {
                if json {
                    return Err("duplicate flag: --format".to_string());
                }
                let value = args.get(i + 1);
                match value.map(String::as_str) {
                    Some("json") => {
                        json = true;
                        i += 1; // consume the value
                    }
                    Some(invalid) => {
                        return Err(format!(
                            "--format requires a value (json), got: \"{invalid}\""
                        ));
                    }
                    None => {
                        return Err("--format requires a value (json), got: (nothing)".to_string());
                    }
                }
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }
    Ok((repair, json))
}

fn check_chain(root: &Path) -> Result<CheckReport> {
    let events_path = root.join(EVENTS_PATH);
    if !events_path.is_file() {
        return CheckReport::new(ReportSurface::ReceiptChain, vec![]);
    }
    let body = fs::read_to_string(&events_path)
        .map_err(|error| format!("read {}: {error}", events_path.display()))?;

    let mut diagnostics = Vec::new();
    let mut expected_previous_hash: Option<String> = None;

    for (line_number, line) in body.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let line_num = line_number + 1;

        let value: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                diagnostics.push(Diagnostic::fail(
                    "receipt-chain-parse",
                    ReportSurface::ReceiptChain,
                    events_path.display().to_string(),
                    "valid JSON event",
                    format!("line {line_num}: {e}"),
                    "restore events.jsonl from backup",
                ));
                expected_previous_hash = None;
                continue;
            }
        };
        if value.get("schema_version").and_then(|value| value.as_i64()) != Some(2) {
            diagnostics.push(Diagnostic::fail(
                "receipt-chain-schema",
                ReportSurface::ReceiptChain,
                events_path.display().to_string(),
                format!("line {line_num}: schema_version 2 receipt"),
                format!(
                    "schema_version {:?}",
                    value.get("schema_version").cloned().unwrap_or(Value::Null)
                ),
                "migrate legacy receipts to schema version 2 before verification",
            ));
            expected_previous_hash = None;
            continue;
        }

        let declared_event_hash = value
            .get("event_hash_sha256")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let declared_previous_hash = value
            .get("previous_event_hash_sha256")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if declared_event_hash.is_none() {
            diagnostics.push(Diagnostic::fail(
                "receipt-chain-unchained",
                ReportSurface::ReceiptChain,
                events_path.display().to_string(),
                "chained event_hash_sha256",
                format!("line {line_num}: unchained event"),
                "run task-registry-flow verify-chain --repair",
            ));
            if let Ok(hash) = receipt_value_hash(&value) {
                expected_previous_hash = Some(hash);
            }
            continue;
        }

        if declared_previous_hash != expected_previous_hash {
            diagnostics.push(Diagnostic::fail(
                "receipt-chain-break",
                ReportSurface::ReceiptChain,
                events_path.display().to_string(),
                format!(
                    "line {line_num}: previous_event_hash_sha256 expected {:?}, got {:?}",
                    expected_previous_hash, declared_previous_hash
                ),
                format!(
                    "expected {:?}, got {:?}",
                    expected_previous_hash, declared_previous_hash
                ),
                "run task-registry-flow verify-chain --repair",
            ));
        }

        let computed = receipt_value_hash(&value)?;
        if let Some(declared) = &declared_event_hash
            && &computed != declared
        {
            diagnostics.push(Diagnostic::fail(
                "receipt-chain-tamper",
                ReportSurface::ReceiptChain,
                events_path.display().to_string(),
                format!("line {line_num}: event_hash_sha256 mismatch"),
                format!("declared {declared}, computed {computed}"),
                "run task-registry-flow verify-chain --repair",
            ));
        }

        expected_previous_hash = Some(computed);
    }

    CheckReport::new(ReportSurface::ReceiptChain, diagnostics)
}

/// Repair the receipt chain by recomputing every event hash from scratch.
///
/// `_broken` is accepted for API consistency with the caller but is not used
/// for targeted repair: a single chain break cascades through all subsequent
/// `previous_event_hash_sha256` values, so every line must be re-chained.
/// The caller (`run_verify_chain`) already guards with `report.has_failures()`
/// so this function is only called when repair is actually needed.
fn repair_chain(root: &Path, _broken: &CheckReport) -> Result<CheckReport> {
    let events_path = root.join(EVENTS_PATH);
    let body = fs::read_to_string(&events_path)
        .map_err(|error| format!("read {}: {error}", events_path.display()))?;

    let mut repaired_lines = Vec::new();
    let mut expected_previous_hash: Option<String> = None;
    let mut fixed_diagnostics = Vec::new();

    for (line_number, line) in body.lines().enumerate() {
        if line.trim().is_empty() {
            repaired_lines.push(line.to_string());
            continue;
        }
        let line_num = line_number + 1;

        let mut value: Value =
            serde_json::from_str(line).map_err(|e| format!("parse line {line_num}: {e}"))?;
        if value.get("schema_version").and_then(|value| value.as_i64()) != Some(2) {
            return Err(format!(
                "cannot repair line {line_num}: expected schema_version 2 receipt"
            ));
        }

        value["previous_event_hash_sha256"] = match &expected_previous_hash {
            Some(prev) => Value::String(prev.clone()),
            None => Value::Null,
        };

        value["event_hash_sha256"] = Value::Null;

        let computed_hash = receipt_value_hash(&value)?;
        value["event_hash_sha256"] = Value::String(computed_hash.clone());

        let original_line = line.to_string();
        let new_line =
            serde_json::to_string(&value).map_err(|e| format!("serialize line {line_num}: {e}"))?;

        if original_line != new_line {
            fixed_diagnostics.push(Diagnostic::pass(
                "receipt-chain-repaired",
                ReportSurface::ReceiptChain,
                events_path.display().to_string(),
                format!("line {line_num} hash chain repaired"),
            ));
        }

        repaired_lines.push(new_line);
        expected_previous_hash = Some(computed_hash);
    }

    let new_body = repaired_lines.join("\n") + "\n";
    // Open for lock only — do not truncate. The atomic rename below
    // replaces the file; truncating before the lock, write, and sync
    // would lose data if any of those steps fail.
    let file = fs::OpenOptions::new()
        .write(true)
        .open(&events_path)
        .map_err(|error| format!("open {}: {error}", events_path.display()))?;
    file.try_lock_exclusive()
        .map_err(|_| "events file is locked by another process; retry in a moment".to_string())?;
    let tmp_path = events_path.with_extension("jsonl.tmp");
    fs::write(&tmp_path, &new_body)
        .map_err(|error| format!("write {}: {error}", tmp_path.display()))?;
    // fsync temp file before atomic rename — prevents zero-length or partial
    // events file after an OS crash or power loss.
    if let Err(error) = std::fs::OpenOptions::new()
        .read(true)
        .open(&tmp_path)
        .and_then(|f| f.sync_all())
    {
        let _ = fs::remove_file(&tmp_path);
        return Err(format!("sync {}: {error}", tmp_path.display()));
    }
    fs::rename(&tmp_path, &events_path).map_err(|error| {
        format!(
            "rename {} -> {}: {error}",
            tmp_path.display(),
            events_path.display()
        )
    })?;

    CheckReport::new(ReportSurface::ReceiptChainFix, fixed_diagnostics)
}

fn format_chain_report(report: &CheckReport) -> String {
    if !report.has_failures() && report.summary.warn == 0 && report.summary.pass == 0 {
        return "receipt chain is intact".to_string();
    }
    let mut lines = Vec::new();
    if report.has_failures() {
        lines.push(format!(
            "receipt chain has {} break(s)",
            report.summary.fail
        ));
        for check in &report.checks {
            if check.status == CheckStatus::Fail {
                lines.push(format!("  fail: {} — {}", check.check_id, check.actual));
            }
        }
    } else {
        lines.push("receipt chain is intact".to_string());
    }
    if report.summary.warn > 0 {
        lines.push(format!("{} warning(s):", report.summary.warn));
        for check in &report.checks {
            if check.status == CheckStatus::Warn {
                lines.push(format!("  warn: {} — {}", check.check_id, check.actual));
            }
        }
    }
    if report.summary.pass > 0 {
        lines.push(format!("{} repair(s):", report.summary.pass));
        for check in &report.checks {
            if check.status == CheckStatus::Pass {
                lines.push(format!("  pass: {} — {}", check.check_id, check.actual));
            }
        }
    }
    lines.join("\n")
}
