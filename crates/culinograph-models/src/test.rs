use super::*;

#[test]
fn exports_shared_contracts() {
    let error = ApplicationError::not_found("recipe");
    assert_eq!(error.to_string(), "recipe was not found");
}
