use culinator_core::Formula;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeSummary {
    pub id: Uuid,
    pub book_id: Option<Uuid>,
    pub symbol: String,
    pub title: String,
    pub protocol_version: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeDocument {
    pub id: Uuid,
    pub book_id: Option<Uuid>,
    pub symbol: String,
    pub title: String,
    pub protocol_version: String,
    pub source_text: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeBookSummary {
    pub id: Uuid,
    pub symbol: String,
    pub title: String,
    pub description: Option<String>,
    pub protocol_version: String,
    pub recipe_count: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewRecipeBook {
    pub title: String,
    pub symbol: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewRecipe {
    pub book_id: Option<Uuid>,
    pub symbol: String,
    pub title: String,
    pub protocol_version: String,
    pub source_text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredFormula {
    pub formula: Formula,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceDiagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub start: Option<usize>,
    pub end: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeOutline {
    pub title: String,
    pub symbol: String,
    pub protocol_version: String,
    pub type_count: usize,
    pub resource_count: usize,
    pub process_count: usize,
    pub operation_count: usize,
    pub serving_count: usize,
    pub formula_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub diagnostics: Vec<SourceDiagnostic>,
    pub outline: Option<RecipeOutline>,
}

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NutrientDefinition {
    pub id: i64,
    pub number: Option<String>,
    pub name: String,
    pub unit_name: String,
    pub rank: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoodRecord {
    pub fdc_id: i64,
    pub data_type: String,
    pub description: String,
    pub food_category_id: Option<i64>,
    pub publication_date: Option<String>,
    pub brand_owner: Option<String>,
    pub brand_name: Option<String>,
    pub gtin_upc: Option<String>,
    pub ingredients: Option<String>,
    pub serving_size: Option<f64>,
    pub serving_size_unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoodNutrientRecord {
    pub id: Option<i64>,
    pub fdc_id: i64,
    pub nutrient_id: i64,
    pub amount: Option<f64>,
    pub data_points: Option<i64>,
    pub derivation_id: Option<i64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub median: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NutritionSearchResult {
    pub fdc_id: i64,
    pub description: String,
    pub data_type: String,
    pub brand_owner: Option<String>,
    pub serving_size: Option<f64>,
    pub serving_size_unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NutritionFacts {
    pub servings_per_container: f64,
    pub serving_size: String,
    pub serving_size_grams: Option<f64>,
    pub calories: f64,
    pub total_fat_grams: f64,
    pub saturated_fat_grams: f64,
    pub trans_fat_grams: f64,
    pub cholesterol_milligrams: f64,
    pub sodium_milligrams: f64,
    pub total_carbohydrate_grams: f64,
    pub dietary_fiber_grams: f64,
    pub total_sugars_grams: f64,
    pub added_sugars_grams: f64,
    pub protein_grams: f64,
    pub vitamin_d_micrograms: Option<f64>,
    pub calcium_milligrams: Option<f64>,
    pub iron_milligrams: Option<f64>,
    pub potassium_milligrams: Option<f64>,
}

impl Default for NutritionFacts {
    fn default() -> Self {
        Self {
            servings_per_container: 1.0,
            serving_size: "1 serving".to_owned(),
            serving_size_grams: None,
            calories: 0.0,
            total_fat_grams: 0.0,
            saturated_fat_grams: 0.0,
            trans_fat_grams: 0.0,
            cholesterol_milligrams: 0.0,
            sodium_milligrams: 0.0,
            total_carbohydrate_grams: 0.0,
            dietary_fiber_grams: 0.0,
            total_sugars_grams: 0.0,
            added_sugars_grams: 0.0,
            protein_grams: 0.0,
            vitamin_d_micrograms: None,
            calcium_milligrams: None,
            iron_milligrams: None,
            potassium_milligrams: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeExportFormat {
    Web,
    Markdown,
    PlainText,
    IngredientCsv,
    Json,
    PrintHtml,
    Epub,
}

fn default_export_formats() -> Vec<RecipeExportFormat> {
    vec![RecipeExportFormat::Web, RecipeExportFormat::Json]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeExportOptions {
    pub site_title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub include_source: bool,
    #[serde(default = "default_export_formats")]
    pub formats: Vec<RecipeExportFormat>,
    pub nutrition: NutritionFacts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFile {
    pub path: String,
    pub media_type: String,
    pub contents: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeExportBundle {
    pub file_name: String,
    pub files: Vec<ExportFile>,
    pub archive: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeImage {
    pub file_name: String,
    pub media_type: String,
    pub data_base64: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportSettings {
    #[serde(default)]
    pub openai_api_key: String,
    #[serde(default = "default_openai_model")]
    pub openai_model: String,
    #[serde(default = "default_true")]
    pub use_local_ocr: bool,
    #[serde(default = "default_tesseract")]
    pub tesseract_command: String,
}

fn default_openai_model() -> String {
    "gpt-4.1-mini".to_owned()
}
fn default_true() -> bool {
    true
}
fn default_tesseract() -> String {
    "tesseract".to_owned()
}

impl Default for ImportSettings {
    fn default() -> Self {
        Self {
            openai_api_key: String::new(),
            openai_model: default_openai_model(),
            use_local_ocr: true,
            tesseract_command: default_tesseract(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicImportSettings {
    pub api_key_configured: bool,
    pub openai_model: String,
    pub use_local_ocr: bool,
    pub tesseract_command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_store_backend: Option<String>,
}

impl PublicImportSettings {
    pub fn from_settings(
        settings: &ImportSettings,
        api_key_configured: bool,
        secret_store_backend: Option<String>,
    ) -> Self {
        Self {
            api_key_configured,
            openai_model: settings.openai_model.clone(),
            use_local_ocr: settings.use_local_ocr,
            tesseract_command: settings.tesseract_command.clone(),
            secret_store_backend,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeImportRequest {
    pub images: Vec<RecipeImage>,
    pub target_language: Option<String>,
    pub recipe_book_title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeImportResult {
    pub title: String,
    pub source_text: String,
    pub extracted_text: String,
    pub warnings: Vec<String>,
    pub validation: ValidationReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleOptions {
    #[serde(default = "default_schedule_duration")]
    pub default_duration_seconds: u64,
}
fn default_schedule_duration() -> u64 {
    300
}
impl Default for ScheduleOptions {
    fn default() -> Self {
        Self {
            default_duration_seconds: default_schedule_duration(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledOperation {
    pub symbol: String,
    pub process: String,
    pub action: String,
    pub start_seconds: u64,
    pub end_seconds: u64,
    pub duration_seconds: u64,
    pub labor: Option<culinator_core::LaborMode>,
    pub dependencies: Vec<String>,
    pub resources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitReport {
    pub catalog_ready: bool,
    pub recipes_seeded: bool,
    pub nutrition_ready: bool,
    pub nutrition_starter: bool,
    pub recipe_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeF64 {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub text: Option<String>,
    pub book_id: Option<Uuid>,
    #[serde(default)]
    pub exclude_allergens: Vec<String>,
    pub max_active_minutes: Option<f64>,
    pub hydration: Option<RangeF64>,
    #[serde(default = "default_search_limit")]
    pub limit: usize,
}

fn default_search_limit() -> usize {
    50
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub recipe_id: Uuid,
    pub book_id: Option<Uuid>,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BookExportFormat {
    Epub,
    PrintHtml,
    Web,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitSystem {
    #[default]
    Metric,
    UsCustomary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Locale {
    #[default]
    EnUs,
    EnGb,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookExportOptions {
    #[serde(default)]
    pub formats: Vec<BookExportFormat>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    #[serde(default)]
    pub unit_system: UnitSystem,
    #[serde(default = "default_true")]
    pub include_nutrition: bool,
    #[serde(default = "default_true")]
    pub toc: bool,
    #[serde(default = "default_true")]
    pub section_dividers: bool,
}

impl Default for BookExportOptions {
    fn default() -> Self {
        Self {
            formats: vec![BookExportFormat::Epub, BookExportFormat::PrintHtml],
            title: None,
            author: None,
            description: None,
            cover_image: None,
            unit_system: UnitSystem::Metric,
            include_nutrition: true,
            toc: true,
            section_dividers: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuredInputFormat {
    JsonLd,
    Json,
    Yaml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredInput {
    pub format: StructuredInputFormat,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportDraft {
    pub title: String,
    pub source_text: String,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitConvertRequest {
    pub value: f64,
    pub from_unit: String,
    pub to_unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitConvertResponse {
    pub value: f64,
    pub unit: String,
    pub dimension: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitFormatRequest {
    pub value: f64,
    pub unit: String,
    #[serde(default)]
    pub unit_system: UnitSystem,
    #[serde(default)]
    pub locale: Locale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitFormatResponse {
    pub formatted: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrefermentBuildRequest {
    pub kind: String,
    pub flour_pct: f64,
    pub hydration: f64,
    #[serde(default)]
    pub inoculation: f64,
    #[serde(default = "default_preferment_stage")]
    pub stage: String,
}

fn default_preferment_stage() -> String {
    "preferment".to_owned()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoughTempRequest {
    pub desired_dough_temp: f64,
    pub friction_factor: f64,
    pub flour_temp: f64,
    pub room_temp: f64,
    pub preferment_temp: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoughTempResponse {
    pub water_temp: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSchedule {
    pub operations: Vec<ScheduledOperation>,
    pub makespan_seconds: u64,
    pub critical_path: Vec<String>,
}
