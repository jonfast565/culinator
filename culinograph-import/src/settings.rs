use culinograph_models::{ApplicationError, ImportSettings, SettingsStore};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct JsonSettingsStore { path: PathBuf }
impl JsonSettingsStore { pub fn new(path: PathBuf) -> Self { Self { path } } }
impl SettingsStore for JsonSettingsStore {
    fn load_import_settings(&self) -> Result<ImportSettings, ApplicationError> {
        if !self.path.exists() { return Ok(ImportSettings::default()); }
        let data = fs::read_to_string(&self.path).map_err(|e| ApplicationError::Persistence(e.to_string()))?;
        serde_json::from_str(&data).map_err(|e| ApplicationError::InvalidInput(format!("invalid settings file: {e}")))
    }
    fn save_import_settings(&self, settings: &ImportSettings) -> Result<(), ApplicationError> {
        if let Some(parent)=self.path.parent() { fs::create_dir_all(parent).map_err(|e| ApplicationError::Persistence(e.to_string()))?; }
        let data=serde_json::to_vec_pretty(settings).map_err(|e| ApplicationError::Internal(e.to_string()))?;
        let temp=self.path.with_extension("json.tmp");
        fs::write(&temp,data).map_err(|e| ApplicationError::Persistence(e.to_string()))?;
        fs::rename(temp,&self.path).map_err(|e| ApplicationError::Persistence(e.to_string()))
    }
}
#[cfg(test)] mod test;
