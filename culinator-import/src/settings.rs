use culinator_models::{ApplicationError, ImportSettings, SecretStore, SettingsStore};
use culinator_secrets::OPENAI_API_KEY;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct StoredImportSettings {
    #[serde(default = "default_openai_model")]
    openai_model: String,
    #[serde(default = "default_true")]
    use_local_ocr: bool,
    #[serde(default = "default_tesseract")]
    tesseract_command: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyImportSettingsFile {
    #[serde(default)]
    openai_api_key: String,
    #[serde(flatten)]
    stored: StoredImportSettings,
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

impl From<&ImportSettings> for StoredImportSettings {
    fn from(value: &ImportSettings) -> Self {
        Self {
            openai_model: value.openai_model.clone(),
            use_local_ocr: value.use_local_ocr,
            tesseract_command: value.tesseract_command.clone(),
        }
    }
}

impl From<StoredImportSettings> for ImportSettings {
    fn from(value: StoredImportSettings) -> Self {
        Self {
            openai_api_key: String::new(),
            openai_model: value.openai_model,
            use_local_ocr: value.use_local_ocr,
            tesseract_command: value.tesseract_command,
        }
    }
}

#[derive(Clone)]
pub struct JsonSettingsStore {
    path: PathBuf,
    secrets: Arc<dyn SecretStore>,
}

impl JsonSettingsStore {
    pub fn new(path: PathBuf, secrets: Arc<dyn SecretStore>) -> Self {
        Self { path, secrets }
    }

    fn migrate_legacy_key(&self, legacy_key: &str) -> Result<(), ApplicationError> {
        if legacy_key.trim().is_empty() {
            return Ok(());
        }
        self.secrets.set_secret(OPENAI_API_KEY, legacy_key.trim())?;
        Ok(())
    }

    fn write_stored(&self, stored: &StoredImportSettings) -> Result<(), ApplicationError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| ApplicationError::Persistence(e.to_string()))?;
        }
        let data = serde_json::to_vec_pretty(stored)
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;
        let temp = self.path.with_extension("json.tmp");
        fs::write(&temp, data).map_err(|e| ApplicationError::Persistence(e.to_string()))?;
        fs::rename(&temp, &self.path).map_err(|e| ApplicationError::Persistence(e.to_string()))
    }
}

impl SettingsStore for JsonSettingsStore {
    fn load_import_settings(&self) -> Result<ImportSettings, ApplicationError> {
        if !self.path.exists() {
            return Ok(ImportSettings::default());
        }
        let data = fs::read_to_string(&self.path)
            .map_err(|e| ApplicationError::Persistence(e.to_string()))?;

        if let Ok(legacy) = serde_json::from_str::<LegacyImportSettingsFile>(&data) {
            let settings = ImportSettings::from(legacy.stored);
            if !legacy.openai_api_key.trim().is_empty() {
                self.migrate_legacy_key(&legacy.openai_api_key)?;
                self.write_stored(&StoredImportSettings::from(&settings))?;
            }
            return Ok(settings);
        }

        let stored: StoredImportSettings = serde_json::from_str(&data)
            .map_err(|e| ApplicationError::InvalidInput(format!("invalid settings file: {e}")))?;
        Ok(ImportSettings::from(stored))
    }

    fn save_import_settings(&self, settings: &ImportSettings) -> Result<(), ApplicationError> {
        self.write_stored(&StoredImportSettings::from(settings))
    }
}

#[cfg(test)]
mod test;
