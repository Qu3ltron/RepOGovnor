use crate::model::Result;
use crate::schema::InstallAction;
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstallPolicy {
    pub(crate) action_vocabulary: Vec<String>,
    pub(crate) dry_run_prefix: String,
    pub(crate) stale_absent: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ManifestPolicy {
    pub(crate) install_policy: InstallPolicy,
}

pub(crate) fn parse_manifest_policy(body: &str) -> Result<ManifestPolicy> {
    let policy = toml::from_str::<ManifestPolicy>(body)
        .map_err(|error| format!("parse MANIFEST.toml policy: {error}"))?;
    for action in &policy.install_policy.action_vocabulary {
        InstallAction::from_str(action)?;
    }
    if policy.install_policy.dry_run_prefix.trim().is_empty() {
        return Err("install_policy.dry_run_prefix must not be empty".to_string());
    }
    for path in &policy.install_policy.stale_absent {
        if path.trim().is_empty() {
            return Err("install_policy.stale_absent entries must not be empty".to_string());
        }
    }
    Ok(policy)
}
