use crate::content;
use culinator_core::Recipe;
use culinator_models::RecipeExportOptions;
pub(crate) fn render(recipe: &Recipe, options: &RecipeExportOptions) -> String {
    let c = content::extract(recipe);
    let mut out = format!(
        "{}\n{}\n\n",
        recipe.title,
        "=".repeat(recipe.title.chars().count())
    );
    if !c.summary.is_empty() {
        out.push_str(&c.summary);
        out.push_str("\n\n");
    }
    if let Some(d) = &options.description {
        out.push_str(d);
        out.push_str("\n\n");
    }
    out.push_str("INGREDIENTS\n");
    for group in &c.ingredient_groups {
        if let Some(label) = &group.label {
            out.push_str(&format!("\n{}\n", label.to_uppercase()));
        }
        for item in &group.items {
            out.push_str(&format!("- {item}\n"));
        }
    }
    if !c.equipment.is_empty() {
        out.push_str("\nEQUIPMENT\n");
        for item in &c.equipment {
            out.push_str(&format!("- {item}\n"));
        }
    }
    out.push_str("\nMETHOD\n");
    for section in &c.sections {
        if let Some(title) = &section.title {
            out.push_str(&format!("\n{}\n", title.to_uppercase()));
        }
        if let Some(note) = &section.note {
            out.push_str(&format!("{note}\n"));
        }
        for step in &section.steps {
            out.push_str(&format!("{}. {}\n", step.number, step.text));
            if let Some(annotation) = step.annotation() {
                out.push_str(&format!("   [{annotation}]\n"));
            }
        }
    }
    out
}

#[cfg(test)]
mod test;
