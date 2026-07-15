use super::*;
use culinograph_core::{Recipe, TypeRef};
use std::collections::BTreeMap;
use uuid::Uuid;

#[test]
fn empty_recipe_has_empty_content() {
    let recipe = Recipe {
        id: Uuid::new_v4(),
        book_id: None,
        symbol: "empty".into(),
        declared_type: TypeRef::named("Recipe"),
        title: "Empty".into(),
        protocol_version: "0.3".into(),
        types: vec![],
        resources: vec![],
        processes: vec![],
        operations: vec![],
        servings: vec![],
        formulas: vec![],
        yields: vec![],
        properties: BTreeMap::new(),
    };
    let content = extract(&recipe);
    assert!(content.ingredients.is_empty());
    assert!(content.instructions.is_empty());
}
