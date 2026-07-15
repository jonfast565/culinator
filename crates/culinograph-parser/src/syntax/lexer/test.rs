use super::*;

#[test]
fn lexer_covers_every_source_byte() {
    let source = " // x\nname café 12.5% /* y */";
    let tokens = lex_lossless(source).expect("tokens");
    assert_eq!(tokens.iter().map(|token| token.text.as_str()).collect::<String>(), source);
    assert_eq!(tokens.first().unwrap().range.start, 0);
    assert_eq!(tokens.last().unwrap().range.end, source.len());
}

#[test]
fn lexer_reports_unterminated_constructs() {
    assert!(matches!(lex_lossless("/*"), Err(SyntaxError::UnterminatedBlockComment(0))));
    assert!(matches!(lex_lossless("\"x"), Err(SyntaxError::UnterminatedString(0))));
}
