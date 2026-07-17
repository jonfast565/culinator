use crate::seed_minimal_catalog;
use crate::store::SqliteNutritionCatalog;
use culinator_models::NutritionCatalog;

#[test]
fn seed_minimal_catalog_is_searchable() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("fdc.sqlite3");
    seed_minimal_catalog(&path).expect("seed");
    let catalog = SqliteNutritionCatalog::open(&path).expect("open");
    let hits = catalog.search_foods("flour", 5).expect("search");
    assert!(!hits.is_empty());
}
