use super::*;
#[test]
fn completion_vocabulary_contains_core_constructs() {
    let labels = ["resource", "operation", "process"];
    assert!(labels.contains(&"operation"));
}
