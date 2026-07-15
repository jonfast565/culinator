mod edit;
mod lexer;
mod tree;

pub use edit::{TextEdit, apply_text_edits};
pub use lexer::{SyntaxError, SyntaxKind, SyntaxToken, TextRange, lex_lossless};
pub use tree::{CstElement, CstNode, CstNodeKind, LosslessDocument};

#[cfg(test)]
mod test;
