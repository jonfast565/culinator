use super::*;
use culinator_core::{Recipe, TypeRef};
use std::collections::BTreeMap;
use uuid::Uuid;
#[test]
fn has_header() {
    let r = Recipe {
        id: Uuid::new_v4(),
        book_id: None,
        symbol: "x".into(),
        declared_type: TypeRef::named("Recipe"),
        title: "X".into(),
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
    assert!(render(&r).starts_with("position,ingredient"));
}
