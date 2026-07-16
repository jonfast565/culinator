use culinator_core::Recipe;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
}

pub fn validate(recipe: &Recipe) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let resources: HashSet<_> = recipe.resources.iter().map(|r| r.symbol.as_str()).collect();
    let operations: HashSet<_> = recipe
        .operations
        .iter()
        .map(|o| o.symbol.as_str())
        .collect();

    for op in &recipe.operations {
        for binding in &op.bindings {
            if !resources.contains(binding.resource.as_str()) {
                out.push(Diagnostic {
                    code: "CG1001",
                    message: format!(
                        "operation `{}` references unknown resource `{}`",
                        op.symbol, binding.resource
                    ),
                });
            }
        }
        for dependency in &op.dependencies {
            if !operations.contains(dependency.predecessor.as_str()) {
                out.push(Diagnostic {
                    code: "CG1002",
                    message: format!(
                        "operation `{}` depends on unknown operation `{}`",
                        op.symbol, dependency.predecessor
                    ),
                });
            }
        }
    }

    detect_cycles(recipe, &mut out);
    out
}

fn detect_cycles(recipe: &Recipe, out: &mut Vec<Diagnostic>) {
    let graph: HashMap<_, Vec<_>> = recipe
        .operations
        .iter()
        .map(|o| {
            (
                o.symbol.as_str(),
                o.dependencies
                    .iter()
                    .map(|d| d.predecessor.as_str())
                    .collect(),
            )
        })
        .collect();
    let mut visiting = HashSet::new();
    let mut visited = HashSet::new();
    for node in graph.keys().copied() {
        if visit(node, &graph, &mut visiting, &mut visited) {
            out.push(Diagnostic {
                code: "CG2001",
                message: format!("dependency cycle includes `{node}`"),
            });
            break;
        }
    }
}

fn visit<'a>(
    node: &'a str,
    graph: &HashMap<&'a str, Vec<&'a str>>,
    visiting: &mut HashSet<&'a str>,
    visited: &mut HashSet<&'a str>,
) -> bool {
    if visiting.contains(node) {
        return true;
    }
    if visited.contains(node) {
        return false;
    }
    visiting.insert(node);
    if graph
        .get(node)
        .is_some_and(|deps| deps.iter().any(|d| visit(d, graph, visiting, visited)))
    {
        return true;
    }
    visiting.remove(node);
    visited.insert(node);
    false
}
#[derive(Debug, Default, Clone, Copy)]
pub struct CulinatorValidator;

impl culinator_models::RecipeValidator for CulinatorValidator {
    fn validate(&self, recipe: &Recipe) -> Vec<culinator_models::SourceDiagnostic> {
        validate(recipe)
            .into_iter()
            .map(|diagnostic| culinator_models::SourceDiagnostic {
                code: diagnostic.code.to_owned(),
                severity: if diagnostic.code.starts_with("CG2") {
                    culinator_models::DiagnosticSeverity::Error
                } else {
                    culinator_models::DiagnosticSeverity::Warning
                },
                message: diagnostic.message,
                start: None,
                end: None,
            })
            .collect()
    }
}

#[cfg(test)]
mod test;
