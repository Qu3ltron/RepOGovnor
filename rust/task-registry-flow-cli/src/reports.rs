use crate::model::Result;
use crate::schema::{CliCommand, CommandReport, FailureCode};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeFailure {
    Text { code: FailureCode, message: String },
    Json { code: FailureCode, payload: String },
}

impl RuntimeFailure {
    pub(crate) fn text(code: FailureCode, message: impl Into<String>) -> Self {
        Self::Text {
            code,
            message: message.into(),
        }
    }

    pub(crate) fn json(code: FailureCode, payload: impl Into<String>) -> Self {
        Self::Json {
            code,
            payload: payload.into(),
        }
    }

    pub(crate) fn code(&self) -> FailureCode {
        match self {
            Self::Text { code, .. } | Self::Json { code, .. } => *code,
        }
    }

    pub(crate) fn summary(&self) -> &str {
        match self {
            Self::Text { message, .. } => message,
            Self::Json { payload, .. } => payload,
        }
    }
}

impl From<String> for RuntimeFailure {
    fn from(value: String) -> Self {
        Self::text(FailureCode::Runtime, value)
    }
}

impl From<&str> for RuntimeFailure {
    fn from(value: &str) -> Self {
        Self::text(FailureCode::Runtime, value)
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
    failure_code: FailureCode,
    error: impl Into<String>,
    receipt_recorded: bool,
) -> Result<String> {
    CommandReport::fail(command, failure_code, error, receipt_recorded).to_json()
}
