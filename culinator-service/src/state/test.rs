use super::*;
#[test]
fn opens_and_migrates_database() {
    let path =
        std::env::temp_dir().join(format!("culinator-state-{}.sqlite3", uuid::Uuid::new_v4()));
    let state =
        ServiceState::sqlite(path.clone(), path.with_file_name("settings.json")).expect("state");
    let _ = state.recipes();
    let _ = std::fs::remove_file(path);
}

#[test]
fn initialize_makes_nutrition_catalog_available() {
    let path =
        std::env::temp_dir().join(format!("culinator-init-{}.sqlite3", uuid::Uuid::new_v4()));
    let state =
        ServiceState::sqlite(path.clone(), path.with_file_name("settings.json")).expect("state");
    let report = state.initialize().expect("initialize");
    assert!(
        report.nutrition_ready,
        "starter nutrition catalog is available"
    );
    assert!(
        report.nutrition_starter,
        "full USDA import is still pending"
    );
    let fdc_path = path.with_file_name("fdc.sqlite3");
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(fdc_path);
}

#[test]
fn seeds_sample_recipes_only_once() {
    let path =
        std::env::temp_dir().join(format!("culinator-seed-{}.sqlite3", uuid::Uuid::new_v4()));
    let state =
        ServiceState::sqlite(path.clone(), path.with_file_name("settings.json")).expect("state");
    state.seed_if_empty().expect("first seed succeeds");
    let after_first = state.recipes().list().expect("list recipes");
    assert_eq!(
        after_first.len(),
        SEED_RECIPES.len() + CROWDED_KITCHEN_RECIPES.len(),
        "both books are populated"
    );
    let books = state.books().list().expect("list books");
    assert_eq!(books.len(), 2);
    assert_eq!(
        books
            .iter()
            .find(|book| book.symbol == "sample_recipes")
            .expect("sample book")
            .recipe_count,
        SEED_RECIPES.len() as i64
    );
    assert_eq!(
        books
            .iter()
            .find(|book| book.symbol == "crowded_kitchen_better_than_store_bought")
            .expect("Crowded Kitchen book")
            .recipe_count,
        CROWDED_KITCHEN_RECIPES.len() as i64
    );
    // Seeding again must be a no-op once the catalog is non-empty.
    state.seed_if_empty().expect("second seed is a no-op");
    let after_second = state.recipes().list().expect("list recipes");
    assert_eq!(
        after_second.len(),
        SEED_RECIPES.len() + CROWDED_KITCHEN_RECIPES.len(),
        "re-seeding does not duplicate"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn every_seed_declares_a_book_section_and_a_source() {
    // The book TOC groups recipes by `section`; a seed without one falls into an
    // unlabeled chapter. The source credit is a licensing requirement, not a nicety.
    for source in SEED_RECIPES.iter().chain(CROWDED_KITCHEN_RECIPES.iter()) {
        let title = source
            .lines()
            .find_map(|line| line.trim().strip_prefix("title "))
            .unwrap_or("<untitled>");
        assert!(
            source.contains("section \""),
            "{title} is missing a `section` chapter"
        );
        assert!(
            source.contains("attribution \""),
            "{title} is missing an attribution credit"
        );
    }
}

#[test]
fn every_registered_seed_passes_recipe_validation() {
    let path = std::env::temp_dir().join(format!(
        "culinator-validate-{}.sqlite3",
        uuid::Uuid::new_v4()
    ));
    let state =
        ServiceState::sqlite(path.clone(), path.with_file_name("settings.json")).expect("state");
    for source in SEED_RECIPES.iter().chain(CROWDED_KITCHEN_RECIPES.iter()) {
        let report = state.recipes().validate_source(source);
        assert!(
            report.valid,
            "seed failed validation: {:?}",
            report.diagnostics
        );
    }
    let _ = std::fs::remove_file(path);
}

#[test]
fn seed_recipes_schedule_without_unresolved_dependencies() {
    let path =
        std::env::temp_dir().join(format!("culinator-sched-{}.sqlite3", uuid::Uuid::new_v4()));
    let state =
        ServiceState::sqlite(path.clone(), path.with_file_name("settings.json")).expect("state");
    let options = culinator_models::ScheduleOptions::default();
    for source in SEED_RECIPES {
        // A cycle or an unknown predecessor (e.g. a downstream `after` that no
        // longer matches a `prep`-desugared operation symbol) surfaces here.
        let schedule = state
            .schedules()
            .schedule_source(source, &options)
            .expect("seed recipe schedules cleanly");
        assert!(schedule.makespan_seconds > 0, "recipe has a real makespan");
    }
    let _ = std::fs::remove_file(path);
}
