//! Streaming USDA FoodData Central CSV importer and SQLite nutrition catalog.

mod import;
mod seed;
mod store;

pub use import::{BuildOptions, BuildReport, DEFAULT_FULL_DOWNLOAD_URL, FdcDatabaseBuilder};
pub use seed::seed_minimal_catalog;
pub use store::SqliteNutritionCatalog;

#[cfg(test)]
mod test;

#[cfg(test)]
#[path = "seed/test.rs"]
mod seed_test;
