use crate::method_html;
use crate::method_html::escape;
use culinator_core::Recipe;
use culinator_models::{ApplicationError, RecipeExportOptions};
use serde_json::json;

pub(crate) fn render(
    recipe: &Recipe,
    options: &RecipeExportOptions,
    label_svg: &str,
) -> Result<String, ApplicationError> {
    let extracted = culinator_narrative::extract(recipe);
    let json_ld = json!({
        "@context": "https://schema.org",
        "@type": "Recipe",
        "name": recipe.title,
        "description": options.description,
        "author": options.author.as_ref().map(|name| json!({"@type":"Person","name":name})),
        "recipeIngredient": extracted.ingredients,
        "recipeInstructions": extracted.sections.iter().map(|section| {
            let steps = section.steps.iter().map(|step| {
                let mut item = json!({"@type":"HowToStep","text": step.text});
                if !step.tools.is_empty() {
                    item["tool"] = json!(step.tools);
                }
                item
            }).collect::<Vec<_>>();
            match &section.title {
                Some(title) => json!({"@type":"HowToSection","name":title,"itemListElement":steps}),
                None => json!({"@type":"HowToSection","itemListElement":steps}),
            }
        }).collect::<Vec<_>>(),
        "nutrition": {
            "@type":"NutritionInformation",
            "calories": format!("{} calories", options.nutrition.calories.round()),
            "fatContent": format!("{} g", options.nutrition.total_fat_grams),
            "carbohydrateContent": format!("{} g", options.nutrition.total_carbohydrate_grams),
            "proteinContent": format!("{} g", options.nutrition.protein_grams),
            "sodiumContent": format!("{} mg", options.nutrition.sodium_milligrams)
        }
    });
    let equipment = method_html::equipment_html(&extracted);
    let equipment_section = if equipment.is_empty() {
        String::new()
    } else {
        format!("<section><h2>Equipment</h2>{equipment}</section>")
    };
    Ok(format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title}</title><script type="application/ld+json">{json_ld}</script><style>{css}</style></head><body><main><header><p class="eyebrow">{site}</p><h1>{title}</h1><p class="summary">{summary}</p><p>{description}</p></header><div class="grid"><article><section><h2>Ingredients</h2>{ingredients}</section>{equipment_section}<section><h2>Method</h2>{method}</section></article><aside>{label}</aside></div></main></body></html>"#,
        title = escape(&recipe.title),
        site = escape(options.site_title.as_deref().unwrap_or("Culinator Recipe")),
        summary = escape(&extracted.summary),
        description = escape(options.description.as_deref().unwrap_or("")),
        ingredients = method_html::ingredients_html(&extracted),
        equipment_section = equipment_section,
        method = method_html::method_html(&extracted, 3),
        label = label_svg,
        json_ld = json_ld,
        css = CSS
    ))
}

const CSS: &str = r#"*{box-sizing:border-box}body{margin:0;background:#f7f4ed;color:#1d2721;font:17px/1.55 system-ui,sans-serif}main{max-width:1100px;margin:auto;padding:4rem 1.5rem}header{border-bottom:2px solid #1d2721;margin-bottom:2rem}.eyebrow{text-transform:uppercase;letter-spacing:.14em;font-size:.75rem}h1{font-family:Georgia,serif;font-size:clamp(2.5rem,7vw,5rem);line-height:.95;margin:.3rem 0 1rem}.summary{color:#5b6a60;letter-spacing:.02em}.grid{display:grid;grid-template-columns:minmax(0,1fr) 340px;gap:3rem}h2{font-family:Georgia,serif;font-size:1.7rem}h3{font-family:Georgia,serif;font-size:1.25rem;margin:1.6rem 0 .4rem}.note{color:#5b6a60}.variant{margin:.8rem 0 0}li{margin:.65rem 0}li small{display:block;color:#5b6a60}aside svg{width:100%;height:auto;background:white}@media(max-width:800px){.grid{grid-template-columns:1fr}aside{max-width:360px}}"#;

#[cfg(test)]
mod test;
