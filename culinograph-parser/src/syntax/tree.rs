use super::{lex_lossless, SyntaxError, SyntaxKind, SyntaxToken, TextRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CstNodeKind {
    Document,
    Block,
    List,
    Parenthesized,
    GenericArguments,
    Statement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CstElement {
    Node(CstNode),
    Token(SyntaxToken),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CstNode {
    pub kind: CstNodeKind,
    pub range: TextRange,
    pub children: Vec<CstElement>,
}

impl CstNode {
    pub fn tokens(&self) -> impl Iterator<Item = &SyntaxToken> {
        self.children.iter().flat_map(|element| match element {
            CstElement::Token(token) => TokenIter::One(Some(token)),
            CstElement::Node(node) => TokenIter::Many(Box::new(node.tokens())),
        })
    }

    pub fn text(&self) -> String { self.tokens().map(|token| token.text.as_str()).collect() }
}

enum TokenIter<'a> {
    One(Option<&'a SyntaxToken>),
    Many(Box<dyn Iterator<Item = &'a SyntaxToken> + 'a>),
}
impl<'a> Iterator for TokenIter<'a> {
    type Item = &'a SyntaxToken;
    fn next(&mut self) -> Option<Self::Item> {
        match self { Self::One(value) => value.take(), Self::Many(iter) => iter.next() }
    }
}

#[derive(Debug, Clone)]
pub struct LosslessDocument {
    source: String,
    tokens: Vec<SyntaxToken>,
    root: CstNode,
}

impl LosslessDocument {
    pub fn parse(source: &str) -> Result<Self, SyntaxError> {
        let tokens = lex_lossless(source)?;
        let root = build_tree(&tokens, source.len())?;
        Ok(Self { source: source.to_owned(), tokens, root })
    }
    pub fn source(&self) -> &str { &self.source }
    pub fn tokens(&self) -> &[SyntaxToken] { &self.tokens }
    pub fn root(&self) -> &CstNode { &self.root }
    pub fn round_trip(&self) -> String { self.tokens.iter().map(|token| token.text.as_str()).collect() }
    pub fn token_at(&self, offset: usize) -> Option<&SyntaxToken> {
        self.tokens.iter().find(|token| token.range.start <= offset && offset < token.range.end)
    }
    pub fn non_trivia_tokens(&self) -> impl Iterator<Item = &SyntaxToken> {
        self.tokens.iter().filter(|token| !token.kind.is_trivia())
    }
}

fn build_tree(tokens: &[SyntaxToken], source_len: usize) -> Result<CstNode, SyntaxError> {
    #[derive(Debug)]
    struct Frame { kind: CstNodeKind, start: usize, open: Option<(char, usize)>, children: Vec<CstElement> }
    let mut stack = vec![Frame { kind: CstNodeKind::Document, start: 0, open: None, children: Vec::new() }];
    for token in tokens.iter().cloned() {
        let opener = match token.kind {
            SyntaxKind::LBrace => Some((CstNodeKind::Block, '{', '}')),
            SyntaxKind::LBracket => Some((CstNodeKind::List, '[', ']')),
            SyntaxKind::LParen => Some((CstNodeKind::Parenthesized, '(', ')')),
            SyntaxKind::Less => Some((CstNodeKind::GenericArguments, '<', '>')),
            _ => None,
        };
        if let Some((kind, open, _)) = opener {
            let start = token.range.start;
            stack.push(Frame { kind, start, open: Some((open, start)), children: vec![CstElement::Token(token)] });
            continue;
        }
        let closer = match token.kind { SyntaxKind::RBrace => Some('}'), SyntaxKind::RBracket => Some(']'), SyntaxKind::RParen => Some(')'), SyntaxKind::Greater => Some('>'), _ => None };
        if let Some(close) = closer {
            if stack.len() == 1 { return Err(SyntaxError::UnexpectedClosingDelimiter { delimiter: close, offset: token.range.start }); }
            let mut frame = stack.pop().expect("non-root frame");
            let expected = match frame.open.expect("nested frame").0 { '{' => '}', '[' => ']', '(' => ')', '<' => '>', _ => unreachable!() };
            if close != expected { return Err(SyntaxError::UnexpectedClosingDelimiter { delimiter: close, offset: token.range.start }); }
            let end = token.range.end;
            frame.children.push(CstElement::Token(token));
            let node = CstNode { kind: frame.kind, range: TextRange::new(frame.start, end), children: frame.children };
            stack.last_mut().expect("parent").children.push(CstElement::Node(node));
            continue;
        }
        stack.last_mut().expect("frame").children.push(CstElement::Token(token));
    }
    if stack.len() != 1 {
        let frame = stack.pop().expect("unclosed frame");
        let (delimiter, offset) = frame.open.expect("nested frame");
        return Err(SyntaxError::UnclosedDelimiter { delimiter, offset });
    }
    let frame = stack.pop().expect("root");
    Ok(CstNode { kind: CstNodeKind::Document, range: TextRange::new(0, source_len), children: frame.children })
}


#[cfg(test)]
mod test;
