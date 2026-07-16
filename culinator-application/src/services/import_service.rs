use culinator_models::{
    ApplicationError, DocumentParser, ImportSettings, OcrEngine, RecipeImageInterpreter,
    RecipeImportRequest, RecipeImportResult, RecipeValidator, SettingsStore,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ImportService {
    ocr: Arc<dyn OcrEngine>,
    interpreter: Arc<dyn RecipeImageInterpreter>,
    settings: Arc<dyn SettingsStore>,
    parser: Arc<dyn DocumentParser>,
    validator: Arc<dyn RecipeValidator>,
}
impl ImportService {
    pub fn new(
        ocr: Arc<dyn OcrEngine>,
        interpreter: Arc<dyn RecipeImageInterpreter>,
        settings: Arc<dyn SettingsStore>,
        parser: Arc<dyn DocumentParser>,
        validator: Arc<dyn RecipeValidator>,
    ) -> Self {
        Self {
            ocr,
            interpreter,
            settings,
            parser,
            validator,
        }
    }
    pub fn settings(&self) -> Result<ImportSettings, ApplicationError> {
        self.settings.load_import_settings()
    }
    pub fn save_settings(&self, value: &ImportSettings) -> Result<(), ApplicationError> {
        self.settings.save_import_settings(value)
    }
    pub async fn translate(
        &self,
        request: RecipeImportRequest,
    ) -> Result<RecipeImportResult, ApplicationError> {
        if request.images.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "at least one recipe image is required".into(),
            ));
        }
        let settings = self.settings()?;
        let extracted = self.ocr.extract_text(&request.images, &settings).await?;
        let (title, source_text, warnings) = self
            .interpreter
            .interpret(
                &request.images,
                extracted.as_deref(),
                request.target_language.as_deref(),
                &settings,
            )
            .await?;
        let validation = match self.parser.parse_recipe(&source_text) {
            Ok(recipe) => {
                let diagnostics = self.validator.validate(&recipe);
                let valid = !diagnostics
                    .iter()
                    .any(|d| d.severity == culinator_models::DiagnosticSeverity::Error);
                culinator_models::ValidationReport {
                    valid,
                    diagnostics,
                    outline: Some(culinator_models::RecipeOutline {
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
            Err(error) => culinator_models::ValidationReport {
                valid: false,
                diagnostics: vec![culinator_models::SourceDiagnostic {
                    code: "IMPORT_PARSE".into(),
                    severity: culinator_models::DiagnosticSeverity::Error,
                    message: error.to_string(),
                    start: None,
                    end: None,
                }],
                outline: None,
            },
        };
        Ok(RecipeImportResult {
            title,
            source_text,
            extracted_text: extracted.unwrap_or_default(),
            warnings,
            validation,
        })
    }
}
#[cfg(test)]
mod test;
