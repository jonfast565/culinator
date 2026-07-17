use crate::{
    ApplicationError, IngredientManualNutrition, LinkResourceNutritionRequest, RecipeNutritionState,
    ResourceNutritionLink, SaveIngredientManualNutritionRequest, SaveRecipeNutritionRequest,
};
use uuid::Uuid;

pub trait ResourceNutritionRepository: Send + Sync {
    fn list_links_for_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<ResourceNutritionLink>, ApplicationError>;

    fn get_link(
        &self,
        recipe_id: Uuid,
        resource_symbol: &str,
    ) -> Result<Option<ResourceNutritionLink>, ApplicationError>;

    fn link_resource(
        &self,
        recipe_id: Uuid,
        input: LinkResourceNutritionRequest,
        food_description: String,
    ) -> Result<ResourceNutritionLink, ApplicationError>;

    fn unlink_resource(
        &self,
        recipe_id: Uuid,
        resource_symbol: &str,
    ) -> Result<bool, ApplicationError>;

    fn get_recipe_nutrition(
        &self,
        recipe_id: Uuid,
    ) -> Result<RecipeNutritionState, ApplicationError>;

    fn save_recipe_nutrition(
        &self,
        recipe_id: Uuid,
        input: SaveRecipeNutritionRequest,
    ) -> Result<RecipeNutritionState, ApplicationError>;

    fn list_manual_ingredient_nutrition(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<IngredientManualNutrition>, ApplicationError>;

    fn save_manual_ingredient_nutrition(
        &self,
        recipe_id: Uuid,
        input: SaveIngredientManualNutritionRequest,
    ) -> Result<IngredientManualNutrition, ApplicationError>;

    fn delete_manual_ingredient_nutrition(
        &self,
        recipe_id: Uuid,
        resource_symbol: &str,
    ) -> Result<bool, ApplicationError>;
}

#[cfg(test)]
#[path = "resource_nutrition/test.rs"]
mod test;
