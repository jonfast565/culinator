use super::*;

#[test]
fn reexports_shared_models_for_compatibility() {
    let error = ApplicationError::not_found("recipe");
    assert_eq!(error.to_string(), "recipe was not found");
}
