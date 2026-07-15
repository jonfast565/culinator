use crate::content;
use culinograph_core::Recipe;
use culinograph_models::RecipeExportOptions;

pub(crate) fn render(recipe: &Recipe, options: &RecipeExportOptions) -> String {
    let content = content::extract(recipe);
    let mut out = format!("# {}\n\n", recipe.title);
    if let Some(description) = options.description.as_deref().filter(|value| !value.is_empty()) { out.push_str(description); out.push_str("\n\n"); }
    if let Some(author) = options.author.as_deref().filter(|value| !value.is_empty()) { out.push_str(&format!("**By:** {author}\n\n")); }
    out.push_str("## Ingredients\n\n");
    for ingredient in content.ingredients { out.push_str(&format!("- {ingredient}\n")); }
    out.push_str("\n## Method\n\n");
    for (index, instruction) in content.instructions.iter().enumerate() { out.push_str(&format!("{}. {instruction}\n", index + 1)); }
    out.push_str(&format!("\n## Nutrition per serving\n\n- Serving size: {}\n- Calories: {}\n- Protein: {} g\n- Carbohydrate: {} g\n- Fat: {} g\n", options.nutrition.serving_size, options.nutrition.calories.round(), options.nutrition.protein_grams, options.nutrition.total_carbohydrate_grams, options.nutrition.total_fat_grams));
    out
}

#[cfg(test)]
mod test;
