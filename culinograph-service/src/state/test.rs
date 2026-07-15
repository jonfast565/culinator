use super::*;
#[test]
fn opens_and_migrates_database() {
    let path = std::env::temp_dir().join(format!(
        "culinograph-state-{}.sqlite3",
        uuid::Uuid::new_v4()
    ));
    let state =
        ServiceState::sqlite(path.clone(), path.with_file_name("settings.json")).expect("state");
    let _ = state.recipes();
    let _ = std::fs::remove_file(path);
}

#[test]
fn seeds_sample_recipes_only_once() {
    let path =
        std::env::temp_dir().join(format!("culinograph-seed-{}.sqlite3", uuid::Uuid::new_v4()));
    let state =
        ServiceState::sqlite(path.clone(), path.with_file_name("settings.json")).expect("state");
    state.seed_if_empty().expect("first seed succeeds");
    let after_first = state.recipes().list().expect("list recipes");
    assert_eq!(
        after_first.len(),
        SEED_RECIPES.len(),
        "every sample recipe is created"
    );
    // Seeding again must be a no-op once the catalog is non-empty.
    state.seed_if_empty().expect("second seed is a no-op");
    let after_second = state.recipes().list().expect("list recipes");
    assert_eq!(
        after_second.len(),
        SEED_RECIPES.len(),
        "re-seeding does not duplicate"
    );
    let _ = std::fs::remove_file(path);
}
