use crate::model::Result;
use crate::schema::HookFormat;
use serde_json::Value;

pub(crate) fn validate_payload_shape(format: HookFormat, value: &Value) -> Result<()> {
    match format {
        HookFormat::Codex => {
            if value.get("tool_name").is_some() || value.get("tool_input").is_some() {
                Ok(())
            } else {
                Err("codex hook payload requires tool_name or tool_input".to_string())
            }
        }
        HookFormat::Antigravity | HookFormat::Cursor => {
            if value.get("toolCall").is_some() || value.get("tool_call").is_some() {
                Ok(())
            } else {
                Err(format!("{format} hook payload requires toolCall"))
            }
        }
    }
}
