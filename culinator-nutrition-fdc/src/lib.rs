//! Streaming USDA FoodData Central CSV importer and SQLite nutrition catalog.

mod import;
mod seed;
mod store;

pub use import::{
    BuildOptions, BuildReport, DEFAULT_FULL_DOWNLOAD_URL, FdcDatabaseBuilder, download_and_build,
};
pub use seed::{catalog_release, needs_full_catalog, seed_minimal_catalog};
pub use store::SqliteNutritionCatalog;

#[cfg(test)]
mod test;

#[cfg(test)]
#[path = "seed/test.rs"]
mod seed_test;
