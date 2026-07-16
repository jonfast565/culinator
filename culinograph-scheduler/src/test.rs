use super::*;
use culinograph_core::*;
use std::collections::BTreeMap;
use uuid::Uuid;
fn operation(symbol: &str, duration: u64, after: &[&str]) -> Operation {
    Operation {
        id: Uuid::new_v4(),
        symbol: symbol.into(),
        declared_type: TypeRef::named("Task"),
        process: "main".into(),
        labor: None,
        duration_min_seconds: Some(duration),
        duration_max_seconds: Some(duration),
        duration_estimated: false,
        target_temperature: None,
        heat_level: None,
        doneness: vec![],
        optional: false,
        dependencies: after
            .iter()
            .map(|p| Dependency {
                predecessor: (*p).into(),
                kind: DependencyKind::FinishStart,
                minimum_lag_seconds: None,
                maximum_lag_seconds: None,
                optional: false,
            })
            .collect(),
        bindings: vec![],
        requirements: vec![],
        effects: vec![],
        properties: BTreeMap::new(),
        span: None,
    }
}
fn recipe(operations: Vec<Operation>) -> Recipe {
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
        operations,
        servings: vec![],
        formulas: vec![],
        yields: vec![],
        properties: BTreeMap::new(),
    }
}
#[test]
fn schedules_parallel_branches() {
    let r = recipe(vec![
        operation("a", 10, &[]),
        operation("b", 20, &[]),
        operation("c", 5, &["a", "b"]),
    ]);
    let s = schedule_recipe(&r, &ScheduleOptions::default()).unwrap();
    assert_eq!(s.makespan_seconds, 25);
    assert_eq!(
        s.operations
            .iter()
            .find(|o| o.symbol == "c")
            .unwrap()
            .start_seconds,
        20
    );
}
#[test]
fn rejects_cycles() {
    let r = recipe(vec![operation("a", 10, &["b"]), operation("b", 10, &["a"])]);
    assert!(schedule_recipe(&r, &ScheduleOptions::default()).is_err());
}
