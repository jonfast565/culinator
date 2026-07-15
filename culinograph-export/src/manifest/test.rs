use super::*;
use culinograph_core::{Recipe, TypeRef};
use culinograph_models::{NutritionFacts, RecipeExportOptions};
use std::collections::BTreeMap;
use uuid::Uuid;
#[test]
fn lists_files() {
    let r = Recipe {
        id: Uuid::new_v4(),
        book_id: None,
        symbol: "x".into(),
        declared_type: TypeRef::named("Recipe"),
        title: "X".into(),
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
    let o = RecipeExportOptions {
        site_title: None,
        author: None,
        description: None,
        include_source: false,
        formats: vec![],
        nutrition: NutritionFacts::default(),
    };
    let f = ExportFile {
        path: "recipe.md".into(),
        media_type: "text/markdown".into(),
        contents: vec![],
    };
    assert!(render(&r, &o, &[f]).contains("recipe.md"));
}
