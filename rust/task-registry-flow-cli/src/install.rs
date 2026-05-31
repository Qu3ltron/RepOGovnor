use crate::model::Result;
use crate::schema::{InstallAction, InstallActionReport};
use std::str::FromStr;

pub(crate) fn action_report(
    path: impl Into<String>,
    action: impl AsRef<str>,
) -> Result<InstallActionReport> {
    Ok(InstallActionReport {
        path: path.into(),
        action: InstallAction::from_str(action.as_ref())?,
    })
}

pub(crate) fn validate_action_vocabulary(actions: &[String]) -> Result<Vec<InstallAction>> {
    actions
        .iter()
        .map(|action| InstallAction::from_str(action))
        .collect()
}

#[cfg(test)]
pub(crate) fn assert_dry_run_unchanged(before: &str, after: &str) -> Result<()> {
    if before == after {
        Ok(())
    } else {
        Err("dry-run installer mutated target state".to_string())
    }
}
