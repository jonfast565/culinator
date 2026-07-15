use culinograph_core::{BindingRole, Recipe, ResourceKind, Value};
use culinograph_models::{ApplicationError, RecipeExportOptions};
use serde_json::json;

pub(crate) fn render(
    recipe: &Recipe,
    options: &RecipeExportOptions,
    label_svg: &str,
) -> Result<String, ApplicationError> {
    let ingredients = recipe
        .resources
        .iter()
        .filter(|r| r.kind == ResourceKind::Ingredient)
        .map(|r| {
            let quantity = r
                .properties
                .get("quantity")
                .map(display_value)
                .unwrap_or_default();
            format!(
                "{} {}",
                quantity,
                r.properties
                    .get("name")
                    .map(display_value)
                    .unwrap_or_else(|| r.symbol.clone())
            )
            .trim()
            .to_owned()
        })
        .collect::<Vec<_>>();
    let instructions = recipe
        .operations
        .iter()
        .map(|op| {
            let inputs = op
                .bindings
                .iter()
                .filter(|b| b.role == BindingRole::Input)
                .map(|b| b.resource.clone())
                .collect::<Vec<_>>();
            let detail = op
                .properties
                .get("description")
                .map(display_value)
                .unwrap_or_else(|| op.symbol.replace('_', " "));
            if inputs.is_empty() {
                detail
            } else {
                format!("{}: {}", detail, inputs.join(", "))
            }
        })
        .collect::<Vec<_>>();
    let json_ld = json!({
        "@context": "https://schema.org",
        "@type": "Recipe",
        "name": recipe.title,
        "description": options.description,
        "author": options.author.as_ref().map(|name| json!({"@type":"Person","name":name})),
        "recipeIngredient": ingredients,
        "recipeInstructions": instructions.iter().map(|text| json!({"@type":"HowToStep","text":text})).collect::<Vec<_>>(),
        "nutrition": {
            "@type":"NutritionInformation",
            "calories": format!("{} calories", options.nutrition.calories.round()),
            "fatContent": format!("{} g", options.nutrition.total_fat_grams),
            "carbohydrateContent": format!("{} g", options.nutrition.total_carbohydrate_grams),
            "proteinContent": format!("{} g", options.nutrition.protein_grams),
            "sodiumContent": format!("{} mg", options.nutrition.sodium_milligrams)
        }
    });
    let ingredient_html = ingredients
        .iter()
        .map(|x| format!("<li>{}</li>", escape(x)))
        .collect::<String>();
    let instruction_html = instructions
        .iter()
        .map(|x| format!("<li>{}</li>", escape(x)))
        .collect::<String>();
    Ok(format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title}</title><script type="application/ld+json">{json_ld}</script><style>{css}</style></head><body><main><header><p class="eyebrow">{site}</p><h1>{title}</h1><p>{description}</p></header><div class="grid"><article><section><h2>Ingredients</h2><ul>{ingredients}</ul></section><section><h2>Method</h2><ol>{instructions}</ol></section></article><aside>{label}</aside></div></main></body></html>"#,
        title = escape(&recipe.title),
        site = escape(
            options
                .site_title
                .as_deref()
                .unwrap_or("Culinograph Recipe")
        ),
        description = escape(options.description.as_deref().unwrap_or("")),
        ingredients = ingredient_html,
        instructions = instruction_html,
        label = label_svg,
        json_ld = json_ld,
        css = CSS
    ))
}

fn display_value(value: &Value) -> String {
    match value {
        Value::Text(v) | Value::Symbol(v) => v.clone(),
        Value::Number(v) => v.to_string(),
        Value::Boolean(v) => v.to_string(),
        Value::Quantity(q) => format!("{} {}", q.value, q.unit),
        Value::List(v) => v.iter().map(display_value).collect::<Vec<_>>().join(", "),
        Value::Object(_) => String::new(),
    }
}
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
const CSS: &str = r#"*{box-sizing:border-box}body{margin:0;background:#f7f4ed;color:#1d2721;font:17px/1.55 system-ui,sans-serif}main{max-width:1100px;margin:auto;padding:4rem 1.5rem}header{border-bottom:2px solid #1d2721;margin-bottom:2rem}.eyebrow{text-transform:uppercase;letter-spacing:.14em;font-size:.75rem}h1{font-family:Georgia,serif;font-size:clamp(2.5rem,7vw,5rem);line-height:.95;margin:.3rem 0 1rem}.grid{display:grid;grid-template-columns:minmax(0,1fr) 340px;gap:3rem}h2{font-family:Georgia,serif;font-size:1.7rem}li{margin:.65rem 0}aside svg{width:100%;height:auto;background:white}@media(max-width:800px){.grid{grid-template-columns:1fr}aside{max-width:360px}}"#;

#[cfg(test)]
mod test;
