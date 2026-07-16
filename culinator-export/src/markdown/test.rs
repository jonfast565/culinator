use super::*;
use culinator_core::{Recipe, TypeRef};
use culinator_models::{NutritionFacts, RecipeExportOptions};
use std::collections::BTreeMap;
use uuid::Uuid;
#[test]
fn renders_heading() {
    let recipe = Recipe {
        id: Uuid::new_v4(),
        book_id: None,
        symbol: "x".into(),
        declared_type: TypeRef::named("Recipe"),
        title: "Toast".into(),
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
    assert!(render(&recipe, &o).starts_with("# Toast"));
}
