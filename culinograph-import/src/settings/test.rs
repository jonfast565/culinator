use super::*;
#[test]
fn settings_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let store = JsonSettingsStore::new(dir.path().join("settings.json"));
    let value = ImportSettings {
        openai_model: "test".into(),
        ..ImportSettings::default()
    };
    store.save_import_settings(&value).unwrap();
    assert_eq!(store.load_import_settings().unwrap().openai_model, "test");
}
