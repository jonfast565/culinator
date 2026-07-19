use culinator_core::Recipe;
use culinator_models::RecipeExportOptions;

pub(crate) fn render(recipe: &Recipe, options: &RecipeExportOptions) -> String {
    let content = culinator_narrative::extract(recipe);
    let mut out = format!("# {}\n\n", recipe.title);
    if !content.summary.is_empty() {
        out.push_str(&format!("{}\n\n", content.summary));
    }
    if let Some(description) = options
        .description
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        out.push_str(description);
        out.push_str("\n\n");
    }
    if let Some(author) = options.author.as_deref().filter(|value| !value.is_empty()) {
        out.push_str(&format!("**By:** {author}\n\n"));
    }
    out.push_str("## Ingredients\n");
    for group in &content.ingredient_groups {
        if let Some(label) = &group.label {
            out.push_str(&format!("\n**{label}**\n"));
        }
        out.push('\n');
        for item in &group.items {
            out.push_str(&format!("- {item}\n"));
        }
    }
    if !content.equipment.is_empty() {
        out.push_str("\n## Equipment\n\n");
        for item in &content.equipment {
            out.push_str(&format!("- {item}\n"));
        }
    }
    out.push_str("\n## Method\n");
    for section in &content.sections {
        if let Some(title) = &section.title {
            out.push_str(&format!("\n### {title}\n"));
        }
        if let Some(note) = &section.note {
            out.push_str(&format!("\n*{note}*\n"));
        }
        out.push('\n');
        for step in &section.steps {
            out.push_str(&format!("{}. {}", step.number, step.text));
            if let Some(annotation) = step.annotation() {
                out.push_str(&format!(" *({annotation})*"));
            }
            out.push('\n');
        }
    }
    out.push_str(&format!("\n## Nutrition per serving\n\n- Serving size: {}\n- Calories: {}\n- Protein: {} g\n- Carbohydrate: {} g\n- Fat: {} g\n", options.nutrition.serving_size, options.nutrition.calories.round(), options.nutrition.protein_grams, options.nutrition.total_carbohydrate_grams, options.nutrition.total_fat_grams));
    out
}

#[cfg(test)]
mod test;
