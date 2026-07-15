#[test]
fn escapes_html() {
    assert_eq!(super::escape("a<b"), "a&lt;b");
}
