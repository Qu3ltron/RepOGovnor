use crate::model::Result;
use crate::reports::{RuntimeFailure, RuntimeResult};
use crate::schema::{CheckReport, Diagnostic, FailureCode, ReportSurface};
use std::fs;
use std::path::Path;

const GAP_PIPELINE_PATH: &str = "docs/gap-pipeline.md";
const REQUIRED_TOP_LEVEL: &[&str] = &[
    "## Current Evidence",
    "## Remaining Gaps",
    "## Negative Non-Claims",
    "## Drain protocol",
];
const REQUIRED_GAP_FIELDS: &[&str] = &[
    "- Claim pressure:",
    "- Current evidence:",
    "- User impact:",
    "- Next closure:",
    "- Reactivation condition:",
];
const REQUIRED_NON_CLAIMS: &[&str] = &[
    "No automatic final release publication",
    "No automatic final release tag push",
    "No product correctness proof",
    "No hosted fleet governance",
    "No remote receipt sync",
];
const FORBIDDEN_CLAIMS: &[&str] = &[
    "there are no remaining gaps",
    "governance proves product correctness",
    "governance checks replace code review",
    "automatic final release publication is supported",
    "automatic final release tag push is supported",
    "hosted fleet governance is provided",
    "remote receipt sync is provided",
];

pub(crate) fn run_command(root: &Path, args: &[String]) -> RuntimeResult<String> {
    let json = match args {
        [] => false,
        [flag, format] if flag == "--format" && format == "json" => true,
        _ => return Err("usage: task-registry-flow backlog-check [--format json]".into()),
    };
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
        Ok("backlog-check ok".to_string())
    }
}

pub(crate) fn report(root: &Path) -> Result<CheckReport> {
    let body = fs::read_to_string(root.join(GAP_PIPELINE_PATH))
        .map_err(|error| format!("read {GAP_PIPELINE_PATH}: {error}"))?;
    report_from_body(&body)
}

pub(crate) fn report_from_body(body: &str) -> Result<CheckReport> {
    let mut checks = Vec::new();
    checks.extend(required_top_level_checks(body));
    checks.extend(gap_field_checks(body));
    checks.extend(non_claim_checks(body));
    checks.extend(forbidden_claim_checks(body));
    CheckReport::new(ReportSurface::Backlog, checks)
}

fn required_top_level_checks(body: &str) -> Vec<Diagnostic> {
    REQUIRED_TOP_LEVEL
        .iter()
        .map(|heading| contains_check("backlog-section", *heading, body.contains(heading)))
        .collect()
}

fn gap_field_checks(body: &str) -> Vec<Diagnostic> {
    gap_sections(body)
        .into_iter()
        .flat_map(|(gap_id, section)| {
            REQUIRED_GAP_FIELDS.iter().map(move |field| {
                contains_check(
                    "backlog-gap-field",
                    format!("{gap_id} {field}"),
                    section.contains(field),
                )
            })
        })
        .collect()
}

fn non_claim_checks(body: &str) -> Vec<Diagnostic> {
    REQUIRED_NON_CLAIMS
        .iter()
        .map(|claim| contains_check("backlog-negative-nonclaim", *claim, body.contains(claim)))
        .collect()
}

fn forbidden_claim_checks(body: &str) -> Vec<Diagnostic> {
    let lower = body.to_ascii_lowercase();
    FORBIDDEN_CLAIMS
        .iter()
        .map(|claim| {
            let found = lower.contains(&claim.to_ascii_lowercase());
            if found {
                Diagnostic::fail(
                    "backlog-forbidden-claim",
                    ReportSurface::Backlog,
                    GAP_PIPELINE_PATH,
                    format!("absence of '{claim}'"),
                    "present",
                    "remove unsupported product, fleet, or final-release automation claim",
                )
            } else {
                Diagnostic::pass(
                    "backlog-forbidden-claim",
                    ReportSurface::Backlog,
                    GAP_PIPELINE_PATH,
                    format!("absence of '{claim}'"),
                )
            }
        })
        .collect()
}

fn contains_check(check_id: &str, expected: impl Into<String>, pass: bool) -> Diagnostic {
    let expected = expected.into();
    if pass {
        Diagnostic::pass(
            check_id,
            ReportSurface::Backlog,
            GAP_PIPELINE_PATH,
            expected,
        )
    } else {
        Diagnostic::fail(
            check_id,
            ReportSurface::Backlog,
            GAP_PIPELINE_PATH,
            expected,
            "missing",
            "update docs/gap-pipeline.md with the required backlog field",
        )
    }
}

fn gap_sections(body: &str) -> Vec<(String, String)> {
    let mut sections = Vec::new();
    let mut current_id: Option<String> = None;
    let mut current_body = String::new();
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("### GP-") {
            if let Some(gap_id) = current_id.take() {
                sections.push((gap_id, current_body.clone()));
                current_body.clear();
            }
            let id = rest
                .split(':')
                .next()
                .map(|suffix| format!("GP-{suffix}"))
                .unwrap_or_else(|| "GP-UNKNOWN".to_string());
            current_id = Some(id);
        }
        if current_id.is_some() {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }
    if let Some(gap_id) = current_id {
        sections.push((gap_id, current_body));
    }
    sections
}

fn format_human(report: &CheckReport) -> String {
    let mut lines = report
        .checks
        .iter()
        .map(|check| {
            format!(
                "{} {}: expected {}, actual {}",
                check.status, check.check_id, check.expected, check.actual
            )
        })
        .collect::<Vec<_>>();
    lines.push(format!(
        "backlog check summary: {} pass, {} warn, {} fail, {} skip",
        report.summary.pass, report.summary.warn, report.summary.fail, report.summary.skip
    ));
    lines.join("\n")
}
