use super::*;
use culinograph_core::{Recipe, TypeRef};
use culinograph_models::{NutritionFacts, RecipeExportOptions, RecipeExporter};
use std::collections::BTreeMap;
use uuid::Uuid;
#[test]
fn creates_zip_bundle() {
    let recipe = Recipe {
        id: Uuid::nil(),
        book_id: None,
        symbol: "tea".into(),
        declared_type: TypeRef::named("Recipe"),
        title: "Tea".into(),
        protocol_version: "0.3".into(),
        types: vec![],
        resources: vec![],
        processes: vec![],
        operations: vec![],
        servings: vec![],
        formulas: vec![],
        yields: vec![],
        properties: BTreeMap::new(),
    };
    let options = RecipeExportOptions {
        site_title: None,
        author: None,
        description: None,
        include_source: true,
        formats: vec![
            culinograph_models::RecipeExportFormat::Web,
            culinograph_models::RecipeExportFormat::Markdown,
            culinograph_models::RecipeExportFormat::PlainText,
            culinograph_models::RecipeExportFormat::IngredientCsv,
            culinograph_models::RecipeExportFormat::Json,
            culinograph_models::RecipeExportFormat::PrintHtml,
            culinograph_models::RecipeExportFormat::Epub,
        ],
        nutrition: NutritionFacts::default(),
    };
    let bundle = StaticRecipeExporter
        .export(&recipe, "recipe tea {}", &options)
        .unwrap();
    assert!(bundle.archive.starts_with(b"PK"));
    assert!(bundle.files.iter().any(|f| f.path == "index.html"));
}
