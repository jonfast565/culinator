use crate::SourceDiagnostic;
use culinator_core::Recipe;

pub trait RecipeValidator: Send + Sync {
    fn validate(&self, recipe: &Recipe) -> Vec<SourceDiagnostic>;
}

#[cfg(test)]
mod test;
