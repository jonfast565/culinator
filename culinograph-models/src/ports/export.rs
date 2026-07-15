use crate::{ApplicationError, RecipeExportBundle, RecipeExportOptions};
use culinograph_core::Recipe;

pub trait RecipeExporter: Send + Sync {
    fn export(
        &self,
        recipe: &Recipe,
        source_text: &str,
        options: &RecipeExportOptions,
    ) -> Result<RecipeExportBundle, ApplicationError>;
}

#[cfg(test)]
mod test;
