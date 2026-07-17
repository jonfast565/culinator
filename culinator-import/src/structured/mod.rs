mod json;
mod jsonld;
mod yaml;

use culinator_models::{
    ApplicationError, ImportDraft, StructuredInput, StructuredInputFormat, StructuredRecipeImporter,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct StructuredRecipeParser;

impl StructuredRecipeImporter for StructuredRecipeParser {
    fn import(&self, input: StructuredInput) -> Result<ImportDraft, ApplicationError> {
        match input.format {
            StructuredInputFormat::JsonLd => jsonld::import(&input.content),
            StructuredInputFormat::Json => json::import(&input.content),
            StructuredInputFormat::Yaml => yaml::import(&input.content),
        }
    }
}

pub(crate) fn emit_cg(
    title: &str,
    description: Option<&str>,
    ingredients: &[String],
    instructions: &[String],
    warnings: &mut Vec<String>,
) -> String {
    let symbol = symbolize(title);
    let mut out = String::from("culinator 0.3;\n\n");
    out.push_str(&format!("recipe {symbol} {{\n"));
    out.push_str(&format!("    title {};\n", quote(title)));
    if let Some(text) = description.filter(|value| !value.trim().is_empty()) {
        out.push_str(&format!("    description {};\n", quote(text)));
    }

    for (index, ingredient) in ingredients.iter().enumerate() {
        let name = ingredient.trim();
        if name.is_empty() {
            warnings.push(format!("skipped empty ingredient at index {index}"));
            continue;
        }
        let ingredient_symbol = format!("ingredient_{index}");
        out.push_str(&format!(
            "\n    ingredient {ingredient_symbol} measured by mass {{\n        name {};\n    }}\n",
            quote(name)
        ));
    }

    if !instructions.is_empty() {
        out.push_str("\n    process preparation {\n");
        for (index, instruction) in instructions.iter().enumerate() {
            let text = instruction.trim();
            if text.is_empty() {
                warnings.push(format!("skipped empty instruction at index {index}"));
                continue;
            }
            out.push_str(&format!(
                "        operation step_{index} does prepare {{\n            description {};\n            duration 5 min;\n            labor active;\n        }}\n",
                quote(text)
            ));
        }
        out.push_str("    }\n");
    } else {
        warnings.push("no instructions found; emitted ingredients only".to_owned());
    }

    out.push_str("}\n");
    out
}

pub(crate) fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

pub(crate) fn symbolize(value: &str) -> String {
    let mut out = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            out.push(character.to_ascii_lowercase());
        } else if !out.ends_with('_') && !out.is_empty() {
            out.push('_');
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "imported_recipe".to_owned()
    } else {
        trimmed.to_owned()
    }
}

pub(crate) fn instruction_text(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Object(map) => map
            .get("text")
            .or_else(|| map.get("name"))
            .and_then(|entry| entry.as_str())
            .map(str::to_owned),
        _ => None,
    }
}

pub(crate) fn ingredient_text(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(text) => Some(text.clone()),
        serde_json::Value::Object(map) => {
            if let Some(text) = map.get("text").and_then(|entry| entry.as_str()) {
                return Some(text.to_owned());
            }
            let name = map.get("name").and_then(|entry| entry.as_str())?;
            let amount = map
                .get("amount")
                .or_else(|| map.get("quantity"))
                .and_then(|entry| entry.as_str());
            Some(match amount {
                Some(amount) => format!("{amount} {name}"),
                None => name.to_owned(),
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod test;
