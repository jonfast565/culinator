use crate::RecipeSearch;

struct Stub;

impl RecipeSearch for Stub {
    fn query(
        &self,
        _query: &crate::SearchQuery,
    ) -> Result<Vec<crate::SearchHit>, crate::ApplicationError> {
        Ok(Vec::new())
    }
}

#[test]
fn trait_is_object_safe() {
    let _: &dyn RecipeSearch = &Stub;
}
