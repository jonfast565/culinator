use super::*;
use culinator_models::{NewRecipeBook, RecipeBookRepository};

#[test]
fn initializes_and_creates_a_book() {
    let path = std::env::temp_dir().join(format!("culinator-{}.sqlite3", uuid::Uuid::new_v4()));
    let repository = SqliteCatalogRepository::new(&path);
    repository.initialize().expect("database initialization");
    let created = repository
        .create_recipe_book(NewRecipeBook {
            title: "Test Book".to_owned(),
            symbol: None,
            description: None,
        })
        .expect("book creation");
    assert_eq!(created.symbol, "test_book");
    let _ = std::fs::remove_file(path);
}
