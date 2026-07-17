use culinator_core::{BindingRole, DependencyKind, Operation, Recipe};
use culinator_models::{
    ApplicationError, RecipeSchedule, RecipeScheduler, ScheduleOptions, ScheduledOperation,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub struct CriticalPathScheduler;

impl RecipeScheduler for CriticalPathScheduler {
    fn schedule(
        &self,
        recipe: &Recipe,
        options: &ScheduleOptions,
    ) -> Result<RecipeSchedule, ApplicationError> {
        schedule_recipe(recipe, options)
    }
}

pub fn schedule_recipe(
    recipe: &Recipe,
    options: &ScheduleOptions,
) -> Result<RecipeSchedule, ApplicationError> {
    let by_symbol: BTreeMap<_, _> = recipe
        .operations
        .iter()
        .map(|op| (op.symbol.as_str(), op))
        .collect();
    let mut remaining: BTreeSet<_> = by_symbol.keys().copied().collect();
    let mut result: BTreeMap<String, ScheduledOperation> = BTreeMap::new();

    while !remaining.is_empty() {
        let ready: Vec<_> = remaining
            .iter()
            .copied()
            .filter(|symbol| {
                by_symbol[*symbol]
                    .dependencies
                    .iter()
                    .filter(|d| !d.optional)
                    .all(|d| result.contains_key(last_segment(&d.predecessor)))
            })
            .collect();
        if ready.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "Operation dependency graph contains a cycle or an unknown predecessor".into(),
            ));
        }
        for symbol in ready {
            let operation = by_symbol[symbol];
            let duration = duration_seconds(operation, options.default_duration_seconds);
            let dependency_start = operation
                .dependencies
                .iter()
                .filter(|d| !d.optional)
                .map(|dependency| {
                    let predecessor = &result[last_segment(&dependency.predecessor)];
                    let lag = dependency.minimum_lag_seconds.unwrap_or(0);
                    match dependency.kind {
                        DependencyKind::FinishStart => predecessor.end_seconds + lag,
                        DependencyKind::StartStart => predecessor.start_seconds + lag,
                        DependencyKind::FinishFinish => predecessor
                            .end_seconds
                            .saturating_add(lag)
                            .saturating_sub(duration),
                        DependencyKind::StartFinish => predecessor
                            .start_seconds
                            .saturating_add(lag)
                            .saturating_sub(duration),
                    }
                })
                .max()
                .unwrap_or(0);
            let mut start = dependency_start;
            loop {
                let conflict_end = result
                    .values()
                    .filter(|existing| {
                        overlaps(
                            start,
                            start + duration,
                            existing.start_seconds,
                            existing.end_seconds,
                        )
                    })
                    .filter(|existing| {
                        has_resource_conflict(operation, by_symbol[existing.symbol.as_str()])
                    })
                    .map(|existing| existing.end_seconds)
                    .max();
                match conflict_end {
                    Some(end) if end > start => start = end,
                    _ => break,
                }
            }
            result.insert(
                symbol.to_owned(),
                ScheduledOperation {
                    symbol: symbol.to_owned(),
                    process: operation.process.clone(),
                    action: operation.declared_type.name.clone(),
                    start_seconds: start,
                    end_seconds: start + duration,
                    duration_seconds: duration,
                    labor: operation.labor,
                    dependencies: operation
                        .dependencies
                        .iter()
                        .map(|d| last_segment(&d.predecessor).to_owned())
                        .collect(),
                    resources: operation
                        .bindings
                        .iter()
                        .map(|b| b.resource.clone())
                        .collect(),
                },
            );
            remaining.remove(symbol);
        }
    }
    let makespan_seconds = result
        .values()
        .map(|item| item.end_seconds)
        .max()
        .unwrap_or(0);
    let mut operations: Vec<_> = result.into_values().collect();
    operations.sort_by_key(|item| (item.start_seconds, item.end_seconds, item.symbol.clone()));
    let critical = critical_path(&operations, makespan_seconds);
    Ok(RecipeSchedule {
        operations,
        makespan_seconds,
        critical_path: critical,
    })
}

fn duration_seconds(operation: &Operation, fallback: u64) -> u64 {
    let per_repetition = operation
        .duration_max_seconds
        .or(operation.duration_min_seconds)
        .unwrap_or(fallback)
        .max(1);
    // `repeat` batches the step: the authored duration is per pass, so the
    // wall-clock cost is `duration * repeat` (e.g. cooking crepes one at a time).
    per_repetition.saturating_mul(operation.repeat.unwrap_or(1).max(1) as u64)
}
fn last_segment(value: &str) -> &str {
    value.rsplit('.').next().unwrap_or(value)
}
fn overlaps(a_start: u64, a_end: u64, b_start: u64, b_end: u64) -> bool {
    a_start < b_end && b_start < a_end
}
fn has_resource_conflict(left: &Operation, right: &Operation) -> bool {
    left.bindings.iter().any(|a| {
        right.bindings.iter().any(|b| {
            a.resource == b.resource
                && (a.exclusive
                    || b.exclusive
                    || matches!(
                        a.role,
                        BindingRole::Equipment | BindingRole::Labor | BindingRole::Container
                    )
                    || matches!(
                        b.role,
                        BindingRole::Equipment | BindingRole::Labor | BindingRole::Container
                    ))
        })
    })
}
fn critical_path(operations: &[ScheduledOperation], makespan: u64) -> Vec<String> {
    let mut path: Vec<_> = operations
        .iter()
        .filter(|item| item.end_seconds == makespan)
        .map(|item| item.symbol.clone())
        .collect();
    let by_symbol: BTreeMap<_, _> = operations
        .iter()
        .map(|item| (item.symbol.as_str(), item))
        .collect();
    let mut cursor = path.first().cloned();
    while let Some(symbol) = cursor {
        let item = by_symbol[symbol.as_str()];
        let previous = item
            .dependencies
            .iter()
            .filter_map(|dep| by_symbol.get(dep.as_str()))
            .filter(|dep| dep.end_seconds == item.start_seconds)
            .max_by_key(|dep| dep.start_seconds);
        if let Some(previous) = previous {
            path.push(previous.symbol.clone());
            cursor = Some(previous.symbol.clone());
        } else {
            break;
        }
    }
    path.reverse();
    path
}

#[cfg(test)]
mod test;
