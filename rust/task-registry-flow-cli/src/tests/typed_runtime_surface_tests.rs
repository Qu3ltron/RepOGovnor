use crate::cli::failure_json_for_test;
use crate::model::EventRecord;
use crate::schema::{
    CheckReport, CliCommand, CommandReport, Diagnostic, EventOutcome, FailureCode, HookFormat,
    ReportSurface, RuntimeSubject, RuntimeSubjectKind,
};
use std::str::FromStr;

#[test]
fn typed_command_enum_round_trips() {
    for command in CliCommand::variants() {
        let parsed = CliCommand::from_str(command).expect("known command parses");
        let encoded = serde_json::to_string(&parsed).expect("command serializes");
        assert_eq!(encoded, format!("\"{command}\""));
        let decoded: CliCommand = serde_json::from_str(&encoded).expect("command deserializes");
        assert_eq!(decoded, parsed);
    }
}

#[test]
fn typed_command_enum_rejects_unknown_values() {
    assert!(CliCommand::from_str("legacy-command").is_err());
    assert!(serde_json::from_str::<CliCommand>("\"legacy-command\"").is_err());
    assert!(crate::runtime::run(vec!["legacy-command".to_string()]).is_err());
}

#[test]
fn typed_hook_format_enum_round_trips() {
    for format in HookFormat::variants() {
        let parsed = HookFormat::from_str(format).expect("known hook format parses");
        let encoded = serde_json::to_string(&parsed).expect("hook format serializes");
        assert_eq!(encoded, format!("\"{format}\""));
        let decoded: HookFormat = serde_json::from_str(&encoded).expect("hook format deserializes");
        assert_eq!(decoded, parsed);
    }
}

#[test]
fn typed_hook_format_rejects_unknown_values() {
    assert!(HookFormat::from_str("legacy").is_err());
    assert!(serde_json::from_str::<HookFormat>("\"legacy\"").is_err());
}

#[test]
fn typed_receipt_event_schema_round_trips() {
    let mut event = EventRecord::new(
        "2026-06-01T00:00:00-04:00".to_string(),
        CliCommand::Validate,
        EventOutcome::Ok,
        7,
        "validated".to_string(),
    );
    event.subject = RuntimeSubject::path(RuntimeSubjectKind::VerifierTarget, "README.md");
    event.diagnostics.push(Diagnostic::pass(
        "source-limit",
        ReportSurface::SourceLimit,
        ".",
        "all files within limit",
    ));

    let encoded = serde_json::to_value(&event).expect("event serializes");
    assert_eq!(encoded["schema_version"], 2);
    assert_eq!(encoded["command"], "validate");
    assert_eq!(encoded["outcome"], "ok");
    assert_eq!(encoded["subject"]["kind"], "verifier-target");
    assert_eq!(encoded["diagnostics"][0]["surface"], "source-limit");

    let decoded: EventRecord = serde_json::from_value(encoded).expect("typed event deserializes");
    assert_eq!(decoded.command, CliCommand::Validate);
    assert_eq!(decoded.subject.kind, RuntimeSubjectKind::VerifierTarget);
    assert_eq!(decoded.diagnostics[0].surface, ReportSurface::SourceLimit);
}

#[test]
fn typed_receipt_event_schema_rejects_unknown_subject_kind() {
    let event = serde_json::json!({
        "schema_version": 2,
        "timestamp": "2026-06-01T00:00:00-04:00",
        "command": "validate",
        "outcome": "ok",
        "duration_ms": 7,
        "subject": {"kind": "unknown", "id": "x", "path": "x"},
        "summary": "validated"
    });
    assert!(serde_json::from_value::<EventRecord>(event).is_err());

    let report = serde_json::json!({
        "schema_version": 1,
        "surface": "unknown",
        "summary": {"pass": 0, "warn": 0, "fail": 0, "skip": 0},
        "checks": []
    });
    assert!(serde_json::from_value::<CheckReport>(report).is_err());
}

#[test]
fn typed_failure_code_emits_in_json_report() {
    let output = failure_json_for_test(CliCommand::Validate, false, "bad input");
    let value: serde_json::Value = serde_json::from_str(&output).expect("failure report JSON");
    assert_eq!(value["schema_version"], 2);
    assert_eq!(value["command"], "validate");
    assert_eq!(value["status"], "fail");
    assert_eq!(value["failure_code"], "runtime");

    let report: CommandReport = serde_json::from_value(value).expect("typed command report");
    assert_eq!(report.failure_code, Some(FailureCode::Runtime));
}

#[test]
fn typed_failure_code_rejects_unknown_values() {
    let report = serde_json::json!({
        "schema_version": 2,
        "command": "validate",
        "status": "fail",
        "summary": "bad input",
        "failure_code": "unknown",
        "receipt_recorded": false
    });
    assert!(serde_json::from_value::<CommandReport>(report).is_err());
}
