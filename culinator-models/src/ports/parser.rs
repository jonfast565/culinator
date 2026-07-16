use crate::ApplicationError;
use culinator_core::{Document, Recipe, RecipeBook};

pub trait DocumentParser: Send + Sync {
    fn parse_document(&self, source: &str) -> Result<Document, ApplicationError>;

    fn parse_recipe(&self, source: &str) -> Result<Recipe, ApplicationError> {
        match self.parse_document(source)? {
            Document::Recipe { recipe } => Ok(recipe),
            Document::RecipeBook { .. } => Err(ApplicationError::InvalidInput(
                "expected a recipe document, found a recipe book".to_owned(),
            )),
        }
    }

    fn parse_recipe_book(&self, source: &str) -> Result<RecipeBook, ApplicationError> {
        match self.parse_document(source)? {
            Document::RecipeBook { book } => Ok(book),
            Document::Recipe { .. } => Err(ApplicationError::InvalidInput(
                "expected a recipe book document, found a recipe".to_owned(),
            )),
        }
    }
}

#[cfg(test)]
mod test;
