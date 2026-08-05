use crate::{
    CalculateRecipeNutritionRequest, DocumentParser, FoodNutrientRecord, FoodRecord,
    LinkResourceNutritionRequest, NutritionCatalog, NutritionSearchResult, NutritionService,
    RecipeDocument, RecipeRepository, ResourceNutritionLink, ResourceNutritionRepository,
    FDC_ENERGY_KCAL, FDC_PROTEIN,
};
use culinator_core::{Document, Recipe};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;

struct MockRecipes {
    document: RecipeDocument,
}

impl RecipeRepository for MockRecipes {
    fn list_recipes(&self) -> Result<Vec<crate::RecipeSummary>, crate::ApplicationError> {
        unimplemented!()
    }
    fn get_recipe(
        &self,
        id: Uuid,
    ) -> Result<Option<RecipeDocument>, crate::ApplicationError> {
        if id == self.document.id {
            Ok(Some(self.document.clone()))
        } else {
            Ok(None)
        }
    }
    fn create_recipe(
        &self,
        _: crate::NewRecipe,
    ) -> Result<RecipeDocument, crate::ApplicationError> {
        unimplemented!()
    }
    fn save_recipe(
        &self,
        _: &Recipe,
        _: &str,
    ) -> Result<RecipeDocument, crate::ApplicationError> {
        unimplemented!()
    }
    fn delete_recipe(&self, _: Uuid) -> Result<bool, crate::ApplicationError> {
        unimplemented!()
    }
    fn move_recipe(
        &self,
        _: Uuid,
        _: Option<Uuid>,
        _: i64,
    ) -> Result<bool, crate::ApplicationError> {
        unimplemented!()
    }
}

struct MockParser {
    recipe: Recipe,
}

impl DocumentParser for MockParser {
    fn parse_document(&self, _: &str) -> Result<Document, crate::ApplicationError> {
        Ok(Document::Recipe {
            recipe: self.recipe.clone(),
        })
    }
}

struct MockCatalog;

impl NutritionCatalog for MockCatalog {
    fn search_foods(
        &self,
        query: &str,
        _limit: usize,
        _options: crate::NutritionSearchOptions,
    ) -> Result<Vec<NutritionSearchResult>, crate::ApplicationError> {
        let lowered = query.to_ascii_lowercase();
        let mut hits = Vec::new();
        if lowered.contains("flour") || lowered.contains("wheat") {
            hits.push(NutritionSearchResult {
                fdc_id: 100,
                description: "Wheat flour, white, all-purpose".to_owned(),
                data_type: "foundation_food".to_owned(),
                brand_owner: None,
                serving_size: None,
                serving_size_unit: None,
            });
        }
        if lowered.contains("egg") {
            hits.push(NutritionSearchResult {
                fdc_id: 101,
                description: "Egg, whole, raw".to_owned(),
                data_type: "foundation_food".to_owned(),
                brand_owner: None,
                serving_size: None,
                serving_size_unit: None,
            });
        }
        Ok(hits)
    }

    fn food(&self, fdc_id: i64) -> Result<Option<FoodRecord>, crate::ApplicationError> {
        Ok(Some(FoodRecord {
            fdc_id,
            data_type: "foundation_food".to_owned(),
            description: format!("Food {fdc_id}"),
            food_category_id: None,
            publication_date: None,
            brand_owner: None,
            brand_name: None,
            gtin_upc: None,
            ingredients: None,
            serving_size: None,
            serving_size_unit: None,
        }))
    }

    fn nutrients_for_food(
        &self,
        fdc_id: i64,
    ) -> Result<Vec<FoodNutrientRecord>, crate::ApplicationError> {
        let protein = if fdc_id == 101 { 12.0 } else { 10.0 };
        Ok(vec![
            FoodNutrientRecord {
                id: None,
                fdc_id,
                nutrient_id: FDC_PROTEIN,
                amount: Some(protein),
                data_points: None,
                derivation_id: None,
                min: None,
                max: None,
                median: None,
            },
            FoodNutrientRecord {
                id: None,
                fdc_id,
                nutrient_id: FDC_ENERGY_KCAL,
                amount: Some(364.0),
                data_points: None,
                derivation_id: None,
                min: None,
                max: None,
                median: None,
            },
        ])
    }
}

#[derive(Default)]
struct MockNutritionRepo {
    links: Mutex<HashMap<(Uuid, String), ResourceNutritionLink>>,
}

impl ResourceNutritionRepository for MockNutritionRepo {
    fn list_links_for_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<ResourceNutritionLink>, crate::ApplicationError> {
        Ok(self
            .links
            .lock()
            .unwrap()
            .values()
            .filter(|link| link.recipe_id == recipe_id)
            .cloned()
            .collect())
    }

    fn get_link(
        &self,
        recipe_id: Uuid,
        resource_symbol: &str,
    ) -> Result<Option<ResourceNutritionLink>, crate::ApplicationError> {
        Ok(self
            .links
            .lock()
            .unwrap()
            .get(&(recipe_id, resource_symbol.to_owned()))
            .cloned())
    }

    fn link_resource(
        &self,
        recipe_id: Uuid,
        input: LinkResourceNutritionRequest,
        food_description: String,
    ) -> Result<ResourceNutritionLink, crate::ApplicationError> {
        let link = ResourceNutritionLink {
            recipe_id,
            resource_symbol: input.resource_symbol.clone(),
            fdc_id: input.fdc_id,
            food_description,
            linked_at: "now".to_owned(),
        };
        self.links
            .lock()
            .unwrap()
            .insert((recipe_id, input.resource_symbol), link.clone());
        Ok(link)
    }

    fn unlink_resource(
        &self,
        recipe_id: Uuid,
        resource_symbol: &str,
    ) -> Result<bool, crate::ApplicationError> {
        Ok(self
            .links
            .lock()
            .unwrap()
            .remove(&(recipe_id, resource_symbol.to_owned()))
            .is_some())
    }

    fn get_recipe_nutrition(
        &self,
        recipe_id: Uuid,
    ) -> Result<crate::RecipeNutritionState, crate::ApplicationError> {
        Ok(crate::RecipeNutritionState {
            recipe_id,
            links: self.list_links_for_recipe(recipe_id)?,
            manual_ingredients: vec![],
            manual_override: false,
            manual_facts: None,
        })
    }

    fn save_recipe_nutrition(
        &self,
        _: Uuid,
        _: crate::SaveRecipeNutritionRequest,
    ) -> Result<crate::RecipeNutritionState, crate::ApplicationError> {
        unimplemented!()
    }

    fn list_manual_ingredient_nutrition(
        &self,
        _: Uuid,
    ) -> Result<Vec<crate::IngredientManualNutrition>, crate::ApplicationError> {
        Ok(vec![])
    }

    fn save_manual_ingredient_nutrition(
        &self,
        _: Uuid,
        _: crate::SaveIngredientManualNutritionRequest,
    ) -> Result<crate::IngredientManualNutrition, crate::ApplicationError> {
        unimplemented!()
    }

    fn delete_manual_ingredient_nutrition(
        &self,
        _: Uuid,
        _: &str,
    ) -> Result<bool, crate::ApplicationError> {
        Ok(false)
    }
}

fn sample_recipe(id: Uuid) -> Recipe {
    use culinator_core::{Dimension, Resource, ResourceKind, TypeRef, Value};
    let mut flour_props = std::collections::BTreeMap::new();
    flour_props.insert(
        "quantity".to_owned(),
        Value::Quantity(culinator_core::Quantity {
            value: 1.0,
            unit: "cup".to_owned(),
            dimension: Dimension::Volume,
        }),
    );
    flour_props.insert(
        "name".to_owned(),
        Value::Text("all-purpose flour".to_owned()),
    );
    let mut egg_props = std::collections::BTreeMap::new();
    egg_props.insert(
        "quantity".to_owned(),
        Value::Quantity(culinator_core::Quantity {
            value: 1.0,
            unit: "count".to_owned(),
            dimension: Dimension::Count,
        }),
    );
    Recipe {
        id,
        book_id: None,
        symbol: "test_pancakes".to_owned(),
        declared_type: TypeRef::named("Recipe"),
        title: "Test Pancakes".to_owned(),
        protocol_version: "0.3".to_owned(),
        types: vec![],
        resources: vec![
            Resource {
                id: Uuid::new_v4(),
                symbol: "flour".to_owned(),
                declared_type: TypeRef::named("Ingredient"),
                kind: ResourceKind::Ingredient,
                optional: false,
                divided: false,
                substitutes: vec![],
                to_taste: false,
                size: None,
                variant: None,
                notes: vec![],
                properties: flour_props,
                span: None,
            },
            Resource {
                id: Uuid::new_v4(),
                symbol: "egg".to_owned(),
                declared_type: TypeRef::named("Ingredient"),
                kind: ResourceKind::Ingredient,
                optional: false,
                divided: false,
                substitutes: vec![],
                to_taste: false,
                size: Some("large".to_owned()),
                variant: None,
                notes: vec![],
                properties: egg_props,
                span: None,
            },
        ],
        processes: vec![],
        operations: vec![],
        servings: vec![],
        formulas: vec![],
        yields: vec![],
        properties: Default::default(),
    }
}

#[test]
fn calculate_auto_links_and_returns_facts() {
    let recipe_id = Uuid::new_v4();
    let recipe = sample_recipe(recipe_id);
    let document = RecipeDocument {
        id: recipe_id,
        book_id: None,
        symbol: recipe.symbol.clone(),
        title: recipe.title.clone(),
        protocol_version: recipe.protocol_version.clone(),
        source_text: "unused".to_owned(),
        updated_at: "now".to_owned(),
    };
    let service = NutritionService::new(
        Arc::new(MockNutritionRepo::default()),
        Arc::new(MockRecipes { document }),
        Arc::new(MockParser { recipe }),
        Arc::new(RwLock::new(Some(Arc::new(MockCatalog) as Arc<dyn NutritionCatalog>))),
    );

    let result = service
        .calculate(
            recipe_id,
            CalculateRecipeNutritionRequest {
                servings_per_container: Some(2.0),
                serving_size: Some("1 pancake".to_owned()),
                serving_size_grams: None,
            },
        )
        .expect("calculate");

    assert!(result.calculated);
    assert!(result.linked_ingredient_count >= 1);
    assert!(result.total_mass_grams > 0.0);
    assert!(result.facts.protein_grams > 0.0);
    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.contains("Auto-linked")),
        "warnings: {:?}",
        result.warnings
    );
}

#[test]
fn calculate_soft_fails_when_nothing_matches() {
    struct EmptyCatalog;
    impl NutritionCatalog for EmptyCatalog {
        fn search_foods(
            &self,
            _: &str,
            _: usize,
            _: crate::NutritionSearchOptions,
        ) -> Result<Vec<NutritionSearchResult>, crate::ApplicationError> {
            Ok(vec![])
        }
        fn food(&self, _: i64) -> Result<Option<FoodRecord>, crate::ApplicationError> {
            Ok(None)
        }
        fn nutrients_for_food(
            &self,
            _: i64,
        ) -> Result<Vec<FoodNutrientRecord>, crate::ApplicationError> {
            Ok(vec![])
        }
    }

    let recipe_id = Uuid::new_v4();
    let recipe = sample_recipe(recipe_id);
    let document = RecipeDocument {
        id: recipe_id,
        book_id: None,
        symbol: recipe.symbol.clone(),
        title: recipe.title.clone(),
        protocol_version: recipe.protocol_version.clone(),
        source_text: "unused".to_owned(),
        updated_at: "now".to_owned(),
    };
    let service = NutritionService::new(
        Arc::new(MockNutritionRepo::default()),
        Arc::new(MockRecipes { document }),
        Arc::new(MockParser { recipe }),
        Arc::new(RwLock::new(Some(
            Arc::new(EmptyCatalog) as Arc<dyn NutritionCatalog>
        ))),
    );

    let result = service
        .calculate(
            recipe_id,
            CalculateRecipeNutritionRequest {
                servings_per_container: None,
                serving_size: None,
                serving_size_grams: None,
            },
        )
        .expect("calculate should soft-fail");

    assert!(result.calculated);
    assert_eq!(result.linked_ingredient_count, 0);
    assert_eq!(result.facts.calories, 0.0);
    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.contains("No ingredients contributed")),
        "warnings: {:?}",
        result.warnings
    );
}
