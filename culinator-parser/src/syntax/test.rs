use super::*;

#[test]
fn round_trip_preserves_comments_whitespace_and_unknown_tokens() {
    let source =
        "culinator 0.3;\n/* heading */\nrecipe x {\n  // note\n  custom café @value;\n}\n";
    let document = LosslessDocument::parse(source).expect("lossless parse");
    assert_eq!(document.round_trip(), source);
    assert_eq!(document.root().text(), source);
    assert!(
        document
            .tokens()
            .iter()
            .any(|token| token.kind == SyntaxKind::BlockComment)
    );
    assert!(
        document
            .tokens()
            .iter()
            .any(|token| token.kind == SyntaxKind::Unknown)
    );
}

#[test]
fn tree_preserves_nested_delimiters() {
    let source = "recipe x { values [a, fn(b)]; type T<A>; }";
    let document = LosslessDocument::parse(source).expect("tree");
    assert_eq!(document.root().text(), source);
}

#[test]
fn edits_preserve_untouched_bytes() {
    let source = "recipe x { title \"Old\"; // keep\n}";
    let start = source.find("Old").unwrap();
    let edited = apply_text_edits(
        source,
        &[TextEdit::replace(TextRange::new(start, start + 3), "New")],
    )
    .unwrap();
    assert_eq!(edited, "recipe x { title \"New\"; // keep\n}");
}

#[test]
fn rejects_overlapping_edits() {
    let edits = [
        TextEdit::delete(TextRange::new(0, 2)),
        TextEdit::delete(TextRange::new(1, 3)),
    ];
    assert!(matches!(
        apply_text_edits("abcd", &edits),
        Err(SyntaxError::OverlappingEdits(1))
    ));
}
