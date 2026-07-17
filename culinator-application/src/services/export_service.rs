use crate::{
    ApplicationError, BookExportOptions, DocumentParser, RecipeBookExporter, RecipeBookRepository,
    RecipeExportBundle, RecipeExporter, RecipeRepository,
};
use culinator_core::{RecipeBook, TypeRef};
use std::{collections::BTreeMap, sync::Arc};
use uuid::Uuid;

#[derive(Clone)]
pub struct ExportService {
    repository: Arc<dyn RecipeRepository>,
    book_repository: Arc<dyn RecipeBookRepository>,
    parser: Arc<dyn DocumentParser>,
    exporter: Arc<dyn RecipeExporter>,
    book_exporter: Arc<dyn RecipeBookExporter>,
}

impl ExportService {
    pub fn new(
        repository: Arc<dyn RecipeRepository>,
        book_repository: Arc<dyn RecipeBookRepository>,
        parser: Arc<dyn DocumentParser>,
        exporter: Arc<dyn RecipeExporter>,
        book_exporter: Arc<dyn RecipeBookExporter>,
    ) -> Self {
        Self {
            repository,
            book_repository,
            parser,
            exporter,
            book_exporter,
        }
    }

    pub fn export_recipe(
        &self,
        id: Uuid,
        options: &culinator_models::RecipeExportOptions,
    ) -> Result<RecipeExportBundle, ApplicationError> {
        let document = self
            .repository
            .get_recipe(id)?
            .ok_or_else(|| ApplicationError::not_found("recipe"))?;
        let mut recipe = self.parser.parse_recipe(&document.source_text)?;
        recipe.id = document.id;
        recipe.book_id = document.book_id;
        self.exporter
            .export(&recipe, &document.source_text, options)
    }

    pub fn export_book(
        &self,
        book_id: Uuid,
        options: &BookExportOptions,
    ) -> Result<RecipeExportBundle, ApplicationError> {
        let summary = self
            .book_repository
            .list_recipe_books()?
            .into_iter()
            .find(|book| book.id == book_id)
            .ok_or_else(|| ApplicationError::not_found("recipe book"))?;
        let book = RecipeBook {
            id: summary.id,
            symbol: summary.symbol,
            declared_type: TypeRef::named("RecipeBook"),
            title: summary.title,
            description: summary.description,
            protocol_version: summary.protocol_version,
            recipes: Vec::new(),
            properties: BTreeMap::new(),
        };
        let mut recipes = Vec::new();
        for recipe_summary in self
            .repository
            .list_recipes()?
            .into_iter()
            .filter(|recipe| recipe.book_id == Some(book_id))
        {
            let document = self
                .repository
                .get_recipe(recipe_summary.id)?
                .ok_or_else(|| ApplicationError::not_found("recipe"))?;
            let mut recipe = self.parser.parse_recipe(&document.source_text)?;
            recipe.id = document.id;
            recipe.book_id = document.book_id;
            recipes.push((recipe, document.source_text));
        }
        self.book_exporter.export_book(&book, &recipes, options)
    }
}

#[cfg(test)]
mod test;
