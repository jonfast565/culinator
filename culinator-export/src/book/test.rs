use super::*;
use culinator_core::{Operation, Process, Recipe, RecipeBook, TypeRef, Value};
use culinator_models::RecipeBookExporter;
use culinator_models::{BookExportFormat, BookExportOptions};
use std::collections::BTreeMap;
use uuid::Uuid;

#[test]
fn exports_zip_bundle_with_book_formats() {
    let book = RecipeBook {
        id: Uuid::new_v4(),
        symbol: "sample".into(),
        declared_type: TypeRef::named("RecipeBook"),
        title: "Sample".into(),
        description: None,
        protocol_version: "0.3".into(),
        recipes: vec![],
        properties: BTreeMap::new(),
    };
    let recipe = Recipe {
        id: Uuid::new_v4(),
        book_id: Some(book.id),
        symbol: "tea".into(),
        declared_type: TypeRef::named("Recipe"),
        title: "Tea".into(),
        protocol_version: "0.3".into(),
        types: vec![],
        resources: vec![],
        processes: vec![Process {
            id: Uuid::new_v4(),
            symbol: "prep".into(),
            declared_type: TypeRef::named("Process"),
            parent: None,
            operations: vec!["steep".into()],
            properties: BTreeMap::new(),
        }],
        operations: vec![Operation {
            id: Uuid::new_v4(),
            symbol: "steep".into(),
            declared_type: TypeRef::named("Operation"),
            process: "prep".into(),
            labor: None,
            duration_min_seconds: Some(180),
            duration_max_seconds: Some(180),
            duration_estimated: false,
            target_temperature: None,
            heat_level: None,
            doneness: vec![],
            optional: false,
            repeat: None,
            notes: vec![],
            dependencies: vec![],
            bindings: vec![],
            requirements: vec![],
            effects: vec![],
            properties: BTreeMap::from([(
                "description".into(),
                Value::Text("Steep the tea".into()),
            )]),
            span: None,
        }],
        servings: vec![],
        formulas: vec![],
        yields: vec![],
        properties: BTreeMap::new(),
    };
    let options = BookExportOptions {
        formats: vec![
            BookExportFormat::Epub,
            BookExportFormat::PrintHtml,
            BookExportFormat::Web,
        ],
        ..BookExportOptions::default()
    };
    let bundle = StaticRecipeBookExporter
        .export_book(&book, &[(recipe, "recipe tea {}".to_owned())], &options)
        .unwrap();
    assert!(bundle.archive.starts_with(b"PK"));
    assert!(bundle.files.iter().any(|file| file.path == "book.epub"));
    assert!(bundle.files.iter().any(|file| file.path == "print.html"));
    assert!(bundle.files.iter().any(|file| file.path == "index.html"));
}
