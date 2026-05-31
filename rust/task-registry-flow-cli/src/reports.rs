use crate::model::Result;
use crate::schema::{CliCommand, CommandReport};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeFailure {
    Text(String),
    Json(String),
}

impl RuntimeFailure {
    pub(crate) fn json(value: impl Into<String>) -> Self {
        Self::Json(value.into())
    }

    pub(crate) fn summary(&self) -> &str {
        match self {
            Self::Text(value) | Self::Json(value) => value,
        }
    }
}

impl From<String> for RuntimeFailure {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for RuntimeFailure {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl fmt::Display for RuntimeFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.summary())
    }
}

pub(crate) type RuntimeResult<T = String> = std::result::Result<T, RuntimeFailure>;

pub(crate) fn success_json(
    command: CliCommand,
    summary: impl Into<String>,
    receipt_recorded: bool,
) -> Result<String> {
    CommandReport::pass(command, summary, receipt_recorded).to_json()
}

pub(crate) fn failure_json(
    command: CliCommand,
    error: impl Into<String>,
    receipt_recorded: bool,
) -> Result<String> {
    CommandReport::fail(command, error, receipt_recorded).to_json()
}
