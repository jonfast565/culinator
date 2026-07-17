use culinator_models::{
    ApplicationError, DocumentParser, ImportDraft, ImportSettings, OcrEngine, PublicImportSettings,
    RecipeImageInterpreter, RecipeImportRequest, RecipeImportResult, RecipeValidator, SecretStore,
    SettingsStore, StructuredInput, StructuredRecipeImporter,
};
use culinator_secrets::{KeyringSecretStore, OPENAI_API_KEY};
use std::sync::Arc;

#[derive(Clone)]
pub struct ImportService {
    ocr: Arc<dyn OcrEngine>,
    interpreter: Arc<dyn RecipeImageInterpreter>,
    structured: Arc<dyn StructuredRecipeImporter>,
    settings: Arc<dyn SettingsStore>,
    secrets: Arc<dyn SecretStore>,
    parser: Arc<dyn DocumentParser>,
    validator: Arc<dyn RecipeValidator>,
}

impl ImportService {
    pub fn new(
        ocr: Arc<dyn OcrEngine>,
        interpreter: Arc<dyn RecipeImageInterpreter>,
        structured: Arc<dyn StructuredRecipeImporter>,
        settings: Arc<dyn SettingsStore>,
        secrets: Arc<dyn SecretStore>,
        parser: Arc<dyn DocumentParser>,
        validator: Arc<dyn RecipeValidator>,
    ) -> Self {
        Self {
            ocr,
            interpreter,
            structured,
            settings,
            secrets,
            parser,
            validator,
        }
    }

    pub fn settings(&self) -> Result<ImportSettings, ApplicationError> {
        let mut settings = self.settings.load_import_settings()?;
        settings.openai_api_key = self.secrets.get_secret(OPENAI_API_KEY)?.unwrap_or_default();
        Ok(settings)
    }

    pub fn public_settings(&self) -> Result<PublicImportSettings, ApplicationError> {
        let settings = self.settings.load_import_settings()?;
        let api_key_configured = self
            .secrets
            .get_secret(OPENAI_API_KEY)?
            .is_some_and(|key| !key.trim().is_empty());
        Ok(PublicImportSettings::from_settings(
            &settings,
            api_key_configured,
            Some(if KeyringSecretStore::is_available() {
                "keychain".to_owned()
            } else {
                "encrypted file".to_owned()
            }),
        ))
    }

    pub fn save_settings(&self, value: &ImportSettings) -> Result<(), ApplicationError> {
        let key = value.openai_api_key.trim();
        if key.is_empty() {
            self.secrets.delete_secret(OPENAI_API_KEY)?;
        } else {
            self.secrets.set_secret(OPENAI_API_KEY, key)?;
        }
        let mut stored = value.clone();
        stored.openai_api_key.clear();
        self.settings.save_import_settings(&stored)
    }

    pub fn import_structured(
        &self,
        input: StructuredInput,
    ) -> Result<ImportDraft, ApplicationError> {
        let mut draft = self.structured.import(input)?;
        match self.parser.parse_recipe(&draft.source_text) {
            Ok(recipe) => {
                draft.title = recipe.title.clone();
                let diagnostics = self.validator.validate(&recipe);
                draft.warnings.extend(
                    diagnostics
                        .into_iter()
                        .filter(|diagnostic| {
                            diagnostic.severity == culinator_models::DiagnosticSeverity::Warning
                        })
                        .map(|diagnostic| diagnostic.message),
                );
                Ok(draft)
            }
            Err(error) => Err(ApplicationError::InvalidInput(format!(
                "structured import produced invalid DSL: {error}"
            ))),
        }
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
