use crate::{
    ApplicationError, CalculateRecipeNutritionRequest, DocumentParser,
    LinkResourceNutritionRequest, NutritionCatalog, RecipeIngredientNutrition,
    RecipeNutritionResult, RecipeRepository, ResourceNutritionRepository, aggregate_nutrients,
    default_serving_context, ingredient_resources, nutrients_to_facts, resource_mass_grams,
    search_result_label,
};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct NutritionService {
    nutrition: Arc<dyn ResourceNutritionRepository>,
    recipes: Arc<dyn RecipeRepository>,
    parser: Arc<dyn DocumentParser>,
    catalog: Option<Arc<dyn NutritionCatalog>>,
}

impl NutritionService {
    pub fn new(
        nutrition: Arc<dyn ResourceNutritionRepository>,
        recipes: Arc<dyn RecipeRepository>,
        parser: Arc<dyn DocumentParser>,
        catalog: Option<Arc<dyn NutritionCatalog>>,
    ) -> Self {
        Self {
            nutrition,
            recipes,
            parser,
            catalog,
        }
    }

    pub fn catalog_available(&self) -> bool {
        self.catalog.is_some()
    }

    pub fn search_foods(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::NutritionSearchResult>, ApplicationError> {
        let catalog = self.catalog.as_ref().ok_or_else(Self::catalog_missing)?;
        if query.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "search query cannot be empty".to_owned(),
            ));
        }
        catalog.search_foods(query, limit.clamp(1, 50))
    }

    pub fn list_links(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<crate::ResourceNutritionLink>, ApplicationError> {
        self.nutrition.list_links_for_recipe(recipe_id)
    }

    pub fn link_resource(
        &self,
        recipe_id: Uuid,
        input: LinkResourceNutritionRequest,
    ) -> Result<crate::ResourceNutritionLink, ApplicationError> {
        if input.resource_symbol.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "resource symbol cannot be empty".to_owned(),
            ));
        }
        let catalog = self.catalog.as_ref().ok_or_else(Self::catalog_missing)?;
        let food = catalog
            .food(input.fdc_id)?
            .ok_or_else(|| ApplicationError::not_found("food"))?;
        let description = search_result_label(&crate::NutritionSearchResult {
            fdc_id: food.fdc_id,
            description: food.description,
            data_type: food.data_type,
            brand_owner: food.brand_owner,
            serving_size: food.serving_size,
            serving_size_unit: food.serving_size_unit,
        });
        self.nutrition.link_resource(recipe_id, input, description)
    }

    pub fn unlink_resource(
        &self,
        recipe_id: Uuid,
        resource_symbol: &str,
    ) -> Result<(), ApplicationError> {
        if self.nutrition.unlink_resource(recipe_id, resource_symbol)? {
            Ok(())
        } else {
            Err(ApplicationError::not_found("nutrition link"))
        }
    }

    pub fn calculate(
        &self,
        recipe_id: Uuid,
        request: CalculateRecipeNutritionRequest,
    ) -> Result<RecipeNutritionResult, ApplicationError> {
        let catalog = self.catalog.as_ref().ok_or_else(Self::catalog_missing)?;
        let document = self
            .recipes
            .get_recipe(recipe_id)?
            .ok_or_else(|| ApplicationError::not_found("recipe"))?;
        let recipe = self.parser.parse_recipe(&document.source_text)?;
        let links: HashMap<String, crate::ResourceNutritionLink> = self
            .nutrition
            .list_links_for_recipe(recipe_id)?
            .into_iter()
            .map(|link| (link.resource_symbol.clone(), link))
            .collect();

        let mut warnings = Vec::new();
        let mut ingredient_rows = Vec::new();
        let mut nutrient_inputs = Vec::new();
        let mut total_mass_grams = 0.0;
        let mut linked_count = 0usize;

        for resource in ingredient_resources(&recipe) {
            let mass_grams = resource_mass_grams(resource);
            let link = links.get(&resource.symbol);
            let resource_name = resource
                .properties
                .get("name")
                .and_then(|value| match value {
                    culinator_core::Value::Text(text) | culinator_core::Value::Symbol(text) => {
                        Some(text.clone())
                    }
                    _ => None,
                });

            if link.is_some() {
                linked_count += 1;
            }

            if mass_grams.is_none() {
                warnings.push(format!(
                    "{} has no mass quantity; skipped from totals",
                    resource.symbol
                ));
            }

            if link.is_none() {
                warnings.push(format!(
                    "{} is not linked to a food database entry",
                    resource.symbol
                ));
            }

            if let (Some(mass), Some(link)) = (mass_grams, link) {
                let nutrients = catalog.nutrients_for_food(link.fdc_id)?;
                total_mass_grams += mass;
                nutrient_inputs.push((mass, nutrients));
            }

            ingredient_rows.push(RecipeIngredientNutrition {
                resource_symbol: resource.symbol.clone(),
                resource_name,
                mass_grams,
                fdc_id: link.map(|value| value.fdc_id),
                food_description: link.map(|value| value.food_description.clone()),
                linked: link.is_some(),
            });
        }

        if linked_count == 0 {
            return Err(ApplicationError::InvalidInput(
                "link at least one ingredient to the nutrition database before calculating"
                    .to_owned(),
            ));
        }

        let totals = aggregate_nutrients(&nutrient_inputs);
        let (default_servings, default_serving_size, default_serving_mass) =
            default_serving_context(&recipe.servings);
        let servings = request
            .servings_per_container
            .unwrap_or(default_servings)
            .max(1.0);
        let serving_size = request.serving_size.unwrap_or(default_serving_size);
        let serving_size_grams = request.serving_size_grams.or(default_serving_mass);
        let facts = nutrients_to_facts(
            &totals,
            total_mass_grams,
            servings,
            &serving_size,
            serving_size_grams,
        );

        Ok(RecipeNutritionResult {
            facts,
            total_mass_grams,
            linked_ingredient_count: linked_count,
            total_ingredient_count: ingredient_rows.len(),
            ingredients: ingredient_rows,
            warnings,
        })
    }

    fn catalog_missing() -> ApplicationError {
        ApplicationError::InvalidInput(
            "nutrition database is not configured; build fdc.sqlite3 in the app data directory"
                .to_owned(),
        )
    }
}

#[cfg(test)]
#[path = "nutrition_service/test.rs"]
mod test;
