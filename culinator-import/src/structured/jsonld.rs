use super::{emit_cg, ingredient_text, instruction_text};
use culinator_models::{ApplicationError, ImportDraft};
use serde_json::Value;

pub(crate) fn import(content: &str) -> Result<ImportDraft, ApplicationError> {
    let value: Value = serde_json::from_str(content)
        .map_err(|error| ApplicationError::InvalidInput(error.to_string()))?;
    let recipe = find_recipe(&value).ok_or_else(|| {
        ApplicationError::InvalidInput("JSON-LD document does not contain a Recipe".into())
    })?;
    draft_from_recipe(recipe)
}

fn find_recipe(value: &Value) -> Option<&Value> {
    if is_recipe(value) {
        return Some(value);
    }
    if let Some(array) = value.as_array() {
        return array.iter().find_map(find_recipe);
    }
    if let Some(graph) = value.get("@graph").and_then(Value::as_array) {
        if let Some(recipe) = graph.iter().find_map(find_recipe) {
            return Some(recipe);
        }
    }
    value.as_object().and_then(|map| {
        map.values()
            .find_map(|entry| if is_recipe(entry) { Some(entry) } else { None })
    })
}

fn is_recipe(value: &Value) -> bool {
    match value.get("@type") {
        Some(Value::String(kind)) => kind.ends_with("Recipe"),
        Some(Value::Array(kinds)) => kinds
            .iter()
            .filter_map(Value::as_str)
            .any(|kind| kind.ends_with("Recipe")),
        _ => value.get("recipeIngredient").is_some() && value.get("name").is_some(),
    }
}

fn draft_from_recipe(recipe: &Value) -> Result<ImportDraft, ApplicationError> {
    let title = recipe
        .get("name")
        .or_else(|| recipe.get("title"))
        .and_then(Value::as_str)
        .ok_or_else(|| ApplicationError::InvalidInput("recipe name is required".into()))?
        .to_owned();
    let description = recipe
        .get("description")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let ingredients = recipe
        .get("recipeIngredient")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(ingredient_text).collect::<Vec<_>>())
        .unwrap_or_default();
    let instructions = recipe
        .get("recipeInstructions")
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
        warnings.push("no recipeIngredient entries found".to_owned());
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
