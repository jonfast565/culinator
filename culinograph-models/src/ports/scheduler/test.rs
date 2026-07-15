use super::*;
fn assert_object_safe(_: &dyn RecipeScheduler) {}
#[test]
fn trait_is_object_safe() {
    let _ = assert_object_safe;
}
