use super::SqliteNutritionCatalog;
use culinator_models::{
    FoodRecord, NutritionCatalog, NutritionImportStore, NutritionSearchOptions, is_branded_food,
};

fn branded_food(fdc_id: i64, description: &str) -> FoodRecord {
    FoodRecord {
        fdc_id,
        data_type: "branded_food".into(),
        description: description.into(),
        food_category_id: None,
        publication_date: None,
        brand_owner: Some("Retail Brand".into()),
        brand_name: None,
        gtin_upc: None,
        ingredients: None,
        serving_size: None,
        serving_size_unit: None,
    }
}

fn foundation_food(fdc_id: i64, description: &str) -> FoodRecord {
    FoodRecord {
        fdc_id,
        data_type: "foundation_food".into(),
        description: description.into(),
        food_category_id: None,
        publication_date: None,
        brand_owner: None,
        brand_name: None,
        gtin_upc: None,
        ingredients: None,
        serving_size: None,
        serving_size_unit: None,
    }
}

#[test]
fn search_prefers_generic_foods_for_short_cooking_names() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = SqliteNutritionCatalog::open(dir.path().join("fdc.sqlite")).unwrap();
    db.begin_import("test").unwrap();
    db.upsert_food(&branded_food(
        1,
        "BUTTER FLAVORED GOURMET POPCORN, BUTTER",
    ))
    .unwrap();
    db.upsert_food(&branded_food(2, "GREEN BUTTER LETTUCE, RED BUTTER LETTUCE"))
        .unwrap();
    db.upsert_food(&foundation_food(3, "Butter, stick, salted"))
        .unwrap();
    db.upsert_food(&foundation_food(4, "Butter, stick, unsalted"))
        .unwrap();
    db.finish_import().unwrap();

    let hits = db
        .search_foods("butter", 5, NutritionSearchOptions::generics_only())
        .expect("search");
    assert!(
        hits.iter().any(|hit| hit.description.contains("Butter, stick")),
        "expected generic butter in results, got {:?}",
        hits.iter()
            .map(|hit| (&hit.description, &hit.data_type))
            .collect::<Vec<_>>()
    );
    assert!(
        !is_branded_food(&hits[0].data_type),
        "first hit should be a generic USDA food, got {:?}",
        hits[0]
    );
    assert!(
        hits.iter().all(|hit| !is_branded_food(&hit.data_type)),
        "generics-only path must not return branded rows: {:?}",
        hits
    );
}

#[test]
fn search_exclude_branded_skips_upc_products() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = SqliteNutritionCatalog::open(dir.path().join("fdc.sqlite")).unwrap();
    db.begin_import("test").unwrap();
    db.upsert_food(&branded_food(1, "BUTTER FLAVORED GOURMET POPCORN, BUTTER"))
        .unwrap();
    db.upsert_food(&foundation_food(2, "Butter, stick, salted"))
        .unwrap();
    db.finish_import().unwrap();

    let generics = db
        .search_foods("butter", 10, NutritionSearchOptions::generics_only())
        .unwrap();
    assert_eq!(generics.len(), 1);
    assert_eq!(generics[0].fdc_id, 2);

    let all = db
        .search_foods("butter", 10, NutritionSearchOptions::all())
        .unwrap();
    assert!(all.iter().any(|hit| hit.fdc_id == 1));
    assert!(all.iter().any(|hit| hit.fdc_id == 2));
}

#[test]
fn stores_and_reads_food() {
    let dir = tempfile::tempdir().unwrap();
    let mut db = SqliteNutritionCatalog::open(dir.path().join("fdc.sqlite")).unwrap();
    db.begin_import("test").unwrap();
    db.upsert_food(&FoodRecord {
        fdc_id: 1,
        data_type: "foundation_food".into(),
        description: "Test flour".into(),
        food_category_id: None,
        publication_date: None,
        brand_owner: None,
        brand_name: None,
        gtin_upc: None,
        ingredients: None,
        serving_size: None,
        serving_size_unit: None,
    })
    .unwrap();
    db.finish_import().unwrap();
    assert_eq!(db.food(1).unwrap().unwrap().description, "Test flour");
}
