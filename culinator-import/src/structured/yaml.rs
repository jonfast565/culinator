use super::json;
use culinator_models::{ApplicationError, ImportDraft};

pub(crate) fn import(content: &str) -> Result<ImportDraft, ApplicationError> {
    let value: serde_yaml::Value = serde_yaml::from_str(content)
        .map_err(|error| ApplicationError::InvalidInput(error.to_string()))?;
    let json = serde_json::to_value(value)
        .map_err(|error| ApplicationError::Internal(error.to_string()))?;
    json::draft_from_value(&json)
}
