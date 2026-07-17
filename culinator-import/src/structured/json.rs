use super::{emit_cg, ingredient_text, instruction_text};
use culinator_models::{ApplicationError, ImportDraft};
use serde_json::Value;

pub(crate) fn import(content: &str) -> Result<ImportDraft, ApplicationError> {
    let value: Value = serde_json::from_str(content)
        .map_err(|error| ApplicationError::InvalidInput(error.to_string()))?;
    draft_from_value(&value)
}

pub(crate) fn draft_from_value(value: &Value) -> Result<ImportDraft, ApplicationError> {
    let title = value
        .get("title")
        .or_else(|| value.get("name"))
        .and_then(Value::as_str)
        .ok_or_else(|| ApplicationError::InvalidInput("title or name is required".into()))?
        .to_owned();
    let description = value
        .get("description")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let ingredients = value
        .get("ingredients")
        .or_else(|| value.get("recipeIngredient"))
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(ingredient_text).collect::<Vec<_>>())
        .unwrap_or_default();
    let instructions = value
        .get("instructions")
        .or_else(|| value.get("recipeInstructions"))
        .or_else(|| value.get("steps"))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(instruction_text)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut warnings = Vec::new();
    if ingredients.is_empty() {
        warnings.push("no ingredients found".to_owned());
    }
    let source_text = emit_cg(
        &title,
        description.as_deref(),
        &ingredients,
        &instructions,
        &mut warnings,
    );
    Ok(ImportDraft {
        title,
        source_text,
        warnings,
    })
}
