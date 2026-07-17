use super::*;
use culinator_core::{Recipe, TypeRef, Value};
use culinator_models::BookExportOptions;
use std::collections::BTreeMap;
use uuid::Uuid;

fn sample_recipe(title: &str, section: &str) -> Recipe {
    Recipe {
        id: Uuid::new_v4(),
        book_id: None,
        symbol: title.to_lowercase().replace(' ', "_"),
        declared_type: TypeRef::named("Recipe"),
        title: title.to_owned(),
        protocol_version: "0.3".into(),
        types: vec![],
        resources: vec![],
        processes: vec![],
        operations: vec![],
        servings: vec![],
        formulas: vec![],
        yields: vec![],
        properties: BTreeMap::from([("section".into(), Value::Text(section.to_owned()))]),
    }
}

#[test]
fn epub_contains_nav_and_recipe_documents() {
    let recipes = vec![
        (sample_recipe("Easy Crepes", "Breakfast"), String::new()),
        (sample_recipe("Guacamole", "Starters"), String::new()),
    ];
    let bytes = render("Sample Book", &recipes, &BookExportOptions::default()).unwrap();
    let archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let names = archive.file_names().map(str::to_owned).collect::<Vec<_>>();
    assert!(names.iter().any(|name| name == "OEBPS/nav.xhtml"));
    assert!(
        names
            .iter()
            .any(|name| name.contains("recipes/easy-crepes.xhtml"))
    );
    assert!(
        names
            .iter()
            .any(|name| name.contains("recipes/guacamole.xhtml"))
    );
    assert!(
        names
            .iter()
            .any(|name| name.contains("sections/breakfast.xhtml"))
    );
}

#[test]
fn epub_starts_with_zip_signature() {
    let recipes = vec![(sample_recipe("Tea", "Recipes"), String::new())];
    let bytes = render("Tea Book", &recipes, &BookExportOptions::default()).unwrap();
    assert_eq!(&bytes[..2], b"PK");
}
