use crate::SourceDiagnostic;
use culinograph_core::Recipe;

pub trait RecipeValidator: Send + Sync {
    fn validate(&self, recipe: &Recipe) -> Vec<SourceDiagnostic>;
}

#[cfg(test)]
mod test;
