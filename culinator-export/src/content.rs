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
            format!("{quantity} {name}").trim().to_owned()
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
            let detail = operation
                .properties
                .get("description")
                .map(display_value)
                .unwrap_or_else(|| operation.symbol.replace('_', " "));
            if inputs.is_empty() {
                detail
            } else {
                format!("{detail}: {}", inputs.join(", "))
            }
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
