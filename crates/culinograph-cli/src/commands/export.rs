use anyhow::{Context, Result};
use culinograph_export::StaticRecipeExporter;
use culinograph_models::{NutritionFacts, RecipeExportOptions, RecipeExporter};
use culinograph_parser::parse_recipe;
use std::{fs, path::Path};

pub fn export_recipe(file: &Path, output: &Path) -> Result<()> {
    let source=fs::read_to_string(file).with_context(||format!("read {}",file.display()))?;
    let recipe=parse_recipe(&source).map_err(|e|anyhow::anyhow!(e.to_string()))?;
    let options=RecipeExportOptions{site_title:None,author:None,description:None,include_source:true,formats:vec![culinograph_models::RecipeExportFormat::Web,culinograph_models::RecipeExportFormat::Markdown,culinograph_models::RecipeExportFormat::PlainText,culinograph_models::RecipeExportFormat::IngredientCsv,culinograph_models::RecipeExportFormat::Json,culinograph_models::RecipeExportFormat::PrintHtml,culinograph_models::RecipeExportFormat::Epub],nutrition:NutritionFacts::default()};
    let bundle=StaticRecipeExporter.export(&recipe,&source,&options).map_err(|e|anyhow::anyhow!(e.to_string()))?;
    fs::write(output,bundle.archive).with_context(||format!("write {}",output.display()))?;
    println!("Exported {}",output.display());
    Ok(())
}
#[cfg(test)] mod test;
