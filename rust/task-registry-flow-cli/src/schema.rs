use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[allow(dead_code)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        pub(crate) enum $name {
            $(
                #[serde(rename = $value)]
                $variant
            ),+
        }

        #[allow(dead_code)]
        impl $name {
            pub(crate) fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }

            pub(crate) fn variants() -> &'static [&'static str] {
                &[$($value),+]
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = String;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(format!(
                        "unknown {}: {} (expected one of: {})",
                        stringify!($name),
                        value,
                        Self::variants().join(", ")
                    )),
                }
            }
        }
    };
}

string_enum!(CliCommand {
    Validate => "validate",
    Activate => "activate",
    Status => "status",
    Defer => "defer",
    Report => "report",
    ReviewerReport => "reviewer-report",
    VersionCheck => "version-check",
    BacklogCheck => "backlog-check",
    ArchiveCompleted => "archive-completed",
    VerifyBehaviors => "verify-behaviors",
    VerifyLanding => "verify-landing",
    VerifyChain => "verify-chain",
    VerifyMutationHook => "verify-mutation-hook",
    Metrics => "metrics",
    SourceLimit => "source-limit",
    ReleaseCheck => "release-check",
    Install => "install",
    StatusCheck => "status-check",
    Usage => "usage",
});

string_enum!(HookFormat {
    Antigravity => "antigravity",
    Codex => "codex",
    Cursor => "cursor",
    Claude => "claude",
});

string_enum!(RuntimeSubjectKind {
    Command => "command",
    MutationTarget => "mutation-target",
    VerifierTarget => "verifier-target",
});

string_enum!(ReportSurface {
    Cli => "cli",
    Manifest => "manifest",
    Migration => "migration",
    ReleaseSource => "release-source",
    TrackedForCi => "tracked-for-ci",
    SourceLimit => "source-limit",
    SourceLimitPlan => "source-limit-plan",
    Status => "status",
    Version => "version",
    Backlog => "backlog",
    ReceiptChain => "receipt-chain",
    ReceiptChainFix => "receipt-chain-fix",
});

string_enum!(FailureCode {
    Usage => "usage",
    Runtime => "runtime",
    Serialization => "serialization",
    ReceiptAppend => "receipt-append",
    DiagnosticReport => "diagnostic-report",
});

string_enum!(TaskStatus {
    Planned => "planned",
    Active => "active",
    Blocked => "blocked",
    Deferred => "deferred",
    Completed => "completed",
    Cancelled => "cancelled",
});

string_enum!(TaskKind {
    Authorization => "authorization",
    Diagnostics => "diagnostics",
    Documentation => "documentation",
    Governance => "governance",
    Implementation => "implementation",
    Migration => "migration",
    Release => "release",
    Schema => "schema",
    Test => "test",
    Validation => "validation",
});

string_enum!(EventOutcome {
    Ok => "ok",
    Error => "error",
    MutationDenied => "mutation-denied",
});

string_enum!(DiagnosticSeverity {
    Info => "info",
    Warning => "warning",
    Error => "error",
});

string_enum!(CheckStatus {
    Pass => "pass",
    Warn => "warn",
    Fail => "fail",
    Skip => "skip",
});

string_enum!(CommandStatus {
    Pass => "pass",
    Fail => "fail",
});

string_enum!(VerifierType {
    Command => "command",
    FileExists => "file_exists",
    FileAbsent => "file_absent",
    Contains => "contains",
    NotContains => "not_contains",
    JsonValid => "json_valid",
    JsonSchema => "json_schema",
});

string_enum!(BehaviorPolarity {
    Positive => "positive",
    Negative => "negative",
    Validation => "validation",
});

string_enum!(MutationScopeKind {
    ExactFile => "exact_file",
    DirectoryTree => "directory_tree",
    GeneratedArtifact => "generated_artifact",
    GovernanceRepair => "governance_repair",
});

string_enum!(InstallAction {
    Aligned => "aligned",
    Create => "create",
    Update => "update",
    Replace => "replace",
    MergeCreate => "merge-create",
    MergeAppend => "merge-append",
    MergeUpdate => "merge-update",
    Preserve => "preserve",
    PreserveValid => "preserve-valid",
    PreserveDrift => "preserve-drift",
    SyncSkill => "sync-skill",
    ReplaceSymlink => "replace-symlink",
    RemoveStale => "remove-stale",
    CreateDir => "create-dir",
    ReplaceDir => "replace-dir",
    CreateSymlink => "create-symlink",
    Chmod => "chmod",
    Skip => "skip",
    LinkPlugin => "link-plugin",
});

string_enum!(ReleaseCheckId {
    ReleaseFilePresent => "release-file-present",
    ReleaseFileExecutable => "release-file-executable",
    ReleaseScriptExecutableUndeclared => "release-script-executable-undeclared",
    ReleaseExecutablePlatform => "release-executable-platform",
    ReleaseRustSourceUndeclared => "release-rust-source-undeclared",
    ReleaseGovernedSourceUndeclared => "release-governed-source-undeclared",
    StalePathAbsent => "stale-path-absent",
    ReleaseVersionConsistent => "release-version-consistent",
    TrackedForCiPresent => "tracked-for-ci-present",
    ReleaseSchemaValid => "release-schema-valid",
});

string_enum!(VersionFileFormat {
    Plain => "plain",
    Json => "json",
    Toml => "toml",
    MarkdownLine => "markdown-line",
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum SchemaVersion {
    V1,
    V2,
}

impl SchemaVersion {
    pub(crate) fn as_i64(self) -> i64 {
        match self {
            Self::V1 => 1,
            Self::V2 => 2,
        }
    }
}

impl Serialize for SchemaVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.as_i64())
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SchemaVersionVisitor;

        impl Visitor<'_> for SchemaVersionVisitor {
            type Value = SchemaVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("schema version 1 or 2")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    1 => Ok(SchemaVersion::V1),
                    2 => Ok(SchemaVersion::V2),
                    _ => Err(E::custom(format!("unsupported schema_version {value}"))),
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    1 => Ok(SchemaVersion::V1),
                    2 => Ok(SchemaVersion::V2),
                    _ => Err(E::custom(format!("unsupported schema_version {value}"))),
                }
            }
        }

        deserializer.deserialize_any(SchemaVersionVisitor)
    }
}

fn deserialize_schema_version_v2<'de, D>(deserializer: D) -> Result<SchemaVersion, D::Error>
where
    D: Deserializer<'de>,
{
    let version = SchemaVersion::deserialize(deserializer)?;
    if version == SchemaVersion::V2 {
        Ok(version)
    } else {
        Err(<D::Error as de::Error>::custom(
            "runtime schema_version must be 2",
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReceiptEvent {
    #[serde(deserialize_with = "deserialize_schema_version_v2")]
    pub(crate) schema_version: SchemaVersion,
    pub(crate) timestamp: String,
    pub(crate) command: CliCommand,
    pub(crate) outcome: EventOutcome,
    pub(crate) duration_ms: u128,
    pub(crate) subject: RuntimeSubject,
    pub(crate) summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) previous_event_hash_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) event_hash_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) diagnostics: Vec<Diagnostic>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) verifier_results: Vec<VerifierResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) mutation_denial: Option<MutationDenial>,
}

impl ReceiptEvent {
    pub(crate) fn new(
        timestamp: String,
        command: CliCommand,
        outcome: EventOutcome,
        duration_ms: u128,
        summary: String,
    ) -> Self {
        Self {
            schema_version: SchemaVersion::V2,
            timestamp,
            command,
            outcome,
            duration_ms,
            subject: RuntimeSubject::command(command),
            summary,
            previous_event_hash_sha256: None,
            event_hash_sha256: None,
            diagnostics: Vec::new(),
            verifier_results: Vec::new(),
            mutation_denial: None,
        }
    }

    pub(crate) fn mutation_denied(
        timestamp: String,
        duration_ms: u128,
        path: String,
        reason: String,
    ) -> Self {
        Self {
            schema_version: SchemaVersion::V2,
            timestamp,
            command: CliCommand::VerifyMutationHook,
            outcome: EventOutcome::MutationDenied,
            duration_ms,
            subject: RuntimeSubject::path(RuntimeSubjectKind::MutationTarget, path.clone()),
            summary: reason.clone(),
            previous_event_hash_sha256: None,
            event_hash_sha256: None,
            diagnostics: Vec::new(),
            verifier_results: Vec::new(),
            mutation_denial: Some(MutationDenial { path, reason }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeSubject {
    pub(crate) kind: RuntimeSubjectKind,
    pub(crate) id: String,
    pub(crate) path: String,
}

impl RuntimeSubject {
    pub(crate) fn command(command: CliCommand) -> Self {
        Self {
            kind: RuntimeSubjectKind::Command,
            id: command.as_str().to_string(),
            path: ".".to_string(),
        }
    }

    pub(crate) fn path(kind: RuntimeSubjectKind, path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            kind,
            id: path.clone(),
            path,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VerifierResult {
    pub(crate) behavior_id: String,
    pub(crate) verifier_type: VerifierType,
    pub(crate) status: CheckStatus,
    pub(crate) subject: RuntimeSubject,
    pub(crate) expected: String,
    pub(crate) actual: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MutationDenial {
    pub(crate) path: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CommandReport {
    #[serde(deserialize_with = "deserialize_schema_version_v2")]
    pub(crate) schema_version: SchemaVersion,
    pub(crate) command: CliCommand,
    pub(crate) status: CommandStatus,
    pub(crate) summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) failure_code: Option<FailureCode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) receipt_recorded: bool,
}

impl CommandReport {
    pub(crate) fn pass(
        command: CliCommand,
        summary: impl Into<String>,
        receipt_recorded: bool,
    ) -> Self {
        Self {
            schema_version: SchemaVersion::V2,
            command,
            status: CommandStatus::Pass,
            summary: summary.into(),
            failure_code: None,
            diagnostics: Vec::new(),
            receipt_recorded,
        }
    }

    pub(crate) fn fail(
        command: CliCommand,
        failure_code: FailureCode,
        actual: impl Into<String>,
        receipt_recorded: bool,
    ) -> Self {
        let actual = actual.into();
        Self {
            schema_version: SchemaVersion::V2,
            command,
            status: CommandStatus::Fail,
            summary: actual.clone(),
            failure_code: Some(failure_code),
            diagnostics: vec![Diagnostic::fail(
                "cli-usage",
                ReportSurface::Cli,
                ".",
                "valid command",
                actual,
                "rerun with a supported command and arguments",
            )],
            receipt_recorded,
        }
    }

    pub(crate) fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|error| format!("serialize command report: {error}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstallActionReport {
    pub(crate) path: String,
    pub(crate) action: InstallAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Diagnostic {
    pub(crate) check_id: String,
    pub(crate) surface: ReportSurface,
    pub(crate) path: String,
    pub(crate) severity: DiagnosticSeverity,
    pub(crate) status: CheckStatus,
    pub(crate) expected: String,
    pub(crate) actual: String,
    pub(crate) remediation: String,
}

impl Diagnostic {
    pub(crate) fn pass(
        check_id: impl Into<String>,
        surface: ReportSurface,
        path: impl Into<String>,
        expected: impl Into<String>,
    ) -> Self {
        let expected = expected.into();
        Self {
            check_id: check_id.into(),
            surface,
            path: path.into(),
            severity: DiagnosticSeverity::Info,
            status: CheckStatus::Pass,
            expected: expected.clone(),
            actual: expected,
            remediation: "none".to_string(),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn warn(
        check_id: impl Into<String>,
        surface: ReportSurface,
        path: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        let actual = actual.into();
        Self {
            check_id: check_id.into(),
            surface,
            path: path.into(),
            severity: DiagnosticSeverity::Warning,
            status: CheckStatus::Warn,
            expected: String::new(),
            actual,
            remediation: String::new(),
        }
    }

    pub(crate) fn fail(
        check_id: impl Into<String>,
        surface: ReportSurface,
        path: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Self {
        Self {
            check_id: check_id.into(),
            surface,
            path: path.into(),
            severity: DiagnosticSeverity::Error,
            status: CheckStatus::Fail,
            expected: expected.into(),
            actual: actual.into(),
            remediation: remediation.into(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        // Warnings may have empty expected/remediation — the actual state IS the signal.
        let skip_expected = self.severity == DiagnosticSeverity::Warning;
        for (field, value) in [
            ("check_id", self.check_id.as_str()),
            ("surface", self.surface.as_str()),
            ("path", self.path.as_str()),
            ("expected", self.expected.as_str()),
            ("actual", self.actual.as_str()),
            ("remediation", self.remediation.as_str()),
        ] {
            if (field == "expected" || field == "remediation") && skip_expected {
                continue;
            }
            if value.trim().is_empty() {
                return Err(format!("diagnostic {field} must not be empty"));
            }
        }
        if self.status == CheckStatus::Fail && self.severity != DiagnosticSeverity::Error {
            return Err("failing diagnostic severity must be error".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CheckSummary {
    pub(crate) pass: usize,
    pub(crate) warn: usize,
    pub(crate) fail: usize,
    pub(crate) skip: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CheckReport {
    pub(crate) schema_version: SchemaVersion,
    pub(crate) surface: ReportSurface,
    pub(crate) summary: CheckSummary,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) failure_code: Option<FailureCode>,
    pub(crate) checks: Vec<Diagnostic>,
}

impl CheckReport {
    pub(crate) fn new(surface: ReportSurface, checks: Vec<Diagnostic>) -> Result<Self, String> {
        let mut summary = CheckSummary {
            pass: 0,
            warn: 0,
            fail: 0,
            skip: 0,
        };
        for check in &checks {
            check.validate()?;
            match check.status {
                CheckStatus::Pass => summary.pass += 1,
                CheckStatus::Warn => summary.warn += 1,
                CheckStatus::Fail => summary.fail += 1,
                CheckStatus::Skip => summary.skip += 1,
            }
        }
        let failure_code = (summary.fail > 0).then_some(FailureCode::DiagnosticReport);
        Ok(Self {
            schema_version: SchemaVersion::V1,
            surface,
            summary,
            failure_code,
            checks,
        })
    }

    pub(crate) fn has_failures(&self) -> bool {
        self.summary.fail > 0
    }

    pub(crate) fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|error| format!("serialize report: {error}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub(crate) enum BehaviorVerifier {
    #[serde(rename = "command")]
    Command {
        command: String,
        #[serde(default = "default_expected_exit")]
        expected_exit: i32,
    },
    #[serde(rename = "file_exists")]
    FileExists { path: String },
    #[serde(rename = "file_absent")]
    FileAbsent { path: String },
    #[serde(rename = "contains")]
    Contains { path: String, needle: String },
    #[serde(rename = "not_contains")]
    NotContains { path: String, needle: String },
    #[serde(rename = "json_valid")]
    JsonValid { path: String },
    #[serde(rename = "json_schema")]
    JsonSchema { path: String, schema_path: String },
}

impl BehaviorVerifier {
    pub(crate) fn verifier_type(&self) -> VerifierType {
        match self {
            Self::Command { .. } => VerifierType::Command,
            Self::FileExists { .. } => VerifierType::FileExists,
            Self::FileAbsent { .. } => VerifierType::FileAbsent,
            Self::Contains { .. } => VerifierType::Contains,
            Self::NotContains { .. } => VerifierType::NotContains,
            Self::JsonValid { .. } => VerifierType::JsonValid,
            Self::JsonSchema { .. } => VerifierType::JsonSchema,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        match self {
            Self::Command { command, .. } => reject_empty("command verifier command", command),
            Self::FileExists { path } | Self::FileAbsent { path } | Self::JsonValid { path } => {
                reject_empty_path(self.verifier_type(), path)
            }
            Self::Contains { path, needle } | Self::NotContains { path, needle } => {
                reject_empty_path(self.verifier_type(), path)?;
                if needle.is_empty() {
                    return Err(format!("{} verifier requires needle", self.verifier_type()));
                }
                Ok(())
            }
            Self::JsonSchema { path, schema_path } => {
                reject_empty_path(self.verifier_type(), path)?;
                if schema_path.trim().is_empty() {
                    return Err("json_schema verifier requires schema_path".to_string());
                }
                Ok(())
            }
        }
    }
}

fn default_expected_exit() -> i32 {
    0
}

fn reject_empty_path(verifier_type: VerifierType, path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err(format!("{verifier_type} verifier requires path"));
    }
    Ok(())
}

fn reject_empty(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MutationScope {
    pub(crate) kind: MutationScopeKind,
    pub(crate) path: String,
}

impl MutationScope {
    pub(crate) fn from_task_target(path: &str) -> Result<Self, String> {
        let normalized = normalize_scope_path(path)?;
        reject_broad_target(&normalized)?;
        if normalized.ends_with('/') {
            return Ok(Self {
                kind: MutationScopeKind::DirectoryTree,
                path: normalized,
            });
        }
        Ok(Self {
            kind: MutationScopeKind::ExactFile,
            path: normalized,
        })
    }

    pub(crate) fn allows(&self, candidate: &str) -> bool {
        let Ok(candidate) = normalize_scope_path(candidate) else {
            return false;
        };
        match self.kind {
            MutationScopeKind::ExactFile | MutationScopeKind::GeneratedArtifact => {
                candidate == self.path
            }
            MutationScopeKind::DirectoryTree | MutationScopeKind::GovernanceRepair => {
                candidate.starts_with(&self.path) && candidate.len() > self.path.len()
            }
        }
    }
}

fn normalize_scope_path(path: &str) -> Result<String, String> {
    let path = path.replace('\\', "/");
    let mut normalized = path.trim().trim_start_matches("./").to_string();
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }
    if normalized.is_empty() || normalized == "." || normalized.starts_with('/') {
        return Err(format!("invalid mutation scope path: {path}"));
    }
    if normalized.split('/').any(|part| part == "..") {
        return Err(format!("mutation scope path must not contain '..': {path}"));
    }
    Ok(normalized)
}

fn reject_broad_target(path: &str) -> Result<(), String> {
    let trimmed = path.trim_end_matches('/');
    if matches!(
        trimmed,
        "." | "" | "src" | "docs" | ".codex" | ".agents" | ".cursor" | "tools"
    ) {
        return Err(format!("mutation target is too broad: {path}"));
    }
    Ok(())
}
