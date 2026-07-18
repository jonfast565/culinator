use crate::{
    ApplicationError, AutoLinkRequest, AutoLinkResult, CalculateRecipeNutritionRequest,
    DocumentParser, FuzzyFoodMatch, FuzzyMatchRequest, IngredientMatchSuggestion,
    LinkResourceNutritionRequest, NutritionCatalog, RecipeIngredientNutrition,
    RecipeNutritionResult, RecipeNutritionState, RecipeRepository, ResourceNutritionRepository,
    SaveIngredientManualNutritionRequest, SaveRecipeNutritionRequest, aggregate_nutrients,
    default_serving_context, fts_query_from_ingredient, ingredient_resources,
    manual_facts_to_nutrients, nutrients_to_facts, rank_fuzzy_matches, resource_mass_grams,
    search_result_label,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Clone)]
pub struct NutritionService {
    nutrition: Arc<dyn ResourceNutritionRepository>,
    recipes: Arc<dyn RecipeRepository>,
    parser: Arc<dyn DocumentParser>,
    catalog: Arc<RwLock<Option<Arc<dyn NutritionCatalog>>>>,
}

impl NutritionService {
    pub fn new(
        nutrition: Arc<dyn ResourceNutritionRepository>,
        recipes: Arc<dyn RecipeRepository>,
        parser: Arc<dyn DocumentParser>,
        catalog: Arc<RwLock<Option<Arc<dyn NutritionCatalog>>>>,
    ) -> Self {
        Self {
            nutrition,
            recipes,
            parser,
            catalog,
        }
    }

    pub fn set_catalog(&self, catalog: Option<Arc<dyn NutritionCatalog>>) {
        *self
            .catalog
            .write()
            .expect("nutrition catalog lock poisoned") = catalog;
    }

    pub fn catalog_available(&self) -> bool {
        self.catalog
            .read()
            .expect("nutrition catalog lock poisoned")
            .is_some()
    }

    fn catalog(&self) -> Result<Arc<dyn NutritionCatalog>, ApplicationError> {
        self.catalog
            .read()
            .expect("nutrition catalog lock poisoned")
            .clone()
            .ok_or_else(Self::catalog_missing)
    }

    pub fn search_foods(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<crate::NutritionSearchResult>, ApplicationError> {
        let catalog = self.catalog()?;
        if query.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "search query cannot be empty".to_owned(),
            ));
        }
        catalog.search_foods(query, limit.clamp(1, 50))
    }

    pub fn fuzzy_match(
        &self,
        request: FuzzyMatchRequest,
    ) -> Result<Vec<FuzzyFoodMatch>, ApplicationError> {
        let catalog = self.catalog()?;
        if request.query.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "search query cannot be empty".to_owned(),
            ));
        }
        let fts_query = fts_query_from_ingredient(&request.query);
        let results = catalog.search_foods(&fts_query, request.limit.clamp(1, 50).max(10))?;
        Ok(rank_fuzzy_matches(&request.query, &results)
            .into_iter()
            .take(request.limit.clamp(1, 50))
            .collect())
    }

    pub fn get_state(&self, recipe_id: Uuid) -> Result<RecipeNutritionState, ApplicationError> {
        self.nutrition.get_recipe_nutrition(recipe_id)
    }

    pub fn save_recipe_nutrition(
        &self,
        recipe_id: Uuid,
        input: SaveRecipeNutritionRequest,
    ) -> Result<RecipeNutritionState, ApplicationError> {
        if input.manual_override && input.facts.is_none() {
            return Err(ApplicationError::InvalidInput(
                "manual override requires nutrition facts".to_owned(),
            ));
        }
        self.nutrition.save_recipe_nutrition(recipe_id, input)
    }

    pub fn save_manual_ingredient(
        &self,
        recipe_id: Uuid,
        input: SaveIngredientManualNutritionRequest,
    ) -> Result<crate::IngredientManualNutrition, ApplicationError> {
        self.nutrition
            .save_manual_ingredient_nutrition(recipe_id, input)
    }

    pub fn delete_manual_ingredient(
        &self,
        recipe_id: Uuid,
        resource_symbol: &str,
    ) -> Result<(), ApplicationError> {
        if self
            .nutrition
            .delete_manual_ingredient_nutrition(recipe_id, resource_symbol)?
        {
            Ok(())
        } else {
            Err(ApplicationError::not_found("manual ingredient nutrition"))
        }
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
        let catalog = self.catalog()?;
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

    pub fn auto_link(
        &self,
        recipe_id: Uuid,
        request: AutoLinkRequest,
    ) -> Result<AutoLinkResult, ApplicationError> {
        self.catalog()?;
        let document = self
            .recipes
            .get_recipe(recipe_id)?
            .ok_or_else(|| ApplicationError::not_found("recipe"))?;
        let recipe = self.parser.parse_recipe(&document.source_text)?;
        let existing: HashMap<String, crate::ResourceNutritionLink> = self
            .nutrition
            .list_links_for_recipe(recipe_id)?
            .into_iter()
            .map(|link| (link.resource_symbol.clone(), link))
            .collect();

        let mut linked = Vec::new();
        let mut skipped = Vec::new();
        let mut suggestions = Vec::new();

        for resource in ingredient_resources(&recipe) {
            if existing.contains_key(&resource.symbol) {
                skipped.push(resource.symbol.clone());
                continue;
            }
            let resource_name = resource_display_name(resource);
            let query = resource_name
                .clone()
                .unwrap_or_else(|| resource.symbol.clone());
            let matches = self.fuzzy_match(FuzzyMatchRequest {
                query: query.clone(),
                limit: 3,
            })?;
            let best = matches.into_iter().next();
            suggestions.push(IngredientMatchSuggestion {
                resource_symbol: resource.symbol.clone(),
                resource_name: resource_name.clone(),
                best_match: best.clone(),
            });
            if request.dry_run {
                continue;
            }
            if let Some(best_match) = best.filter(|item| item.score >= request.min_score) {
                let link = self.nutrition.link_resource(
                    recipe_id,
                    LinkResourceNutritionRequest {
                        resource_symbol: resource.symbol.clone(),
                        fdc_id: best_match.result.fdc_id,
                    },
                    search_result_label(&best_match.result),
                )?;
                linked.push(link);
            } else {
                skipped.push(resource.symbol.clone());
            }
        }

        Ok(AutoLinkResult {
            linked,
            skipped,
            suggestions,
        })
    }

    pub fn calculate(
        &self,
        recipe_id: Uuid,
        request: CalculateRecipeNutritionRequest,
    ) -> Result<RecipeNutritionResult, ApplicationError> {
        let state = self.nutrition.get_recipe_nutrition(recipe_id)?;
        if state.manual_override {
            let facts = state.manual_facts.clone().ok_or_else(|| {
                ApplicationError::InvalidInput(
                    "recipe manual override is enabled but no facts are saved".to_owned(),
                )
            })?;
            return Ok(RecipeNutritionResult {
                facts,
                total_mass_grams: 0.0,
                linked_ingredient_count: 0,
                total_ingredient_count: 0,
                ingredients: vec![],
                warnings: vec!["Using saved recipe-level manual nutrition override.".to_owned()],
                manual_override: true,
                calculated: false,
            });
        }

        let catalog = self.catalog()?;
        let document = self
            .recipes
            .get_recipe(recipe_id)?
            .ok_or_else(|| ApplicationError::not_found("recipe"))?;
        let recipe = self.parser.parse_recipe(&document.source_text)?;
        let links: HashMap<String, crate::ResourceNutritionLink> = state
            .links
            .into_iter()
            .map(|link| (link.resource_symbol.clone(), link))
            .collect();
        let manual: HashMap<String, crate::IngredientManualNutrition> = state
            .manual_ingredients
            .into_iter()
            .map(|entry| (entry.resource_symbol.clone(), entry))
            .collect();

        let mut warnings = Vec::new();
        let mut ingredient_rows = Vec::new();
        let mut nutrient_inputs = Vec::new();
        let mut total_mass_grams = 0.0;
        let mut linked_count = 0usize;
        let mut sourced_count = 0usize;

        for resource in ingredient_resources(&recipe) {
            let mass_grams = resource_mass_grams(resource);
            let link = links.get(&resource.symbol);
            let manual_entry = manual.get(&resource.symbol);
            let resource_name = resource_display_name(resource);
            let has_fdc = link.is_some();
            let has_manual = manual_entry.is_some();

            if has_fdc || has_manual {
                linked_count += 1;
            }

            if mass_grams.is_none() {
                warnings.push(format!(
                    "{} has no mass quantity; skipped from totals",
                    resource.symbol
                ));
            }

            if !has_fdc && !has_manual {
                warnings.push(format!(
                    "{} is not linked to a food database entry or manual facts",
                    resource.symbol
                ));
            }

            if let Some(mass) = mass_grams {
                if let Some(link) = link {
                    let nutrients = catalog.nutrients_for_food(link.fdc_id)?;
                    total_mass_grams += mass;
                    nutrient_inputs.push((mass, nutrients));
                    sourced_count += 1;
                } else if let Some(manual_entry) = manual_entry {
                    let nutrients = manual_facts_to_nutrients(&manual_entry.facts_per_100g);
                    if nutrients.is_empty() {
                        warnings.push(format!(
                            "{} has manual nutrition entry but no nutrient values",
                            resource.symbol
                        ));
                    } else {
                        total_mass_grams += mass;
                        nutrient_inputs.push((mass, nutrients));
                        sourced_count += 1;
                    }
                }
            }

            ingredient_rows.push(RecipeIngredientNutrition {
                resource_symbol: resource.symbol.clone(),
                resource_name,
                mass_grams,
                fdc_id: link.map(|value| value.fdc_id),
                food_description: link.map(|value| value.food_description.clone()),
                linked: has_fdc,
                manual: has_manual,
            });
        }

        if sourced_count == 0 {
            return Err(ApplicationError::InvalidInput(
                "link ingredients or enter manual nutrition facts before calculating".to_owned(),
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
            manual_override: false,
            calculated: true,
        })
    }

    fn catalog_missing() -> ApplicationError {
        ApplicationError::InvalidInput(
            "nutrition database is not configured; build fdc.sqlite3 in the app data directory"
                .to_owned(),
        )
    }
}

fn resource_display_name(resource: &culinator_core::Resource) -> Option<String> {
    resource
        .properties
        .get("name")
        .and_then(|value| match value {
            culinator_core::Value::Text(text) | culinator_core::Value::Symbol(text) => {
                Some(text.clone())
            }
            _ => None,
        })
}

#[cfg(test)]
#[path = "nutrition_service/test.rs"]
mod test;
