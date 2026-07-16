use anyhow::{Context, Result};
use culinator_export::StaticRecipeExporter;
use culinator_models::{NutritionFacts, RecipeExportOptions, RecipeExporter};
use culinator_parser::parse_recipe;
use std::{fs, path::Path};

pub fn export_recipe(file: &Path, output: &Path) -> Result<()> {
    let source = fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
    let recipe = parse_recipe(&source).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let options = RecipeExportOptions {
        site_title: None,
        author: None,
        description: None,
        include_source: true,
        formats: vec![
            culinator_models::RecipeExportFormat::Web,
            culinator_models::RecipeExportFormat::Markdown,
            culinator_models::RecipeExportFormat::PlainText,
            culinator_models::RecipeExportFormat::IngredientCsv,
            culinator_models::RecipeExportFormat::Json,
            culinator_models::RecipeExportFormat::PrintHtml,
            culinator_models::RecipeExportFormat::Epub,
        ],
        nutrition: NutritionFacts::default(),
    };
    let bundle = StaticRecipeExporter
        .export(&recipe, &source, &options)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    fs::write(output, bundle.archive).with_context(|| format!("write {}", output.display()))?;
    println!("Exported {}", output.display());
    Ok(())
}
#[cfg(test)]
mod test;
