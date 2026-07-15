use super::*;

#[test]
fn applies_multiple_edits_in_source_order() {
    let edits = [
        TextEdit::replace(TextRange::new(0, 1), "A"),
        TextEdit::insert(3, "!"),
    ];
    assert_eq!(apply_text_edits("abc", &edits).unwrap(), "Abc!");
}
