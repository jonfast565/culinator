use crate::{ApplicationError, ImportSettings, RecipeImage};
use async_trait::async_trait;

#[async_trait]
pub trait OcrEngine: Send + Sync {
    async fn extract_text(
        &self,
        images: &[RecipeImage],
        settings: &ImportSettings,
    ) -> Result<Option<String>, ApplicationError>;
}

#[async_trait]
pub trait RecipeImageInterpreter: Send + Sync {
    async fn interpret(
        &self,
        images: &[RecipeImage],
        extracted_text: Option<&str>,
        target_language: Option<&str>,
        settings: &ImportSettings,
    ) -> Result<(String, String, Vec<String>), ApplicationError>;
}

pub trait SettingsStore: Send + Sync {
    fn load_import_settings(&self) -> Result<ImportSettings, ApplicationError>;
    fn save_import_settings(&self, settings: &ImportSettings) -> Result<(), ApplicationError>;
}

#[cfg(test)]
mod test;
