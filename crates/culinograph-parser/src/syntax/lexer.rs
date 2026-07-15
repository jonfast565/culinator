use std::ops::Range;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl TextRange {
    pub const fn new(start: usize, end: usize) -> Self { Self { start, end } }
    pub const fn len(self) -> usize { self.end - self.start }
    pub const fn is_empty(self) -> bool { self.start == self.end }
    pub fn as_range(self) -> Range<usize> { self.start..self.end }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SyntaxKind {
    Whitespace,
    LineComment,
    BlockComment,
    Identifier,
    Number,
    Percent,
    String,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Less,
    Greater,
    Comma,
    Semicolon,
    Equals,
    Dot,
    Colon,
    Plus,
    Minus,
    Star,
    Slash,
    Unknown,
}

impl SyntaxKind {
    pub const fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace | Self::LineComment | Self::BlockComment)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxToken {
    pub kind: SyntaxKind,
    pub range: TextRange,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SyntaxError {
    #[error("unterminated string at byte {0}")]
    UnterminatedString(usize),
    #[error("unterminated block comment at byte {0}")]
    UnterminatedBlockComment(usize),
    #[error("invalid edit range {start}..{end} for source length {source_len}")]
    InvalidEditRange { start: usize, end: usize, source_len: usize },
    #[error("overlapping edits at byte {0}")]
    OverlappingEdits(usize),
    #[error("unclosed delimiter `{delimiter}` at byte {offset}")]
    UnclosedDelimiter { delimiter: char, offset: usize },
    #[error("unexpected closing delimiter `{delimiter}` at byte {offset}")]
    UnexpectedClosingDelimiter { delimiter: char, offset: usize },
}

pub fn lex_lossless(source: &str) -> Result<Vec<SyntaxToken>, SyntaxError> {
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut tokens = Vec::new();
    while i < bytes.len() {
        let start = i;
        let kind = match bytes[i] {
            b if b.is_ascii_whitespace() => {
                i += 1;
                while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
                SyntaxKind::Whitespace
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' { i += 1; }
                SyntaxKind::LineComment
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i += 2;
                let mut closed = false;
                while i + 1 < bytes.len() {
                    if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        i += 2;
                        closed = true;
                        break;
                    }
                    i += 1;
                }
                if !closed { return Err(SyntaxError::UnterminatedBlockComment(start)); }
                SyntaxKind::BlockComment
            }
            b'"' => {
                i += 1;
                let mut closed = false;
                while i < bytes.len() {
                    match bytes[i] {
                        b'\\' => { i += 1; if i < bytes.len() { i += 1; } }
                        b'"' => { i += 1; closed = true; break; }
                        _ => i += 1,
                    }
                }
                if !closed { return Err(SyntaxError::UnterminatedString(start)); }
                SyntaxKind::String
            }
            b if b.is_ascii_alphabetic() || b == b'_' => {
                i += 1;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'_' | b'-')) { i += 1; }
                SyntaxKind::Identifier
            }
            b if b.is_ascii_digit() => {
                i += 1;
                while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') { i += 1; }
                if bytes.get(i) == Some(&b'%') { i += 1; SyntaxKind::Percent } else { SyntaxKind::Number }
            }
            b'{' => { i += 1; SyntaxKind::LBrace }
            b'}' => { i += 1; SyntaxKind::RBrace }
            b'[' => { i += 1; SyntaxKind::LBracket }
            b']' => { i += 1; SyntaxKind::RBracket }
            b'(' => { i += 1; SyntaxKind::LParen }
            b')' => { i += 1; SyntaxKind::RParen }
            b'<' => { i += 1; SyntaxKind::Less }
            b'>' => { i += 1; SyntaxKind::Greater }
            b',' => { i += 1; SyntaxKind::Comma }
            b';' => { i += 1; SyntaxKind::Semicolon }
            b'=' => { i += 1; SyntaxKind::Equals }
            b'.' => { i += 1; SyntaxKind::Dot }
            b':' => { i += 1; SyntaxKind::Colon }
            b'+' => { i += 1; SyntaxKind::Plus }
            b'-' => { i += 1; SyntaxKind::Minus }
            b'*' => { i += 1; SyntaxKind::Star }
            b'/' => { i += 1; SyntaxKind::Slash }
            _ => {
                // Preserve one full UTF-8 scalar as an unknown token.
                let width = source[i..].chars().next().map(char::len_utf8).unwrap_or(1);
                i += width;
                SyntaxKind::Unknown
            }
        };
        tokens.push(SyntaxToken {
            kind,
            range: TextRange::new(start, i),
            text: source[start..i].to_owned(),
        });
    }
    Ok(tokens)
}


#[cfg(test)]
mod test;
