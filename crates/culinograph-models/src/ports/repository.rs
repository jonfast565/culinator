use crate::{
    ApplicationError, NewRecipe, NewRecipeBook, RecipeBookSummary, RecipeDocument, RecipeSummary,
};
use culinograph_core::{Formula, FormulaResult, Recipe, RecipeBook};
use uuid::Uuid;

pub trait RecipeRepository: Send + Sync {
    fn list_recipes(&self) -> Result<Vec<RecipeSummary>, ApplicationError>;
    fn get_recipe(&self, id: Uuid) -> Result<Option<RecipeDocument>, ApplicationError>;
    fn create_recipe(&self, recipe: NewRecipe) -> Result<RecipeDocument, ApplicationError>;
    fn save_recipe(&self, recipe: &Recipe, source_text: &str) -> Result<RecipeDocument, ApplicationError>;
    fn delete_recipe(&self, id: Uuid) -> Result<bool, ApplicationError>;
    fn move_recipe(&self, id: Uuid, book_id: Option<Uuid>, position: i64) -> Result<bool, ApplicationError>;
}

pub trait RecipeBookRepository: Send + Sync {
    fn list_recipe_books(&self) -> Result<Vec<RecipeBookSummary>, ApplicationError>;
    fn create_recipe_book(&self, input: NewRecipeBook) -> Result<RecipeBookSummary, ApplicationError>;
    fn update_recipe_book(&self, id: Uuid, input: NewRecipeBook) -> Result<Option<RecipeBookSummary>, ApplicationError>;
    fn save_recipe_book(&self, book: &RecipeBook) -> Result<(), ApplicationError>;
    fn delete_recipe_book(&self, id: Uuid) -> Result<bool, ApplicationError>;
}

pub trait FormulaRepository: Send + Sync {
    fn save_formula(&self, formula: &Formula) -> Result<(), ApplicationError>;
    fn get_formula(&self, id: Uuid) -> Result<Option<Formula>, ApplicationError>;
    fn list_formulas_for_recipe(&self, recipe_id: Uuid) -> Result<Vec<Formula>, ApplicationError>;
    fn save_formula_run(
        &self,
        formula_id: Uuid,
        target_mass_grams: f64,
        result: &FormulaResult,
    ) -> Result<(), ApplicationError>;
}

pub trait CatalogRepository:
    RecipeRepository + RecipeBookRepository + FormulaRepository + Send + Sync
{
}

impl<T> CatalogRepository for T where
    T: RecipeRepository + RecipeBookRepository + FormulaRepository + Send + Sync
{
}

#[cfg(test)]
mod test;
