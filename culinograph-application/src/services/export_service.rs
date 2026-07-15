use crate::{
    ApplicationError, DocumentParser, RecipeExportBundle, RecipeExportOptions, RecipeExporter,
    RecipeRepository,
};
use std::sync::Arc;
use uuid::Uuid;
#[derive(Clone)]
pub struct ExportService {
    repository: Arc<dyn RecipeRepository>,
    parser: Arc<dyn DocumentParser>,
    exporter: Arc<dyn RecipeExporter>,
}
impl ExportService {
    pub fn new(
        repository: Arc<dyn RecipeRepository>,
        parser: Arc<dyn DocumentParser>,
        exporter: Arc<dyn RecipeExporter>,
    ) -> Self {
        Self {
            repository,
            parser,
            exporter,
        }
    }
    pub fn export_recipe(
        &self,
        id: Uuid,
        options: &RecipeExportOptions,
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
}
#[cfg(test)]
mod test;
