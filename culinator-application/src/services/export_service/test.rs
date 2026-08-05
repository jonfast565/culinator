use super::*;
use crate::{
    LinkResourceNutritionRequest, NutritionCatalog, NutritionSearchResult, NutritionService,
    RecipeDocument, RecipeExporter, RecipeRepository, ResourceNutritionLink,
    ResourceNutritionRepository,
};
use culinator_core::{Dimension, Recipe, Resource, ResourceKind, TypeRef, Value};
use culinator_models::{BookExportOptions, NutritionFacts, RecipeExportFormat, RecipeExportOptions};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, RwLock};
use uuid::Uuid;

struct MockRecipes {
    document: RecipeDocument,
}

impl RecipeRepository for MockRecipes {
    fn list_recipes(&self) -> Result<Vec<crate::RecipeSummary>, ApplicationError> {
        unimplemented!()
    }

    fn get_recipe(&self, id: Uuid) -> Result<Option<RecipeDocument>, ApplicationError> {
        if id == self.document.id {
            Ok(Some(self.document.clone()))
        } else {
            Ok(None)
        }
    }

    fn create_recipe(&self, _: crate::NewRecipe) -> Result<RecipeDocument, ApplicationError> {
        unimplemented!()
    }

    fn save_recipe(&self, _: &Recipe, _: &str) -> Result<RecipeDocument, ApplicationError> {
        unimplemented!()
    }

    fn delete_recipe(&self, _: Uuid) -> Result<bool, ApplicationError> {
        unimplemented!()
    }

    fn move_recipe(
        &self,
        _: Uuid,
        _: Option<Uuid>,
        _: i64,
    ) -> Result<bool, ApplicationError> {
        unimplemented!()
    }
}

struct MockParser {
    recipe: Recipe,
}

impl DocumentParser for MockParser {
    fn parse_document(&self, _: &str) -> Result<culinator_core::Document, ApplicationError> {
        Ok(culinator_core::Document::Recipe {
            recipe: self.recipe.clone(),
        })
    }
}

struct LinkedNutritionRepo {
    links: Mutex<HashMap<(Uuid, String), ResourceNutritionLink>>,
}

impl LinkedNutritionRepo {
    fn with_link(recipe_id: Uuid, resource_symbol: &str, fdc_id: i64) -> Self {
        let link = ResourceNutritionLink {
            recipe_id,
            resource_symbol: resource_symbol.to_owned(),
            fdc_id,
            food_description: "Wheat flour".to_owned(),
            linked_at: "now".to_owned(),
        };
        let mut links = HashMap::new();
        links.insert((recipe_id, resource_symbol.to_owned()), link);
        Self {
            links: Mutex::new(links),
        }
    }
}

impl ResourceNutritionRepository for LinkedNutritionRepo {
    fn list_links_for_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<ResourceNutritionLink>, ApplicationError> {
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
    ) -> Result<Option<ResourceNutritionLink>, ApplicationError> {
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
    ) -> Result<ResourceNutritionLink, ApplicationError> {
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
    ) -> Result<bool, ApplicationError> {
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
    ) -> Result<crate::RecipeNutritionState, ApplicationError> {
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
    ) -> Result<crate::RecipeNutritionState, ApplicationError> {
        unimplemented!()
    }

    fn list_manual_ingredient_nutrition(
        &self,
        _: Uuid,
    ) -> Result<Vec<crate::IngredientManualNutrition>, ApplicationError> {
        Ok(vec![])
    }

    fn save_manual_ingredient_nutrition(
        &self,
        _: Uuid,
        _: crate::SaveIngredientManualNutritionRequest,
    ) -> Result<crate::IngredientManualNutrition, ApplicationError> {
        unimplemented!()
    }

    fn delete_manual_ingredient_nutrition(
        &self,
        _: Uuid,
        _: &str,
    ) -> Result<bool, ApplicationError> {
        Ok(false)
    }
}

struct MockCatalog;

impl NutritionCatalog for MockCatalog {
    fn search_foods(
        &self,
        _: &str,
        _: usize,
        _: crate::NutritionSearchOptions,
    ) -> Result<Vec<NutritionSearchResult>, ApplicationError> {
        Ok(vec![])
    }

    fn food(&self, fdc_id: i64) -> Result<Option<crate::FoodRecord>, ApplicationError> {
        Ok(Some(crate::FoodRecord {
            fdc_id,
            data_type: "foundation_food".to_owned(),
            description: "Wheat flour".to_owned(),
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
        _: i64,
    ) -> Result<Vec<crate::FoodNutrientRecord>, ApplicationError> {
        Ok(vec![
            crate::FoodNutrientRecord {
                id: None,
                fdc_id: 100,
                nutrient_id: crate::FDC_ENERGY_KCAL,
                amount: Some(364.0),
                data_points: None,
                derivation_id: None,
                min: None,
                max: None,
                median: None,
            },
            crate::FoodNutrientRecord {
                id: None,
                fdc_id: 100,
                nutrient_id: crate::FDC_PROTEIN,
                amount: Some(10.0),
                data_points: None,
                derivation_id: None,
                min: None,
                max: None,
                median: None,
            },
        ])
    }
}

struct CapturingExporter {
    nutrition: Arc<Mutex<Option<NutritionFacts>>>,
}

impl RecipeExporter for CapturingExporter {
    fn export(
        &self,
        _: &Recipe,
        _: &str,
        options: &RecipeExportOptions,
    ) -> Result<RecipeExportBundle, ApplicationError> {
        *self.nutrition.lock().unwrap() = Some(options.nutrition.clone());
        Ok(RecipeExportBundle {
            file_name: "test.zip".to_owned(),
            media_type: "application/zip".to_owned(),
            files: vec![],
            archive: vec![],
        })
    }
}

struct NoopBookExporter;

impl RecipeBookExporter for NoopBookExporter {
    fn export_book(
        &self,
        _: &RecipeBook,
        _: &[(Recipe, String)],
        _: &BookExportOptions,
    ) -> Result<RecipeExportBundle, ApplicationError> {
        Err(ApplicationError::Internal("unused".to_owned()))
    }
}

fn sample_recipe(id: Uuid) -> Recipe {
    let mut flour_props = BTreeMap::new();
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
    Recipe {
        id,
        book_id: None,
        symbol: "test_bread".to_owned(),
        declared_type: TypeRef::named("Recipe"),
        title: "Test Bread".to_owned(),
        protocol_version: "0.3".to_owned(),
        types: vec![],
        resources: vec![Resource {
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
        }],
        processes: vec![],
        operations: vec![],
        servings: vec![],
        formulas: vec![],
        yields: vec![],
        properties: Default::default(),
    }
}

#[test]
fn export_recipe_resolves_linked_nutrition_instead_of_zero_hints() {
    let recipe_id = Uuid::new_v4();
    let recipe = sample_recipe(recipe_id);
    let document = RecipeDocument {
        id: recipe_id,
        book_id: None,
        symbol: recipe.symbol.clone(),
        title: recipe.title.clone(),
        protocol_version: recipe.protocol_version.clone(),
        source_text: "recipe test_bread {}".to_owned(),
        updated_at: "now".to_owned(),
    };
    let captured = Arc::new(Mutex::new(None));
    let nutrition = NutritionService::new(
        Arc::new(LinkedNutritionRepo::with_link(recipe_id, "flour", 100)),
        Arc::new(MockRecipes { document: document.clone() }),
        Arc::new(MockParser { recipe: recipe.clone() }),
        Arc::new(RwLock::new(Some(Arc::new(MockCatalog) as Arc<dyn NutritionCatalog>))),
    );
    let service = ExportService::new(
        Arc::new(MockRecipes { document }),
        Arc::new(NoopBookRepo),
        Arc::new(MockParser { recipe }),
        Arc::new(CapturingExporter {
            nutrition: captured.clone(),
        }),
        Arc::new(NoopBookExporter),
        nutrition,
    );

    service
        .export_recipe(
            recipe_id,
            &RecipeExportOptions {
                site_title: None,
                author: None,
                description: None,
                include_source: false,
                formats: vec![RecipeExportFormat::Web],
                nutrition: NutritionFacts::default(),
            },
        )
        .expect("export");

    let facts = captured.lock().unwrap().clone().expect("nutrition captured");
    assert!(facts.calories > 0.0, "expected resolved calories, got {facts:?}");
    assert!(facts.protein_grams > 0.0);
}

struct NoopBookRepo;

impl RecipeBookRepository for NoopBookRepo {
    fn list_recipe_books(&self) -> Result<Vec<crate::RecipeBookSummary>, ApplicationError> {
        Ok(vec![])
    }

    fn create_recipe_book(
        &self,
        _: crate::NewRecipeBook,
    ) -> Result<crate::RecipeBookSummary, ApplicationError> {
        unimplemented!()
    }

    fn update_recipe_book(
        &self,
        _: Uuid,
        _: crate::NewRecipeBook,
    ) -> Result<Option<crate::RecipeBookSummary>, ApplicationError> {
        Ok(None)
    }

    fn save_recipe_book(&self, _: &RecipeBook) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn delete_recipe_book(&self, _: Uuid) -> Result<bool, ApplicationError> {
        Ok(false)
    }
}
