use crate::{ApplicationError, LinkResourceNutritionRequest, ResourceNutritionLink};
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
}

#[cfg(test)]
#[path = "resource_nutrition/test.rs"]
mod test;
