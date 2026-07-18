use crate::seed::{needs_full_catalog, seed_minimal_catalog};
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

#[test]
fn needs_full_catalog_until_import_completes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("fdc.sqlite3");
    seed_minimal_catalog(&path).expect("seed");
    assert!(needs_full_catalog(&path));

    let connection = rusqlite::Connection::open(&path).expect("open");
    connection
        .execute(
            "UPDATE metadata SET value='2026-04' WHERE key='fdc_release'",
            [],
        )
        .expect("mark release");
    connection
        .execute(
            "UPDATE metadata SET value='false' WHERE key='import_complete'",
            [],
        )
        .expect("mark incomplete");
    assert!(needs_full_catalog(&path));

    connection
        .execute(
            "INSERT INTO metadata(key,value) VALUES('import_complete','true') ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [],
        )
        .expect("mark complete");
    assert!(!needs_full_catalog(&path));
}
