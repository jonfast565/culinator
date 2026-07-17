use crate::SearchService;
use culinator_models::{SearchHit, SearchQuery};
use std::sync::Arc;

struct StubSearch;

impl culinator_models::RecipeSearch for StubSearch {
    fn query(
        &self,
        _query: &SearchQuery,
    ) -> Result<Vec<SearchHit>, culinator_models::ApplicationError> {
        Ok(vec![SearchHit {
            recipe_id: uuid::Uuid::new_v4(),
            book_id: None,
            title: "Stub".to_owned(),
            snippet: "stub".to_owned(),
            score: 1.0,
        }])
    }
}

#[test]
fn query_delegates_to_repository() {
    let service = SearchService::new(Arc::new(StubSearch));
    let hits = service
        .query(&SearchQuery {
            text: Some("garlic".to_owned()),
            book_id: None,
            exclude_allergens: Vec::new(),
            max_active_minutes: None,
            hydration: None,
            limit: 10,
        })
        .expect("query");
    assert_eq!(hits.len(), 1);
}
