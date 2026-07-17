use crate::{ApplicationError, BookExportOptions, RecipeExportBundle};
use culinator_core::{Recipe, RecipeBook};

pub trait RecipeBookExporter: Send + Sync {
    fn export_book(
        &self,
        book: &RecipeBook,
        recipes: &[(Recipe, String)],
        options: &BookExportOptions,
    ) -> Result<RecipeExportBundle, ApplicationError>;
}
