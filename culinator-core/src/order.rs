use crate::{Operation, Symbol};
use std::collections::{BTreeMap, BTreeSet};

/// Dependency predecessors may be dotted paths ("prep.dice_onion"); operations
/// are matched on the final segment, mirroring the scheduler.
fn last_segment(value: &str) -> &str {
    value.rsplit('.').next().unwrap_or(value)
}

/// Topological sort on `after` dependencies for display purposes; parallel
/// steps tie-break by declaration order. Unknown predecessors are ignored, and
/// a cycle falls back to declaration order for the remaining operations.
/// Optional dependencies still order the display (this is reading order, not
/// scheduling).
pub fn sort_operations_for_display(operations: &[Operation]) -> Vec<&Operation> {
    if operations.len() <= 1 {
        return operations.iter().collect();
    }
    let symbols: BTreeSet<&str> = operations
        .iter()
        .map(|operation| operation.symbol.as_str())
        .collect();
    let mut placed: BTreeSet<&str> = BTreeSet::new();
    let mut result: Vec<&Operation> = Vec::with_capacity(operations.len());
    while result.len() < operations.len() {
        let ready: Vec<&Operation> = operations
            .iter()
            .filter(|operation| {
                !placed.contains(operation.symbol.as_str())
                    && operation.dependencies.iter().all(|dependency| {
                        let predecessor = last_segment(&dependency.predecessor);
                        !symbols.contains(predecessor) || placed.contains(predecessor)
                    })
            })
            .collect();
        if ready.is_empty() {
            result.extend(
                operations
                    .iter()
                    .filter(|operation| !placed.contains(operation.symbol.as_str())),
            );
            break;
        }
        for operation in ready {
            placed.insert(operation.symbol.as_str());
            result.push(operation);
        }
    }
    result
}

/// Transitive predecessor closure per operation symbol, computed over the
/// display order. Lets callers ask "must `b` wait for `a`?" when deciding
/// whether two steps can overlap.
pub fn transitive_predecessors(operations: &[Operation]) -> BTreeMap<Symbol, BTreeSet<Symbol>> {
    let mut map: BTreeMap<Symbol, BTreeSet<Symbol>> = BTreeMap::new();
    for operation in sort_operations_for_display(operations) {
        let mut predecessors = BTreeSet::new();
        for dependency in &operation.dependencies {
            let predecessor = last_segment(&dependency.predecessor).to_owned();
            if let Some(upstream) = map.get(&predecessor) {
                predecessors.extend(upstream.iter().cloned());
            }
            predecessors.insert(predecessor);
        }
        map.insert(operation.symbol.clone(), predecessors);
    }
    map
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Dependency, DependencyKind, TypeRef};
    use std::collections::BTreeMap;
    use uuid::Uuid;

    fn operation(symbol: &str, after: &[&str]) -> Operation {
        Operation {
            id: Uuid::new_v4(),
            symbol: symbol.into(),
            declared_type: TypeRef::named("Mix"),
            process: "main".into(),
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
            dependencies: after
                .iter()
                .map(|predecessor| Dependency {
                    predecessor: (*predecessor).into(),
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

    fn symbols(operations: &[&Operation]) -> Vec<String> {
        operations
            .iter()
            .map(|operation| operation.symbol.clone())
            .collect()
    }

    #[test]
    fn reorders_operations_declared_after_their_dependents() {
        let ops = vec![
            operation("bake", &["combine"]),
            operation("combine", &["boil"]),
            operation("boil", &[]),
        ];
        let sorted = sort_operations_for_display(&ops);
        assert_eq!(symbols(&sorted), vec!["boil", "combine", "bake"]);
    }

    #[test]
    fn parallel_steps_keep_declaration_order() {
        let ops = vec![
            operation("dice_onion", &[]),
            operation("dice_tomato", &[]),
            operation("fold", &["dice_onion", "dice_tomato"]),
            operation("mince_garlic", &[]),
        ];
        let sorted = sort_operations_for_display(&ops);
        assert_eq!(
            symbols(&sorted),
            vec!["dice_onion", "dice_tomato", "mince_garlic", "fold"]
        );
    }

    #[test]
    fn dotted_predecessors_match_on_last_segment() {
        let ops = vec![
            operation("fold", &["prep.dice_onion"]),
            operation("dice_onion", &[]),
        ];
        let sorted = sort_operations_for_display(&ops);
        assert_eq!(symbols(&sorted), vec!["dice_onion", "fold"]);
    }

    #[test]
    fn unknown_predecessors_are_ignored() {
        let ops = vec![operation("bake", &["missing"]), operation("boil", &[])];
        let sorted = sort_operations_for_display(&ops);
        assert_eq!(symbols(&sorted), vec!["bake", "boil"]);
    }

    #[test]
    fn cycle_falls_back_to_declaration_order() {
        let ops = vec![
            operation("a", &["b"]),
            operation("b", &["a"]),
            operation("c", &[]),
        ];
        let sorted = sort_operations_for_display(&ops);
        assert_eq!(symbols(&sorted), vec!["c", "a", "b"]);
    }

    #[test]
    fn transitive_predecessors_follow_chains() {
        let ops = vec![
            operation("boil", &[]),
            operation("drain", &["boil"]),
            operation("preheat", &[]),
            operation("combine", &["drain"]),
            operation("bake", &["combine", "preheat"]),
        ];
        let map = transitive_predecessors(&ops);
        assert!(map["combine"].contains("boil"));
        assert!(map["bake"].contains("drain"));
        assert!(map["bake"].contains("preheat"));
        assert!(!map["combine"].contains("preheat"));
        assert!(map["boil"].is_empty());
    }
}
