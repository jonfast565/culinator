mod edit;
mod lexer;
mod outline;
mod tree;

pub use edit::{TextEdit, apply_text_edits};
pub use lexer::{SyntaxError, SyntaxKind, SyntaxToken, TextRange, lex_lossless};
pub use outline::{Outline, OutlineForm, OutlineNode};
pub use tree::{CstElement, CstNode, CstNodeKind, LosslessDocument};

#[cfg(test)]
mod test;
