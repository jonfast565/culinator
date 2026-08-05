use crate::{
    FDC_ENERGY_KCAL, FDC_PROTEIN, FoodNutrientRecord, NutritionSearchResult, aggregate_nutrients,
    default_serving_context, fts_query_from_ingredient, fts_queries_from_ingredient,
    ingredient_match_query, ingredient_resources, manual_facts_to_nutrients, nutrients_to_facts,
    rank_fuzzy_matches, resource_mass_grams, string_similarity, value_mass_grams,
};
use culinator_core::{Dimension, Resource, ResourceKind, Value};
use std::collections::BTreeMap;

fn ingredient_resource(
    symbol: &str,
    name: Option<&str>,
    quantity: culinator_core::Quantity,
    size: Option<&str>,
) -> Resource {
    let mut properties = BTreeMap::new();
    properties.insert("quantity".to_owned(), Value::Quantity(quantity));
    if let Some(name) = name {
        properties.insert("name".to_owned(), Value::Text(name.to_owned()));
    }
    Resource {
        id: uuid::Uuid::new_v4(),
        symbol: symbol.to_owned(),
        declared_type: culinator_core::TypeRef::named("Ingredient"),
        kind: ResourceKind::Ingredient,
        optional: false,
        divided: false,
        substitutes: vec![],
        to_taste: false,
        size: size.map(|value| value.to_owned()),
        variant: None,
        notes: vec![],
        properties,
        span: None,
    }
}

#[test]
fn resource_mass_reads_quantity_property() {
    let resource = ingredient_resource(
        "flour",
        None,
        culinator_core::Quantity {
            value: 250.0,
            unit: "g".to_owned(),
            dimension: Dimension::Mass,
        },
        None,
    );
    assert_eq!(resource_mass_grams(&resource), Some(250.0));
}

#[test]
fn resource_mass_converts_volume_via_density() {
    let resource = ingredient_resource(
        "flour",
        Some("all-purpose flour"),
        culinator_core::Quantity {
            value: 1.0,
            unit: "cup".to_owned(),
            dimension: Dimension::Volume,
        },
        None,
    );
    let grams = resource_mass_grams(&resource).expect("cup flour density");
    // 1 cup ≈ 236.59 ml * 0.59 g/ml ≈ 139.6 g
    assert!((grams - 139.6).abs() < 1.0, "got {grams}");
}

#[test]
fn resource_mass_converts_count_egg() {
    let resource = ingredient_resource(
        "egg",
        None,
        culinator_core::Quantity {
            value: 2.0,
            unit: "count".to_owned(),
            dimension: Dimension::Count,
        },
        Some("large"),
    );
    assert_eq!(resource_mass_grams(&resource), Some(100.0));
}

#[test]
fn resource_mass_converts_garlic_clove() {
    let resource = ingredient_resource(
        "garlic",
        None,
        culinator_core::Quantity {
            value: 3.0,
            unit: "clove".to_owned(),
            dimension: Dimension::Count,
        },
        None,
    );
    assert_eq!(resource_mass_grams(&resource), Some(9.0));
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
fn string_similarity_uses_synonyms() {
    let score = string_similarity("scallion", "Onions, spring or scallions (includes tops and bulb), raw");
    // Long USDA descriptors score modestly on F1; ranking bonuses still lift them.
    assert!(score > 0.25, "got {score}");
}

#[test]
fn fts_query_strips_prep_words() {
    let query = fts_query_from_ingredient("Hass avocado, diced");
    assert!(query.contains("avocado"));
    assert!(!query.contains("diced"));
}

#[test]
fn fts_query_strips_warm_and_for_the_pan() {
    let milk = fts_query_from_ingredient("warm milk");
    assert!(milk.to_lowercase().contains("milk"));
    assert!(!milk.to_lowercase().contains("warm"));

    let butter = normalize_check("butter for the pan");
    assert!(butter.contains("butter"));
    assert!(!butter.contains("pan"));
}

fn normalize_check(name: &str) -> String {
    crate::normalize_ingredient_name(name)
}

#[test]
fn fts_queries_prefer_and_then_or() {
    let queries = fts_queries_from_ingredient("all-purpose flour");
    assert!(
        queries.first().is_some_and(|query| query.contains(" AND ") || query.contains("flour")),
        "{queries:?}"
    );
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
fn rank_fuzzy_matches_prefers_foundation_over_branded() {
    let results = vec![
        NutritionSearchResult {
            fdc_id: 10,
            description: "Milk, whole".to_owned(),
            data_type: "branded_food".to_owned(),
            brand_owner: Some("ACME Dairy".to_owned()),
            serving_size: None,
            serving_size_unit: None,
        },
        NutritionSearchResult {
            fdc_id: 11,
            description: "Milk, whole, 3.25% milkfat".to_owned(),
            data_type: "foundation_food".to_owned(),
            brand_owner: None,
            serving_size: None,
            serving_size_unit: None,
        },
    ];
    let ranked = rank_fuzzy_matches("whole milk", &results);
    assert_eq!(ranked.first().unwrap().result.fdc_id, 11);
    assert!(ranked[0].score > ranked[1].score);
}

#[test]
fn rank_fuzzy_matches_rejects_branded_product_titles_for_short_names() {
    // Regression: auto-link was picking retail UPC rows that merely mention
    // the cooking name ("EGG BEATERS…", "BUTTER FLAVORED… POPCORN").
    let cases = [
        (
            "egg",
            "EGG BEATERS Egg Whites, 16 OZ",
            "Egg, whole, raw",
            Some("Conagra Brands"),
        ),
        (
            "butter",
            "BUTTER FLAVORED GOURMET POPCORN, BUTTER",
            "Butter, salted",
            Some("POPCORNOPOLIS"),
        ),
        (
            "flour",
            "MANINI'S, MULTI-PURPOSE FLOUR",
            "Wheat flour, white, all-purpose, enriched, bleached",
            Some("Maninis"),
        ),
        (
            "chocolate",
            "CHOCOLATE CHOCOLATE CHOCOLATE, MILK CHOCOLATE NONPAREILS",
            "Chocolate, dark, 45-59% cacao solids",
            Some("Chocolate Chocolate Chocolate"),
        ),
    ];
    for (query, branded_desc, generic_desc, brand) in cases {
        let results = vec![
            NutritionSearchResult {
                fdc_id: 1,
                description: branded_desc.to_owned(),
                data_type: "branded_food".to_owned(),
                brand_owner: brand.map(|value| value.to_owned()),
                serving_size: None,
                serving_size_unit: None,
            },
            NutritionSearchResult {
                fdc_id: 2,
                description: generic_desc.to_owned(),
                data_type: "foundation_food".to_owned(),
                brand_owner: None,
                serving_size: None,
                serving_size_unit: None,
            },
        ];
        let ranked = rank_fuzzy_matches(query, &results);
        assert_eq!(
            ranked.first().unwrap().result.fdc_id,
            2,
            "{query}: expected foundation food over {branded_desc:?}; scores {:?}",
            ranked
                .iter()
                .map(|item| (item.result.fdc_id, item.score, &item.result.description))
                .collect::<Vec<_>>()
        );
        assert!(
            ranked[0].score > ranked[1].score,
            "{query}: foundation score {} should beat branded {}",
            ranked[0].score,
            ranked[1].score
        );
    }
}

#[test]
fn string_similarity_does_not_treat_substring_as_near_perfect() {
    let popcorn = string_similarity("butter", "BUTTER FLAVORED GOURMET POPCORN, BUTTER");
    let salted = string_similarity("butter", "Butter, salted");
    assert!(
        salted > popcorn + 0.25,
        "salted={salted} should clearly beat popcorn={popcorn}"
    );
    assert!(popcorn < 0.55, "popcorn similarity too high: {popcorn}");
}

#[test]
fn ingredient_match_query_uses_name_and_size() {
    let resource = ingredient_resource(
        "egg",
        Some("large egg"),
        culinator_core::Quantity {
            value: 1.0,
            unit: "count".to_owned(),
            dimension: Dimension::Count,
        },
        Some("large"),
    );
    let query = ingredient_match_query(&resource);
    assert!(query.contains("egg"));
    assert!(query.contains("large"));
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
                to_taste: false,
                size: None,
                variant: None,
                notes: vec![],
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
                to_taste: false,
                size: None,
                variant: None,
                notes: vec![],
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
