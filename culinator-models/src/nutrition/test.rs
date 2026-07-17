use crate::{
    FDC_ENERGY_KCAL, FDC_PROTEIN, FoodNutrientRecord, NutritionSearchResult, aggregate_nutrients,
    default_serving_context, fts_query_from_ingredient, ingredient_resources,
    manual_facts_to_nutrients, nutrients_to_facts, rank_fuzzy_matches, resource_mass_grams,
    string_similarity, value_mass_grams,
};
use culinator_core::{Dimension, Resource, ResourceKind, Value};
use std::collections::BTreeMap;

#[test]
fn resource_mass_reads_quantity_property() {
    let mut properties = std::collections::BTreeMap::new();
    properties.insert(
        "quantity".to_owned(),
        Value::Quantity(culinator_core::Quantity {
            value: 250.0,
            unit: "g".to_owned(),
            dimension: Dimension::Mass,
        }),
    );
    let resource = Resource {
        id: uuid::Uuid::new_v4(),
        symbol: "flour".to_owned(),
        declared_type: culinator_core::TypeRef::named("Ingredient"),
        kind: ResourceKind::Ingredient,
        optional: false,
        divided: false,
        substitutes: vec![],
        properties,
        span: None,
    };
    assert_eq!(resource_mass_grams(&resource), Some(250.0));
}

#[test]
fn aggregate_scales_by_mass() {
    let totals = aggregate_nutrients(&[(
        200.0,
        vec![FoodNutrientRecord {
            id: Some(1),
            fdc_id: 1,
            nutrient_id: FDC_PROTEIN,
            amount: Some(10.0),
            data_points: None,
            derivation_id: None,
            min: None,
            max: None,
            median: None,
        }],
    )]);
    assert_eq!(totals.get(&FDC_PROTEIN), Some(&20.0));
}

#[test]
fn nutrients_to_facts_divides_by_servings() {
    let mut totals = BTreeMap::new();
    totals.insert(FDC_ENERGY_KCAL, 400.0);
    totals.insert(FDC_PROTEIN, 20.0);
    let facts = nutrients_to_facts(&totals, 400.0, 2.0, "1 cup", Some(200.0));
    assert_eq!(facts.calories, 200.0);
    assert_eq!(facts.protein_grams, 10.0);
    assert_eq!(facts.servings_per_container, 2.0);
}

#[test]
fn string_similarity_prefers_close_names() {
    let score = string_similarity("Hass avocado", "Avocados, raw, all commercial varieties");
    assert!(score > 0.2);
    assert!(string_similarity("avocado", "avocado") > 0.99);
}

#[test]
fn fts_query_strips_prep_words() {
    let query = fts_query_from_ingredient("Hass avocado, diced");
    assert!(query.contains("avocado"));
    assert!(!query.contains("diced"));
}

#[test]
fn manual_facts_to_nutrients_maps_macros() {
    let facts = nutrients_to_facts(
        &BTreeMap::from([(FDC_PROTEIN, 20.0)]),
        100.0,
        1.0,
        "100 g",
        Some(100.0),
    );
    let nutrients = manual_facts_to_nutrients(&facts);
    assert!(nutrients.iter().any(|item| item.nutrient_id == FDC_PROTEIN));
}

#[test]
fn rank_fuzzy_matches_orders_by_score() {
    let results = vec![
        NutritionSearchResult {
            fdc_id: 1,
            description: "Avocados, raw".to_owned(),
            data_type: "Foundation".to_owned(),
            brand_owner: None,
            serving_size: None,
            serving_size_unit: None,
        },
        NutritionSearchResult {
            fdc_id: 2,
            description: "White bread".to_owned(),
            data_type: "Foundation".to_owned(),
            brand_owner: None,
            serving_size: None,
            serving_size_unit: None,
        },
    ];
    let ranked = rank_fuzzy_matches("avocado", &results);
    assert_eq!(ranked.first().unwrap().result.fdc_id, 1);
}

#[test]
fn ingredient_resources_filters_kind() {
    let recipe = culinator_core::Recipe {
        id: uuid::Uuid::new_v4(),
        book_id: None,
        symbol: "test".to_owned(),
        declared_type: culinator_core::TypeRef::named("Recipe"),
        title: "Test".to_owned(),
        protocol_version: "0.3".to_owned(),
        types: vec![],
        resources: vec![
            Resource {
                id: uuid::Uuid::new_v4(),
                symbol: "flour".to_owned(),
                declared_type: culinator_core::TypeRef::named("Ingredient"),
                kind: ResourceKind::Ingredient,
                optional: false,
                divided: false,
                substitutes: vec![],
                properties: Default::default(),
                span: None,
            },
            Resource {
                id: uuid::Uuid::new_v4(),
                symbol: "bowl".to_owned(),
                declared_type: culinator_core::TypeRef::named("Container"),
                kind: ResourceKind::Container,
                optional: false,
                divided: false,
                substitutes: vec![],
                properties: Default::default(),
                span: None,
            },
        ],
        processes: vec![],
        operations: vec![],
        servings: vec![],
        formulas: vec![],
        yields: vec![],
        properties: Default::default(),
    };
    assert_eq!(ingredient_resources(&recipe).len(), 1);
    assert_eq!(default_serving_context(&recipe.servings).0, 1.0);
    assert_eq!(value_mass_grams(&Value::Number(1.0)), None);
}
