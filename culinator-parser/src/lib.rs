//! Parser facade for Culinator.
//!
//! The parser has two deliberately separate layers:
//! - [`syntax`] tokenizes every byte and builds a lossless concrete syntax tree.
//! - [`semantic`] projects supported syntax into the domain AST.
//!
//! Consumers that edit source should retain a [`LosslessDocument`] and apply
//! [`TextEdit`] values. Re-running semantic projection never rewrites trivia or
//! unsupported declarations.

mod semantic;
pub mod syntax;

pub use semantic::{Diagnostic, ParseError, ParseOutcome};
pub use syntax::{
    CstElement, CstNode, CstNodeKind, LosslessDocument, SyntaxError, SyntaxKind, SyntaxToken,
    TextEdit, TextRange, apply_text_edits,
};

use culinator_core::{Document, Recipe, RecipeBook};

/// Parse a document semantically while using the lossless lexer for source
/// validation and span accounting.
pub fn parse_document(source: &str) -> Result<Document, ParseError> {
    // Always build the lossless representation first. This guarantees byte
    // coverage and catches malformed strings/comments with source offsets.
    LosslessDocument::parse(source).map_err(ParseError::from)?;
    semantic::parse_semantic_document(source)
}

pub fn parse_recipe(source: &str) -> Result<Recipe, ParseError> {
    LosslessDocument::parse(source).map_err(ParseError::from)?;
    semantic::parse_semantic_recipe(source)
}

/// Parse a recipe, recovering from syntax errors instead of stopping at the
/// first one. Returns whatever could be projected plus every problem found.
///
/// This is the editor path: while a declaration is half-typed you still get the
/// rest of the recipe, so live previews and outlines do not blank out. The
/// strict [`parse_recipe`] remains the source of truth for validation,
/// scheduling, and export — it rejects anything that produces a diagnostic.
///
/// Note this deliberately skips the lossless pre-pass, whose whole job is to
/// reject malformed input.
pub fn parse_recipe_recovering(source: &str) -> ParseOutcome<Recipe> {
    let outcome = semantic::parse_semantic_document_recovering(source);
    let value = match outcome.value {
        Some(Document::Recipe { recipe }) => Some(recipe),
        // A book document still yields its first recipe, which is what an
        // editor previewing one recipe out of a book wants.
        Some(Document::RecipeBook { book }) => book.recipes.into_iter().next(),
        None => None,
    };
    ParseOutcome {
        value,
        diagnostics: outcome.diagnostics,
    }
}

pub fn parse_recipe_book(source: &str) -> Result<RecipeBook, ParseError> {
    LosslessDocument::parse(source).map_err(ParseError::from)?;
    semantic::parse_semantic_recipe_book(source)
}

/// Parse both exact syntax and the supported semantic model.
pub fn parse_lossless(source: &str) -> Result<ParsedDocument, ParseError> {
    let syntax = LosslessDocument::parse(source).map_err(ParseError::from)?;
    let semantic = semantic::parse_semantic_document(source)?;
    Ok(ParsedDocument { syntax, semantic })
}

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub syntax: LosslessDocument,
    pub semantic: Document,
}

impl ParsedDocument {
    /// Applies non-overlapping edits to the original source and reparses both
    /// layers. Untouched bytes remain byte-for-byte identical.
    pub fn edit(&self, edits: &[TextEdit]) -> Result<Self, ParseError> {
        let source = apply_text_edits(self.syntax.source(), edits).map_err(ParseError::from)?;
        parse_lossless(&source)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CulinatorParser;

impl culinator_models::DocumentParser for CulinatorParser {
    fn parse_document(&self, source: &str) -> Result<Document, culinator_models::ApplicationError> {
        parse_document(source)
            .map_err(|error| culinator_models::ApplicationError::Parse(error.to_string()))
    }
}

#[cfg(test)]
mod test;
