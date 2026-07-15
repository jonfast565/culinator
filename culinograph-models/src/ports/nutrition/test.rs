use crate::{FoodRecord, NutritionCatalog};

fn assert_object_safe(_: &dyn NutritionCatalog) {}

#[test]
fn nutrition_catalog_is_object_safe() {
    struct Empty;
    impl NutritionCatalog for Empty {
        fn search_foods(&self, _: &str, _: usize) -> Result<Vec<crate::NutritionSearchResult>, crate::ApplicationError> { Ok(vec![]) }
        fn food(&self, _: i64) -> Result<Option<FoodRecord>, crate::ApplicationError> { Ok(None) }
        fn nutrients_for_food(&self, _: i64) -> Result<Vec<crate::FoodNutrientRecord>, crate::ApplicationError> { Ok(vec![]) }
    }
    assert_object_safe(&Empty);
}
