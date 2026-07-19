//! Minimal starter nutrition catalog for first launch when the full USDA
//! archive has not been built yet.

use crate::store::SqliteNutritionCatalog;
use culinator_models::{FoodNutrientRecord, FoodRecord, NutrientDefinition, NutritionImportStore};
use rusqlite::{Connection, OptionalExtension};
use std::path::Path;

const STARTER_RELEASE: &str = "starter";

const STARTER_FOODS: &[(&str, i64)] = &[
    ("All-purpose flour", 20001),
    ("Butter, salted", 20002),
    ("Egg, whole, raw", 20003),
    ("Granulated sugar", 20004),
    ("Milk, whole", 20005),
    ("Olive oil", 20006),
    ("Salt, table", 20007),
    ("Water, tap", 20008),
    ("Chicken breast, raw", 20009),
    ("Tomatoes, raw", 20010),
];

/// USDA-style nutrient ids used in starter rows (Energy, Protein, Fat, Carbs).
const NUTRIENT_ENERGY: i64 = 1008;
const NUTRIENT_PROTEIN: i64 = 1003;
const NUTRIENT_FAT: i64 = 1004;
const NUTRIENT_CARBS: i64 = 1005;

pub fn catalog_release(path: impl AsRef<Path>) -> Option<String> {
    metadata_value(path, "fdc_release")
}

fn metadata_value(path: impl AsRef<Path>, key: &str) -> Option<String> {
    let connection = Connection::open(path).ok()?;
    connection
        .query_row("SELECT value FROM metadata WHERE key=?1", [key], |row| {
            row.get(0)
        })
        .optional()
        .ok()?
}

pub fn needs_full_catalog(path: impl AsRef<Path>) -> bool {
    match catalog_release(&path) {
        None => true,
        Some(release) if release == STARTER_RELEASE => true,
        Some(_) => metadata_value(path, "import_complete").as_deref() != Some("true"),
    }
}

pub fn seed_minimal_catalog(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut store = SqliteNutritionCatalog::open(path)?;
    store.begin_import(STARTER_RELEASE)?;
    for nutrient in [
        NutrientDefinition {
            id: NUTRIENT_ENERGY,
            number: Some("208".to_owned()),
            name: "Energy".to_owned(),
            unit_name: "kcal".to_owned(),
            rank: Some(100),
        },
        NutrientDefinition {
            id: NUTRIENT_PROTEIN,
            number: Some("203".to_owned()),
            name: "Protein".to_owned(),
            unit_name: "g".to_owned(),
            rank: Some(200),
        },
        NutrientDefinition {
            id: NUTRIENT_FAT,
            number: Some("204".to_owned()),
            name: "Total lipid (fat)".to_owned(),
            unit_name: "g".to_owned(),
            rank: Some(300),
        },
        NutrientDefinition {
            id: NUTRIENT_CARBS,
            number: Some("205".to_owned()),
            name: "Carbohydrate, by difference".to_owned(),
            unit_name: "g".to_owned(),
            rank: Some(400),
        },
    ] {
        store.upsert_nutrient(&nutrient)?;
    }

    for (index, (description, fdc_id)) in STARTER_FOODS.iter().enumerate() {
        store.upsert_food(&FoodRecord {
            fdc_id: *fdc_id,
            data_type: "foundation_food".to_owned(),
            description: (*description).to_owned(),
            food_category_id: None,
            publication_date: None,
            brand_owner: None,
            brand_name: None,
            gtin_upc: None,
            ingredients: None,
            serving_size: Some(100.0),
            serving_size_unit: Some("g".to_owned()),
        })?;
        let energy = 50.0 + (index as f64 * 17.0);
        for (nutrient_id, amount) in [
            (NUTRIENT_ENERGY, energy),
            (NUTRIENT_PROTEIN, 3.0 + index as f64 * 0.4),
            (NUTRIENT_FAT, 1.0 + index as f64 * 0.3),
            (NUTRIENT_CARBS, 5.0 + index as f64 * 0.5),
        ] {
            store.upsert_food_nutrient(&FoodNutrientRecord {
                id: Some(fdc_id * 10 + nutrient_id),
                fdc_id: *fdc_id,
                nutrient_id,
                amount: Some(amount),
                data_points: None,
                derivation_id: None,
                min: None,
                max: None,
                median: None,
            })?;
        }
    }
    store.finish_import()?;
    Ok(())
}
