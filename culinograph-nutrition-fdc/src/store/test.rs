use super::SqliteNutritionCatalog;
use culinograph_models::{FoodRecord, NutritionCatalog, NutritionImportStore};
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
