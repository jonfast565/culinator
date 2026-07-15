//! Streaming USDA FoodData Central CSV importer and SQLite nutrition catalog.

mod import;
mod store;

pub use import::{BuildOptions, BuildReport, DEFAULT_FULL_DOWNLOAD_URL, FdcDatabaseBuilder};
pub use store::SqliteNutritionCatalog;

#[cfg(test)]
mod test;
