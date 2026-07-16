use crate::ApplicationError;
use crate::models::{FoodNutrientRecord, FoodRecord, NutrientDefinition, NutritionSearchResult};

/// Read-side contract for interchangeable nutrition databases.
pub trait NutritionCatalog: Send + Sync {
    fn search_foods(
        &self,
        query: &str,
        limit: usize,
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
