use super::*;
use culinator_core::{Recipe, TypeRef};
use culinator_models::BookExportOptions;
use std::collections::BTreeMap;
use uuid::Uuid;

#[test]
fn print_html_includes_title_toc_and_page_breaks() {
    let breakfast = Recipe {
        id: Uuid::new_v4(),
        book_id: None,
        symbol: "crepes".into(),
        declared_type: TypeRef::named("Recipe"),
        title: "Easy Crepes".into(),
        protocol_version: "0.3".into(),
        types: vec![],
        resources: vec![],
        processes: vec![],
        operations: vec![],
        servings: vec![],
        formulas: vec![],
        yields: vec![],
        properties: BTreeMap::from([(
            "section".into(),
            culinator_core::Value::Text("Breakfast".into()),
        )]),
    };
    let mut starters = breakfast.clone();
    starters.id = Uuid::new_v4();
    starters.symbol = "guac".into();
    starters.title = "Guacamole".into();
    starters.properties.insert(
        "section".into(),
        culinator_core::Value::Text("Starters".into()),
    );
    let html = render(
        "Sample Book",
        &[(breakfast, String::new()), (starters, String::new())],
        &BookExportOptions::default(),
    );
    assert!(html.contains("Sample Book"));
    assert!(html.contains("Contents"));
    assert!(html.contains("@media print"));
    assert!(html.contains("page-break-before:always"));
    assert!(html.contains("Easy Crepes"));
    assert!(html.contains("Guacamole"));
    assert!(html.contains("Breakfast"));
    assert!(html.contains("Starters"));
}
