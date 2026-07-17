use super::*;
use culinator_models::SecretStore;
use culinator_secrets::{EncryptedFileSecretStore, OPENAI_API_KEY};
use std::sync::Arc;

#[test]
fn settings_round_trip_without_api_key_in_json() {
    let dir = tempfile::tempdir().unwrap();
    let secrets: Arc<dyn SecretStore> = Arc::new(EncryptedFileSecretStore::new(
        dir.path().join("secrets.enc"),
    ));
    let store = JsonSettingsStore::new(dir.path().join("settings.json"), secrets.clone());
    let value = ImportSettings {
        openai_model: "test".into(),
        ..ImportSettings::default()
    };
    store.save_import_settings(&value).unwrap();
    let loaded = store.load_import_settings().unwrap();
    assert_eq!(loaded.openai_model, "test");
    let raw = std::fs::read_to_string(dir.path().join("settings.json")).unwrap();
    assert!(!raw.contains("openaiApiKey"));
    assert!(!raw.contains("openai_api_key"));
}

#[test]
fn migrates_legacy_plaintext_api_key_to_secret_store() {
    let dir = tempfile::tempdir().unwrap();
    let secrets: Arc<dyn SecretStore> = Arc::new(EncryptedFileSecretStore::new(
        dir.path().join("secrets.enc"),
    ));
    let path = dir.path().join("settings.json");
    std::fs::write(
        &path,
        r#"{"openaiApiKey":"sk-legacy","openaiModel":"gpt-test","useLocalOcr":true,"tesseractCommand":"tesseract"}"#,
    )
    .unwrap();
    let store = JsonSettingsStore::new(path.clone(), secrets.clone());
    let loaded = store.load_import_settings().unwrap();
    assert_eq!(loaded.openai_model, "gpt-test");
    assert!(loaded.openai_api_key.is_empty());
    assert_eq!(
        secrets.get_secret(OPENAI_API_KEY).unwrap(),
        Some("sk-legacy".to_owned())
    );
    let raw = std::fs::read_to_string(&path).unwrap();
    assert!(!raw.contains("sk-legacy"));
    assert!(!raw.contains("openaiApiKey"));
}
