use crate::{ApplicationError, SearchHit, SearchQuery};

/// Full-text and structured recipe discovery.
pub trait RecipeSearch: Send + Sync {
    fn query(&self, query: &SearchQuery) -> Result<Vec<SearchHit>, ApplicationError>;
}

#[cfg(test)]
#[path = "search/test.rs"]
mod test;
