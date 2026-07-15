use super::*;

#[test]
fn builds_balanced_lossless_tree() {
    let source = "x { y [z]; }";
    let document = LosslessDocument::parse(source).unwrap();
    assert_eq!(document.root().text(), source);
    assert_eq!(document.round_trip(), source);
}

#[test]
fn rejects_unbalanced_tree() {
    assert!(matches!(
        LosslessDocument::parse("x {"),
        Err(SyntaxError::UnclosedDelimiter { .. })
    ));
}
