use crate::model::Result;

pub(crate) fn require_schema_version_v2(schema_version: i64, plan_path: &str) -> Result<()> {
    if schema_version == 2 {
        Ok(())
    } else {
        Err(format!(
            "{plan_path} Task Manifest schema_version must be 2"
        ))
    }
}
