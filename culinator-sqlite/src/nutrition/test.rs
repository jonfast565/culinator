use culinator_models::{NutritionFacts, SaveIngredientManualNutritionRequest, SaveRecipeNutritionRequest};
use rusqlite::Connection;
use uuid::Uuid;

use crate::{migrate, nutrition};

fn open_db() -> Connection {
    let connection = Connection::open_in_memory().unwrap();
    migrate(&connection).unwrap();
    connection
}

fn sample_facts() -> NutritionFacts {
    NutritionFacts {
        servings_per_container: 1.0,
        serving_size: "100 g".to_owned(),
        serving_size_grams: Some(100.0),
        calories: 160.0,
        total_fat_grams: 14.0,
        saturated_fat_grams: 2.0,
        trans_fat_grams: 0.0,
        cholesterol_milligrams: 0.0,
        sodium_milligrams: 7.0,
        total_carbohydrate_grams: 8.0,
        dietary_fiber_grams: 6.0,
        total_sugars_grams: 0.5,
        added_sugars_grams: 0.0,
        protein_grams: 2.0,
        vitamin_d_micrograms: None,
        calcium_milligrams: None,
        iron_milligrams: None,
        potassium_milligrams: Some(485.0),
    }
}

#[test]
fn recipe_nutrition_override_round_trips() {
    let connection = open_db();
    let recipe_id = Uuid::new_v4();
    connection
        .execute(
            "INSERT INTO recipes (id, symbol, title, protocol_version, declared_type_json, source_text)
             VALUES (?1, 'test', 'Test', '0.3', '{}', 'recipe test {}')",
            [recipe_id.to_string()],
        )
        .unwrap();

    let facts = sample_facts();
    connection
        .execute(
            "INSERT INTO recipe_nutrition (recipe_id, manual_override, facts_json)
             VALUES (?1, 1, ?2)",
            rusqlite::params![recipe_id.to_string(), serde_json::to_string(&facts).unwrap()],
        )
        .unwrap();

    let state = nutrition::get_recipe_nutrition_state(&connection, &recipe_id.to_string()).unwrap();
    assert!(state.manual_override);
    assert_eq!(state.manual_facts.unwrap().calories, 160.0);
}

#[test]
fn manual_ingredient_nutrition_round_trips() {
    let connection = open_db();
    let recipe_id = Uuid::new_v4();
    connection
        .execute(
            "INSERT INTO recipes (id, symbol, title, protocol_version, declared_type_json, source_text)
             VALUES (?1, 'test', 'Test', '0.3', '{}', 'recipe test {}')",
            [recipe_id.to_string()],
        )
        .unwrap();

    let request = SaveIngredientManualNutritionRequest {
        resource_symbol: "avocado".to_owned(),
        facts_per_100g: sample_facts(),
    };
    let facts_json = serde_json::to_string(&request.facts_per_100g).unwrap();
    connection
        .execute(
            "INSERT INTO resource_nutrition_manual (recipe_id, resource_symbol, facts_per_100g_json)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![recipe_id.to_string(), request.resource_symbol, facts_json],
        )
        .unwrap();

    let entries =
        nutrition::list_manual_ingredient_nutrition(&connection, &recipe_id.to_string()).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].resource_symbol, "avocado");
    assert_eq!(entries[0].facts_per_100g.protein_grams, 2.0);
}

#[test]
fn save_recipe_nutrition_request_shape() {
    let request = SaveRecipeNutritionRequest {
        manual_override: true,
        facts: Some(sample_facts()),
    };
    assert!(request.manual_override);
    assert!(request.facts.is_some());
}
