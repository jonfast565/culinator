use crate::{
    ApplicationError, DiagnosticSeverity, DocumentParser, NewRecipe, RecipeOutline,
    RecipeRepository, RecipeValidator, SourceDiagnostic, ValidationReport,
};
use culinograph_core::Recipe;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct RecipeService {
    repository: Arc<dyn RecipeRepository>,
    parser: Arc<dyn DocumentParser>,
    validator: Arc<dyn RecipeValidator>,
}

impl RecipeService {
    pub fn new(
        repository: Arc<dyn RecipeRepository>,
        parser: Arc<dyn DocumentParser>,
        validator: Arc<dyn RecipeValidator>,
    ) -> Self {
        Self {
            repository,
            parser,
            validator,
        }
    }

    pub fn list(&self) -> Result<Vec<crate::RecipeSummary>, ApplicationError> {
        self.repository.list_recipes()
    }

    pub fn get(&self, id: Uuid) -> Result<crate::RecipeDocument, ApplicationError> {
        self.repository
            .get_recipe(id)?
            .ok_or_else(|| ApplicationError::not_found("recipe"))
    }

    pub fn create(&self, book_id: Option<Uuid>) -> Result<crate::RecipeDocument, ApplicationError> {
        self.repository.create_recipe(NewRecipe {
            book_id,
            symbol: "new_recipe".to_owned(),
            title: "Untitled Recipe".to_owned(),
            protocol_version: "0.3".to_owned(),
            source_text:
                "culinograph 0.3;\n\nrecipe new_recipe {\n    title \"Untitled Recipe\";\n}\n"
                    .to_owned(),
        })
    }

    pub fn save(
        &self,
        id: Uuid,
        source_text: &str,
    ) -> Result<crate::RecipeDocument, ApplicationError> {
        let mut recipe = self.parser.parse_recipe(source_text)?;
        let diagnostics = self.validator.validate(&recipe);
        if diagnostics
            .iter()
            .any(|item| item.severity == DiagnosticSeverity::Error)
        {
            return Err(ApplicationError::Validation);
        }
        recipe.id = id;
        recipe.book_id = self
            .repository
            .get_recipe(id)?
            .and_then(|document| document.book_id);
        self.repository.save_recipe(&recipe, source_text)
    }

    pub fn delete(&self, id: Uuid) -> Result<(), ApplicationError> {
        if self.repository.delete_recipe(id)? {
            Ok(())
        } else {
            Err(ApplicationError::not_found("recipe"))
        }
    }

    pub fn move_to_book(
        &self,
        id: Uuid,
        book_id: Option<Uuid>,
        position: i64,
    ) -> Result<(), ApplicationError> {
        if self.repository.move_recipe(id, book_id, position)? {
            Ok(())
        } else {
            Err(ApplicationError::not_found("recipe"))
        }
    }

    pub fn validate_source(&self, source_text: &str) -> ValidationReport {
        match self.parser.parse_recipe(source_text) {
            Ok(recipe) => self.report_for_recipe(&recipe),
            Err(error) => ValidationReport {
                valid: false,
                diagnostics: vec![SourceDiagnostic {
                    code: "CG0001".to_owned(),
                    severity: DiagnosticSeverity::Error,
                    message: error.to_string(),
                    start: None,
                    end: None,
                }],
                outline: None,
            },
        }
    }

    fn report_for_recipe(&self, recipe: &Recipe) -> ValidationReport {
        let diagnostics = self.validator.validate(recipe);
        let valid = !diagnostics
            .iter()
            .any(|item| item.severity == DiagnosticSeverity::Error);
        ValidationReport {
            valid,
            diagnostics,
            outline: Some(RecipeOutline {
                title: recipe.title.clone(),
                symbol: recipe.symbol.clone(),
                protocol_version: recipe.protocol_version.clone(),
                type_count: recipe.types.len(),
                resource_count: recipe.resources.len(),
                process_count: recipe.processes.len(),
                operation_count: recipe.operations.len(),
                serving_count: recipe.servings.len(),
                formula_count: recipe.formulas.len(),
            }),
        }
    }
}

#[cfg(test)]
mod test;
