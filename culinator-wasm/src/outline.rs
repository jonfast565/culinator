//! Projection of [`culinator_parser::Outline`] onto the shape the desktop
//! recipe builder consumes (`Outline` in
//! `culinator-desktop/src/features/recipe-builder/outline.ts`).
//!
//! Field names are camelCase to match the TypeScript interface exactly, and
//! `Option::None` is skipped rather than serialized as `null`, so optional TS
//! fields stay optional — the same conventions as `ui_model.rs`.

use crate::offsets::Utf16Offsets;
use culinator_parser::{Outline, OutlineForm, OutlineNode, TextRange};
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiRange {
    pub start: usize,
    pub end: usize,
}

impl UiRange {
    /// Offsets are converted to UTF-16 so `String.prototype.slice` on the JS
    /// side lands where the parser meant. See `offsets.rs`.
    fn new(range: TextRange, offsets: &Utf16Offsets) -> Self {
        Self {
            start: offsets.at(range.start),
            end: offsets.at(range.end),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiOutlineNode {
    pub keyword: String,
    /// `"declaration"` when the node carries a block, otherwise `"statement"`.
    pub form: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Includes leading trivia: the span to delete.
    pub range: UiRange,
    /// Excludes leading trivia: the span to replace.
    pub code_range: UiRange,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_range: Option<UiRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_range: Option<UiRange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_inner_range: Option<UiRange>,
    pub indent: String,
    pub children: Vec<UiOutlineNode>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiOutline {
    pub nodes: Vec<UiOutlineNode>,
    pub source_len: usize,
    /// False when the source had no walkable tree — an unbalanced brace or an
    /// unterminated string. The builder keeps its last good outline and
    /// disables structural editing rather than reading this as "no
    /// declarations".
    pub parsed: bool,
}

fn node(node: &OutlineNode, offsets: &Utf16Offsets) -> UiOutlineNode {
    UiOutlineNode {
        keyword: node.keyword.clone(),
        form: match node.form {
            OutlineForm::Declaration => "declaration",
            OutlineForm::Statement => "statement",
        },
        symbol: node.symbol.clone(),
        range: UiRange::new(node.range, offsets),
        code_range: UiRange::new(node.code_range, offsets),
        value_range: node.value_range.map(|range| UiRange::new(range, offsets)),
        header_range: node.header_range.map(|range| UiRange::new(range, offsets)),
        block_inner_range: node
            .block_inner_range
            .map(|range| UiRange::new(range, offsets)),
        indent: node.indent.clone(),
        children: node
            .children
            .iter()
            .map(|child| self::node(child, offsets))
            .collect(),
    }
}

pub fn project(source: &str) -> UiOutline {
    let offsets = Utf16Offsets::new(source);
    // `sourceLen` is in the same UTF-16 units as the ranges, so it can be
    // compared against a JavaScript `string.length`.
    let source_len = offsets.at(source.len());
    match Outline::parse(source) {
        Ok(outline) => UiOutline {
            nodes: outline
                .nodes
                .iter()
                .map(|child| node(child, &offsets))
                .collect(),
            source_len,
            parsed: true,
        },
        Err(_) => UiOutline {
            nodes: Vec::new(),
            source_len,
            parsed: false,
        },
    }
}
