use crate::{ApplicationError, NewRecipeBook, RecipeBookRepository, RecipeBookSummary};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct BookService {
    repository: Arc<dyn RecipeBookRepository>,
}

impl BookService {
    pub fn new(repository: Arc<dyn RecipeBookRepository>) -> Self {
        Self { repository }
    }

    pub fn list(&self) -> Result<Vec<RecipeBookSummary>, ApplicationError> {
        self.repository.list_recipe_books()
    }

    pub fn create(&self, input: NewRecipeBook) -> Result<RecipeBookSummary, ApplicationError> {
        if input.title.trim().is_empty() {
            return Err(ApplicationError::InvalidInput("book title cannot be empty".to_owned()));
        }
        self.repository.create_recipe_book(input)
    }

    pub fn update(&self, id: Uuid, input: NewRecipeBook) -> Result<RecipeBookSummary, ApplicationError> {
        self.repository
            .update_recipe_book(id, input)?
            .ok_or_else(|| ApplicationError::not_found("recipe book"))
    }

    pub fn delete(&self, id: Uuid) -> Result<(), ApplicationError> {
        if self.repository.delete_recipe_book(id)? {
            Ok(())
        } else {
            Err(ApplicationError::not_found("recipe book"))
        }
    }
}

#[cfg(test)]
mod test;
