use culinator_core::{BindingRole, Recipe, ResourceKind, Value};

pub(crate) struct RecipeContent {
    pub ingredients: Vec<String>,
    pub instructions: Vec<String>,
}

pub(crate) fn extract(recipe: &Recipe) -> RecipeContent {
    let ingredients = recipe
        .resources
        .iter()
        .filter(|resource| resource.kind == ResourceKind::Ingredient)
        .map(|resource| {
            let quantity = resource
                .properties
                .get("quantity")
                .map(display_value)
                .unwrap_or_default();
            let name = resource
                .properties
                .get("name")
                .map(display_value)
                .unwrap_or_else(|| resource.symbol.replace('_', " "));
            // Reassemble the prose line, folding in the structured nuance that a
            // bare "quantity name" would drop: size grade, handling notes, and
            // the open-ended "plus more to taste".
            let size = resource.size.as_deref().unwrap_or("");
            let mut line = format!("{quantity} {size} {name}")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            for note in &resource.notes {
                line.push_str(&format!(", {note}"));
            }
            if resource.to_taste {
                line.push_str(", plus more to taste");
            }
            line
        })
        .collect();
    let instructions = recipe
        .operations
        .iter()
        .map(|operation| {
            let inputs = operation
                .bindings
                .iter()
                .filter(|binding| binding.role == BindingRole::Input)
                .map(|binding| binding.resource.replace('_', " "))
                .collect::<Vec<_>>();
            let mut detail = operation
                .properties
                .get("description")
                .map(display_value)
                .unwrap_or_else(|| operation.symbol.replace('_', " "));
            if let Some(repeat) = operation.repeat {
                detail.push_str(&format!(" (repeat {repeat}\u{00d7})"));
            }
            let mut line = if inputs.is_empty() {
                detail
            } else {
                format!("{detail}: {}", inputs.join(", "))
            };
            for note in &operation.notes {
                line.push_str(&format!(". {note}"));
            }
            line
        })
        .collect();
    RecipeContent {
        ingredients,
        instructions,
    }
}

pub(crate) fn display_value(value: &Value) -> String {
    match value {
        Value::Text(value) | Value::Symbol(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Boolean(value) => value.to_string(),
        Value::Quantity(quantity) => format!("{} {}", quantity.value, quantity.unit),
        Value::List(values) => values
            .iter()
            .map(display_value)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Range { min, max } => {
            format!("{}\u{2013}{}", display_value(min), display_value(max))
        }
        Value::Object(_) => String::new(),
    }
}

#[cfg(test)]
mod test;
