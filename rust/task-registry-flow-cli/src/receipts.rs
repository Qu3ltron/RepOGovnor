use crate::model::{EventRecord, Result};
use crate::runtime;
use crate::schema::{CliCommand, EventOutcome};
use std::path::Path;

pub(crate) fn should_record(command: CliCommand, explicit: bool) -> bool {
    explicit
        || matches!(
            command,
            CliCommand::Activate
                | CliCommand::Status
                | CliCommand::Defer
                | CliCommand::VerifyLanding
                | CliCommand::ArchiveCompleted
        )
}

pub(crate) fn append_command_event(
    root: &Path,
    command: CliCommand,
    outcome: EventOutcome,
    duration_ms: u128,
    summary: &str,
) -> Result<()> {
    runtime::append_event(
        root,
        EventRecord::new(
            runtime::timestamp(),
            command,
            outcome,
            duration_ms,
            runtime::truncate_detail(summary),
        ),
    )
}
