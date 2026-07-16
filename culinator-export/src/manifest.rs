use culinator_core::Recipe;
use culinator_models::{ExportFile, RecipeExportOptions};
use serde_json::json;

pub(crate) fn render(
    recipe: &Recipe,
    options: &RecipeExportOptions,
    files: &[ExportFile],
) -> String {
    serde_json::to_string_pretty(&json!({
        "format": "culinator-export",
        "version": 2,
        "recipeId": recipe.id,
        "recipeTitle": recipe.title,
        "files": files.iter().map(|file| &file.path).collect::<Vec<_>>(),
        "formats": options.formats,
        "includesSource": options.include_source
    }))
    .unwrap_or_else(|_| "{}".to_owned())
}

#[cfg(test)]
mod test;
