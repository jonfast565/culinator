//! Streaming USDA FoodData Central CSV importer and SQLite nutrition catalog.

mod import;
mod store;

pub use import::{BuildOptions, BuildReport, FdcDatabaseBuilder, DEFAULT_FULL_DOWNLOAD_URL};
pub use store::SqliteNutritionCatalog;

#[cfg(test)]
mod test;
