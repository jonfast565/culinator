use crate::{ApplicationError, RecipeSearch, SearchHit, SearchQuery};
use std::sync::Arc;

#[derive(Clone)]
pub struct SearchService {
    repository: Arc<dyn RecipeSearch>,
}

impl SearchService {
    pub fn new(repository: Arc<dyn RecipeSearch>) -> Self {
        Self { repository }
    }

    pub fn query(&self, query: &SearchQuery) -> Result<Vec<SearchHit>, ApplicationError> {
        self.repository.query(query)
    }
}

#[cfg(test)]
#[path = "search_service/test.rs"]
mod search_service_test;
