use culinator_core::{
    Formula, FormulaBasis, FormulaIngredient, LaborMode, Recipe, ResourceKind, TypeRef, Value,
};
use culinator_models::SearchQuery;
use rusqlite::Connection;
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::{migrate, save_recipe};

fn test_recipe(title: &str, ingredients: &[&str], does: &[&str]) -> Recipe {
    Recipe {
        id: Uuid::new_v4(),
        book_id: None,
        symbol: "test_recipe".to_owned(),
        declared_type: TypeRef::named("Recipe"),
        title: title.to_owned(),
        protocol_version: "1".to_owned(),
        types: Vec::new(),
        resources: ingredients
            .iter()
            .map(|symbol| culinator_core::Resource {
                id: Uuid::new_v4(),
                symbol: (*symbol).to_owned(),
                declared_type: TypeRef::named("Ingredient"),
                kind: ResourceKind::Ingredient,
                optional: false,
                divided: false,
                substitutes: Vec::new(),
                to_taste: false,
                size: None,
                variant: None,
                notes: Vec::new(),
                properties: BTreeMap::new(),
                span: None,
            })
            .collect(),
        processes: Vec::new(),
        operations: does
            .iter()
            .enumerate()
            .map(|(index, action)| culinator_core::Operation {
                id: Uuid::new_v4(),
                symbol: format!("step_{index}"),
                declared_type: TypeRef::named("Operation"),
                process: "main".to_owned(),
                labor: Some(LaborMode::Active),
                duration_min_seconds: Some(600),
                duration_max_seconds: Some(600),
                duration_estimated: false,
                target_temperature: None,
                heat_level: None,
                doneness: Vec::new(),
                optional: false,
                repeat: None,
                notes: Vec::new(),
                dependencies: Vec::new(),
                bindings: Vec::new(),
                requirements: Vec::new(),
                effects: Vec::new(),
                properties: [("does".to_owned(), Value::Text((*action).to_owned()))]
                    .into_iter()
                    .collect(),
                span: None,
            })
            .collect(),
        servings: Vec::new(),
        formulas: Vec::new(),
        yields: Vec::new(),
        properties: BTreeMap::new(),
    }
}

#[test]
fn fts_query_finds_title_and_ingredients() {
    let mut connection = Connection::open_in_memory().expect("open");
    migrate(&connection).expect("migrate");

    let recipe = test_recipe("Garlic Bread", &["garlic", "butter", "bread"], &["bake"]);
    save_recipe(&mut connection, &recipe, "recipe test { }").expect("save");

    let hits = super::search(
        &connection,
        &SearchQuery {
            text: Some("garlic".to_owned()),
            book_id: None,
            exclude_allergens: Vec::new(),
            max_active_minutes: None,
            hydration: None,
            limit: 10,
        },
    )
    .expect("search");

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "Garlic Bread");
}

#[test]
fn active_time_filter_excludes_long_recipes() {
    let mut connection = Connection::open_in_memory().expect("open");
    migrate(&connection).expect("migrate");

    let mut short = test_recipe("Quick Toast", &["bread"], &["toast"]);
    short.operations[0].duration_max_seconds = Some(300);
    save_recipe(&mut connection, &short, "recipe short { }").expect("save");

    let hits = super::search(
        &connection,
        &SearchQuery {
            text: None,
            book_id: None,
            exclude_allergens: Vec::new(),
            max_active_minutes: Some(8.0),
            hydration: None,
            limit: 10,
        },
    )
    .expect("search");
    assert_eq!(hits.len(), 1);

    let hits = super::search(
        &connection,
        &SearchQuery {
            text: None,
            book_id: None,
            exclude_allergens: Vec::new(),
            max_active_minutes: Some(4.0),
            hydration: None,
            limit: 10,
        },
    )
    .expect("search");
    assert!(hits.is_empty());
}

#[test]
fn hydration_filter_uses_formula_metric() {
    let mut connection = Connection::open_in_memory().expect("open");
    migrate(&connection).expect("migrate");

    let mut recipe = test_recipe("Country Loaf", &["flour", "water"], &["mix"]);
    recipe.formulas.push(Formula {
        id: Uuid::new_v4(),
        recipe_id: Some(recipe.id),
        symbol: "dough".to_owned(),
        name: "Dough".to_owned(),
        basis: FormulaBasis::ReferencePercent,
        ingredients: vec![
            FormulaIngredient {
                id: Uuid::new_v4(),
                symbol: "flour".to_owned(),
                name: "Flour".to_owned(),
                stage: "final".to_owned(),
                basis: FormulaBasis::ReferencePercent,
                percentage: Some(100.0),
                mass_grams: None,
                is_reference: true,
                is_flour: true,
                water_fraction: 0.0,
                scalable: true,
                properties: BTreeMap::new(),
            },
            FormulaIngredient {
                id: Uuid::new_v4(),
                symbol: "water".to_owned(),
                name: "Water".to_owned(),
                stage: "final".to_owned(),
                basis: FormulaBasis::ReferencePercent,
                percentage: Some(75.0),
                mass_grams: None,
                is_reference: false,
                is_flour: false,
                water_fraction: 1.0,
                scalable: true,
                properties: BTreeMap::new(),
            },
        ],
        properties: BTreeMap::new(),
    });
    save_recipe(&mut connection, &recipe, "recipe loaf { }").expect("save");

    let hits = super::search(
        &connection,
        &SearchQuery {
            text: None,
            book_id: None,
            exclude_allergens: Vec::new(),
            max_active_minutes: None,
            hydration: Some(culinator_models::RangeF64 {
                min: Some(70.0),
                max: Some(80.0),
            }),
            limit: 10,
        },
    )
    .expect("search");
    assert_eq!(hits.len(), 1);
}
