use crate::schema::{CliCommand, EventOutcome};
use crate::{receipts, reports, runtime};
use reports::RuntimeFailure;
use std::env;
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Copy)]
struct CliOptions {
    output_format: OutputFormat,
    record_receipt: bool,
}

pub(crate) fn main_entry() -> i32 {
    let started = Instant::now();
    let raw_args = env::args().skip(1).collect::<Vec<_>>();
    let (options, args) = match parse_global_options(raw_args) {
        Ok(parsed) => parsed,
        Err(error) => {
            render_error(
                OutputFormat::Text,
                CliCommand::Usage,
                false,
                &RuntimeFailure::from(error),
            );
            return 1;
        }
    };
    let command = args
        .first()
        .and_then(|value| CliCommand::from_str(value).ok())
        .unwrap_or(CliCommand::Usage);
    let mut result = runtime::run(args);
    let duration_ms = started.elapsed().as_millis();
    let outcome = if result.is_ok() {
        EventOutcome::Ok
    } else {
        EventOutcome::Error
    };

    let mut receipt_recorded = false;
    if receipts::should_record(command, options.record_receipt) {
        match record_command_receipt(Path::new("."), command, outcome, duration_ms, &result) {
            Ok(()) => receipt_recorded = true,
            Err(error) => result = Err(error),
        }
    }

    match result {
        Ok(detail) => {
            if options.output_format == OutputFormat::Json {
                println!(
                    "{}",
                    reports::success_json(command, detail, receipt_recorded)
                        .unwrap_or_else(|error| {
                            format!(
                                r#"{{"status":"error","command":"{}","error":"serialization failed: {}"}}"#,
                                command.as_str(),
                                error.replace('"', "\\\"")
                            )
                        })
                );
            } else if !detail.is_empty() {
                println!("{detail}");
            }
            0
        }
        Err(error) => {
            render_error(options.output_format, command, receipt_recorded, &error);
            1
        }
    }
}

pub(crate) fn record_command_receipt(
    root: &Path,
    command: CliCommand,
    outcome: EventOutcome,
    duration_ms: u128,
    result: &reports::RuntimeResult<String>,
) -> Result<(), RuntimeFailure> {
    let summary = result
        .as_ref()
        .map_or_else(|error| error.summary().to_string(), Clone::clone);
    receipts::append_command_event(root, command, outcome, duration_ms, &summary)
        .map_err(|error| RuntimeFailure::Text(format!("required receipt append failed: {error}")))
}

fn parse_global_options(args: Vec<String>) -> Result<(CliOptions, Vec<String>), String> {
    let mut output_format = OutputFormat::Text;
    let mut record_receipt = false;
    let mut parsed = Vec::new();
    let mut iter = args.into_iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--format" if parsed.is_empty() => {
                output_format = match iter.next().as_deref() {
                    Some("text") => OutputFormat::Text,
                    Some("json") => OutputFormat::Json,
                    _ => return Err("unknown output format; expected text or json".to_string()),
                };
            }
            "--record-receipt" if parsed.is_empty() => record_receipt = true,
            value if value.starts_with("--") && parsed.is_empty() => {
                return Err(format!("unknown global option: {value}"));
            }
            _ => {
                parsed.push(arg);
                parsed.extend(iter);
                break;
            }
        }
    }
    Ok((
        CliOptions {
            output_format,
            record_receipt,
        },
        parsed,
    ))
}

fn render_error(
    output_format: OutputFormat,
    command: CliCommand,
    receipt_recorded: bool,
    error: &RuntimeFailure,
) {
    match (output_format, error) {
        (_, RuntimeFailure::Json(value)) => println!("{value}"),
        (OutputFormat::Json, RuntimeFailure::Text(value)) => {
            println!(
                "{}",
                reports::failure_json(command, value, receipt_recorded).unwrap_or_else(|error| {
                    format!(
                        r#"{{"status":"error","command":"{}","error":"serialization failed: {}"}}"#,
                        command.as_str(),
                        error.replace('"', "\\\"")
                    )
                })
            );
        }
        (OutputFormat::Text, RuntimeFailure::Text(value)) => {
            eprintln!("task-registry-flow error: {value}");
        }
    }
}

#[cfg(test)]
pub(crate) fn failure_json_for_test(
    command: CliCommand,
    receipt_recorded: bool,
    error: &str,
) -> String {
    reports::failure_json(command, error, receipt_recorded).unwrap_or_else(|e| {
        format!(
            r#"{{"status":"error","command":"{}","error":"serialization failed: {}"}}"#,
            command.as_str(),
            e.replace('"', "\\\"")
        )
    })
}
