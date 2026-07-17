use super::*;
use culinator_core::*;
use std::collections::BTreeMap;
use uuid::Uuid;

fn recipe() -> Recipe {
    Recipe {
        id: Uuid::new_v4(),
        book_id: None,
        symbol: "r".into(),
        declared_type: TypeRef::named("Recipe"),
        title: "R".into(),
        protocol_version: "0.3".into(),
        types: vec![],
        resources: vec![],
        processes: vec![],
        operations: vec![],
        servings: vec![],
        formulas: vec![],
        yields: vec![],
        properties: BTreeMap::new(),
    }
}
#[test]
fn accepts_empty_graph() {
    assert!(validate(&recipe()).is_empty());
}
#[test]
fn finds_unknown_dependency() {
    let mut r = recipe();
    r.operations.push(Operation {
        id: Uuid::new_v4(),
        symbol: "b".into(),
        declared_type: TypeRef::named("Operation"),
        process: "p".into(),
        labor: None,
        duration_min_seconds: None,
        duration_max_seconds: None,
        duration_estimated: false,
        target_temperature: None,
        heat_level: None,
        doneness: vec![],
        optional: false,
        repeat: None,
        notes: vec![],
        dependencies: vec![Dependency {
            predecessor: "missing".into(),
            kind: DependencyKind::FinishStart,
            minimum_lag_seconds: None,
            maximum_lag_seconds: None,
            optional: false,
        }],
        bindings: vec![],
        requirements: vec![],
        effects: vec![],
        properties: BTreeMap::new(),
        span: None,
    });
    assert!(validate(&r).iter().any(|d| d.code == "CG1002"));
}
