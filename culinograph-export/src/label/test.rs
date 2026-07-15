#[test]
fn label_contains_heading() {
    assert!(super::render(&Default::default()).contains("Nutrition Facts"));
}
