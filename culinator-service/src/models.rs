use culinator_core::{Formula, PercentageView};
use culinator_models::{ImportSettings, RecipeExportOptions, RecipeImage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub api_version: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSummary {
    pub id: String,
    pub book_id: Option<String>,
    pub symbol: String,
    pub title: String,
    pub protocol_version: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeDocument {
    pub id: String,
    pub book_id: Option<String>,
    pub symbol: String,
    pub title: String,
    pub protocol_version: String,
    pub updated_at: String,
    pub source_text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub severity: &'static str,
    pub message: String,
    pub start: Option<usize>,
    pub end: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeOutline {
    pub title: String,
    pub symbol: String,
    pub protocol_version: String,
    pub type_count: usize,
    pub resource_count: usize,
    pub process_count: usize,
    pub operation_count: usize,
    pub serving_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub valid: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub outline: Option<RecipeOutline>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRecipeRequest {
    pub source_text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateRequest {
    pub source_text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormulaCalculationRequest {
    pub formula: Formula,
    pub target_mass_grams: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PercentageRequest {
    pub formula: Formula,
    pub view: PercentageView,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormulaRunRequest {
    pub target_mass_grams: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeBookSummary {
    pub id: String,
    pub symbol: String,
    pub title: String,
    pub description: Option<String>,
    pub protocol_version: String,
    pub recipe_count: i64,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRecipeBookRequest {
    pub title: String,
    pub symbol: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateRecipeRequest {
    pub book_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveRecipeRequest {
    pub book_id: Option<String>,
    #[serde(default)]
    pub position: i64,
}
#[cfg(test)]
mod test;

impl From<culinator_application::RecipeSummary> for RecipeSummary {
    fn from(value: culinator_application::RecipeSummary) -> Self {
        Self {
            id: value.id.to_string(),
            book_id: value.book_id.map(|id| id.to_string()),
            symbol: value.symbol,
            title: value.title,
            protocol_version: value.protocol_version,
            updated_at: value.updated_at,
        }
    }
}

impl From<culinator_application::RecipeDocument> for RecipeDocument {
    fn from(value: culinator_application::RecipeDocument) -> Self {
        Self {
            id: value.id.to_string(),
            book_id: value.book_id.map(|id| id.to_string()),
            symbol: value.symbol,
            title: value.title,
            protocol_version: value.protocol_version,
            updated_at: value.updated_at,
            source_text: value.source_text,
        }
    }
}

impl From<culinator_application::SourceDiagnostic> for Diagnostic {
    fn from(value: culinator_application::SourceDiagnostic) -> Self {
        let severity = match value.severity {
            culinator_application::DiagnosticSeverity::Error => "error",
            culinator_application::DiagnosticSeverity::Warning => "warning",
            culinator_application::DiagnosticSeverity::Information => "information",
        };
        Self {
            severity,
            message: value.message,
            start: value.start,
            end: value.end,
        }
    }
}

impl From<culinator_application::RecipeOutline> for RecipeOutline {
    fn from(value: culinator_application::RecipeOutline) -> Self {
        Self {
            title: value.title,
            symbol: value.symbol,
            protocol_version: value.protocol_version,
            type_count: value.type_count,
            resource_count: value.resource_count,
            process_count: value.process_count,
            operation_count: value.operation_count,
            serving_count: value.serving_count,
        }
    }
}

impl From<culinator_application::RecipeBookSummary> for RecipeBookSummary {
    fn from(value: culinator_application::RecipeBookSummary) -> Self {
        Self {
            id: value.id.to_string(),
            symbol: value.symbol,
            title: value.title,
            description: value.description,
            protocol_version: value.protocol_version,
            recipe_count: value.recipe_count,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRecipeRequest {
    pub options: RecipeExportOptions,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportBookRequest {
    pub options: culinator_models::BookExportOptions,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredImportRequest {
    pub format: culinator_models::StructuredInputFormat,
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRecipeResponse {
    pub file_name: String,
    pub media_type: &'static str,
    pub archive_base64: String,
    pub files: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateRecipeImagesRequest {
    pub images: Vec<RecipeImage>,
    pub target_language: Option<String>,
    pub recipe_book_title: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateImportSettingsRequest {
    pub openai_api_key: Option<String>,
    pub openai_model: String,
    pub use_local_ocr: bool,
    pub tesseract_command: String,
}

impl UpdateImportSettingsRequest {
    pub fn merge(self, mut existing: ImportSettings) -> ImportSettings {
        if let Some(key) = self.openai_api_key {
            if !key.trim().is_empty() {
                existing.openai_api_key = key;
            }
        }
        existing.openai_model = self.openai_model;
        existing.use_local_ocr = self.use_local_ocr;
        existing.tesseract_command = self.tesseract_command;
        existing
    }
}
