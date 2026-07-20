//! A syntactic map of every declaration and statement in a document, with the
//! byte ranges needed to rewrite one of them in place.
//!
//! This exists so a structured editor can change `quantity 8 oz;` by splicing
//! those sixteen bytes rather than regenerating the declaration around them.
//! Regenerating loses whatever the editor's model does not know about — the
//! `allergen milk;` in `baked_macaroni_and_cheese.cg`, the `//` comments that
//! carry the authoring rationale, the property ordering someone chose. Here
//! those are not preserved so much as never touched.
//!
//! The outline is deliberately **purely syntactic**: it groups tokens into
//! statements and blocks and reads the leading identifiers, but it does not
//! know what an `ingredient` is. New syntax therefore costs it nothing, and it
//! stays correct for constructs the semantic model drops entirely (yields,
//! servings, formulas) or never modelled (`allergen`).
//!
//! ## Invariants
//!
//! Within any scope — the document, or the inside of a block — nodes are
//! contiguous and non-overlapping, the first starts at the scope start, and
//! each node's [`OutlineNode::range`] ends exactly where the next begins. Any
//! bytes after the last node are trailing trivia. So the scope's text is
//! `concat(node.range) + trailing`, which is [`LosslessDocument::round_trip`]'s
//! discipline applied per scope, and it is what guarantees no byte is
//! silently unaccounted for. `syntax/outline/test.rs` asserts this over every
//! seed recipe.
//!
//! Leading trivia attaches to the node that *follows* it, so deleting a
//! declaration takes its explanatory comment with it and an insertion lands on
//! the far side of the blank line rather than between a comment and the thing
//! it describes.

use super::{
    CstElement, CstNode, CstNodeKind, LosslessDocument, SyntaxError, SyntaxKind, SyntaxToken,
    TextRange,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutlineForm {
    /// Carries a block: `ingredient flour measured by mass { … }`.
    Declaration,
    /// Ends at a semicolon: `title "Pizza Dough";`.
    Statement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineNode {
    /// The leading identifier — `ingredient`, `operation`, `title`, `allergen`.
    /// Empty when the node does not start with one.
    pub keyword: String,
    pub form: OutlineForm,
    /// The second token when it is an identifier: the declared symbol on a
    /// declaration (`flour`), the referenced one on a binding (`after knead;`).
    pub symbol: Option<String>,
    /// The whole node **including leading trivia**. This is the span to delete.
    pub range: TextRange,
    /// The node without leading trivia. This is the span to replace: leaving
    /// the trivia outside means indentation and preceding comments survive.
    pub code_range: TextRange,
    /// Everything after the keyword, up to (not including) the `;`. The text a
    /// caller displays for a property it does not model.
    pub value_range: Option<TextRange>,
    /// Declarations only: the keyword through to just before the `{`.
    pub header_range: Option<TextRange>,
    /// Declarations only: between the braces. Insert new members at its end.
    pub block_inner_range: Option<TextRange>,
    /// Whitespace preceding the node on its own line, for matching insertions
    /// to the file's existing style rather than assuming four spaces.
    pub indent: String,
    pub children: Vec<OutlineNode>,
}

impl OutlineNode {
    /// Direct child statement/declaration with this keyword, if any.
    pub fn child(&self, keyword: &str) -> Option<&OutlineNode> {
        self.children.iter().find(|node| node.keyword == keyword)
    }
    /// Every direct child with this keyword — `note` and `input` repeat.
    pub fn children_named<'a>(
        &'a self,
        keyword: &'a str,
    ) -> impl Iterator<Item = &'a OutlineNode> + 'a {
        self.children
            .iter()
            .filter(move |node| node.keyword == keyword)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outline {
    /// Document-level nodes: the `culinator 0.3;` header and the `recipe` (or
    /// `book`) declaration.
    pub nodes: Vec<OutlineNode>,
    pub source_len: usize,
}

impl Outline {
    pub fn parse(source: &str) -> Result<Self, SyntaxError> {
        Ok(Self::from_document(&LosslessDocument::parse(source)?))
    }

    /// Parse, or yield an empty outline when the source has no tree to walk.
    ///
    /// Unlike the semantic parser, which recovers per declaration, the lossless
    /// tree hard-fails on an unbalanced delimiter or an unterminated string —
    /// there is no sensible partial answer. Callers should keep showing their
    /// last good outline and disable structural editing rather than treat the
    /// empty result as "this document has no declarations".
    pub fn parse_recovering(source: &str) -> Self {
        Self::parse(source).unwrap_or(Self {
            nodes: Vec::new(),
            source_len: source.len(),
        })
    }

    fn from_document(document: &LosslessDocument) -> Self {
        let source = document.source();
        Self {
            nodes: scope_nodes(
                source,
                &document.root().children,
                TextRange::new(0, source.len()),
            ),
            source_len: source.len(),
        }
    }

    /// The `recipe` declaration, which owns everything a recipe editor touches.
    pub fn recipe(&self) -> Option<&OutlineNode> {
        self.nodes.iter().find(|node| node.keyword == "recipe")
    }
}

fn as_token(element: &CstElement) -> Option<&SyntaxToken> {
    match element {
        CstElement::Token(token) => Some(token),
        CstElement::Node(_) => None,
    }
}

/// Indentation on the line where `offset` sits, or empty when something other
/// than whitespace precedes it (an inline `{ quantity 1 tbsp; }`).
fn indent_at(source: &str, offset: usize) -> String {
    let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    let prefix = &source[line_start..offset];
    if !prefix.is_empty() && prefix.bytes().all(|byte| byte == b' ' || byte == b'\t') {
        prefix.to_owned()
    } else {
        String::new()
    }
}

/// Split one scope's elements into nodes. `scope` bounds the region so the
/// first node's leading trivia starts at the scope start rather than at the
/// first token.
fn scope_nodes(source: &str, elements: &[CstElement], scope: TextRange) -> Vec<OutlineNode> {
    let mut nodes = Vec::new();
    // Start of the node being accumulated, including its leading trivia.
    let mut node_start = scope.start;
    let mut code_start: Option<usize> = None;
    let mut code_end = scope.start;
    let mut keyword = String::new();
    let mut symbol: Option<String> = None;
    let mut value_start: Option<usize> = None;
    let mut seen = 0usize;

    for element in elements {
        let (range, token) = match element {
            CstElement::Token(token) if token.kind.is_trivia() => continue,
            CstElement::Token(token) => (token.range, Some(token)),
            CstElement::Node(node) => (node.range, None),
        };
        let is_block = matches!(element, CstElement::Node(node) if node.kind == CstNodeKind::Block);
        let is_semicolon = token.is_some_and(|token| token.kind == SyntaxKind::Semicolon);

        if code_start.is_none() {
            code_start = Some(range.start);
        }
        seen += 1;
        match seen {
            1 => {
                if let Some(token) = token
                    && token.kind == SyntaxKind::Identifier
                {
                    keyword = token.text.clone();
                }
            }
            // The value begins at the second element — unless that is already
            // the terminator, as in a bare `optional;`.
            2 if !is_semicolon => {
                value_start = Some(range.start);
                if let Some(token) = token
                    && token.kind == SyntaxKind::Identifier
                {
                    symbol = Some(token.text.clone());
                }
            }
            _ => {}
        }
        code_end = range.end;

        if !is_block && !is_semicolon {
            continue;
        }

        let start = code_start.expect("a node was started");
        let (form, header_range, block_inner_range, children, value_end) = if is_block {
            let block = match element {
                CstElement::Node(node) => node,
                CstElement::Token(_) => unreachable!("is_block implies a node"),
            };
            let (inner, inner_elements) = block_interior(block);
            (
                OutlineForm::Declaration,
                Some(TextRange::new(start, block.range.start)),
                Some(inner),
                scope_nodes(source, inner_elements, inner),
                block.range.start,
            )
        } else {
            (OutlineForm::Statement, None, None, Vec::new(), range.start)
        };

        nodes.push(OutlineNode {
            keyword: std::mem::take(&mut keyword),
            form,
            symbol: symbol.take(),
            range: TextRange::new(node_start, code_end),
            code_range: TextRange::new(start, code_end),
            value_range: value_start
                .take()
                .filter(|&begin| begin < value_end)
                .map(|begin| TextRange::new(begin, value_end)),
            header_range,
            block_inner_range,
            indent: indent_at(source, start),
            children,
        });

        node_start = code_end;
        code_start = None;
        seen = 0;
    }

    // A run of tokens with no terminator — half-typed input, or trailing junk
    // before a `}`. Emit it so its bytes stay accounted for.
    if let Some(start) = code_start {
        nodes.push(OutlineNode {
            keyword,
            form: OutlineForm::Statement,
            symbol,
            range: TextRange::new(node_start, code_end),
            code_range: TextRange::new(start, code_end),
            value_range: value_start
                .filter(|&begin| begin < code_end)
                .map(|begin| TextRange::new(begin, code_end)),
            header_range: None,
            block_inner_range: None,
            indent: indent_at(source, start),
            children: Vec::new(),
        });
    }
    nodes
}

/// The span between a block's braces, and the elements inside them.
fn block_interior(block: &CstNode) -> (TextRange, &[CstElement]) {
    let open = block.children.first().and_then(as_token);
    let close = block.children.last().and_then(as_token);
    match (open, close) {
        (Some(open), Some(close)) if block.children.len() >= 2 => (
            TextRange::new(open.range.end, close.range.start),
            &block.children[1..block.children.len() - 1],
        ),
        // `build_tree` always brackets a block with its delimiters, so this is
        // unreachable; treat the whole node as interior rather than panicking.
        _ => (block.range, block.children.as_slice()),
    }
}

#[cfg(test)]
mod test;
