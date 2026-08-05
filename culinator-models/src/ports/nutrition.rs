use crate::ApplicationError;
use crate::models::{FoodNutrientRecord, FoodRecord, NutrientDefinition, NutritionSearchResult};

/// Controls how [`NutritionCatalog::search_foods`] scans the dataset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NutritionSearchOptions {
    /// When true, skip Branded Foods rows entirely (fast path for ingredient matching).
    pub exclude_branded: bool,
}

impl NutritionSearchOptions {
    pub const fn generics_only() -> Self {
        Self {
            exclude_branded: true,
        }
    }

    pub const fn all() -> Self {
        Self {
            exclude_branded: false,
        }
    }
}

/// Read-side contract for interchangeable nutrition databases.
pub trait NutritionCatalog: Send + Sync {
    fn search_foods(
        &self,
        query: &str,
        limit: usize,
        options: NutritionSearchOptions,
    ) -> Result<Vec<NutritionSearchResult>, ApplicationError>;
    fn food(&self, fdc_id: i64) -> Result<Option<FoodRecord>, ApplicationError>;
    fn nutrients_for_food(&self, fdc_id: i64) -> Result<Vec<FoodNutrientRecord>, ApplicationError>;
}

/// Write-side contract used by bulk dataset builders.
pub trait NutritionImportStore {
    fn begin_import(&mut self, release: &str) -> Result<(), ApplicationError>;
    fn upsert_nutrient(&mut self, nutrient: &NutrientDefinition) -> Result<(), ApplicationError>;
    fn upsert_food(&mut self, food: &FoodRecord) -> Result<(), ApplicationError>;
    fn upsert_food_nutrient(&mut self, value: &FoodNutrientRecord) -> Result<(), ApplicationError>;
    fn finish_import(&mut self) -> Result<(), ApplicationError>;
}

#[cfg(test)]
mod test;
