use super::*;
use culinator_core::{Recipe, TypeRef};
use culinator_models::{NutritionFacts, RecipeExportOptions, RecipeExporter};
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
        .export(&recipe, "recipe tea {}", &options)
        .unwrap();
    assert!(bundle.archive.starts_with(b"PK"));
    assert_eq!(bundle.media_type, "application/zip");
    assert!(bundle.files.iter().any(|f| f.path == "index.html"));
}

#[test]
fn single_format_export_returns_the_bare_file() {
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
        include_source: false,
        formats: vec![culinator_models::RecipeExportFormat::Markdown],
        nutrition: NutritionFacts::default(),
    };
    let bundle = StaticRecipeExporter
        .export(&recipe, "recipe tea {}", &options)
        .unwrap();
    assert_eq!(bundle.file_name, "tea.md");
    assert_eq!(bundle.media_type, "text/markdown; charset=utf-8");
    assert_eq!(bundle.files.len(), 1);
    assert!(String::from_utf8_lossy(&bundle.archive).starts_with("# Tea"));
}
