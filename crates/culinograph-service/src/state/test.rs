use super::*;
#[test] fn opens_and_migrates_database() { let path=std::env::temp_dir().join(format!("culinograph-state-{}.sqlite3",uuid::Uuid::new_v4())); let state=ServiceState::sqlite(path.clone(),path.with_file_name("settings.json")).expect("state"); let _=state.recipes(); let _=std::fs::remove_file(path); }
