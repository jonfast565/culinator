//! WebAssembly bindings that let the desktop app reuse the Rust parser instead
//! of maintaining a second, regex-based one in TypeScript.
//!
//! The main entry point is [`parse_ui_model`], which parses with error
//! recovery and returns JSON shaped exactly like the editor's `UiRecipeModel`.
//! Recovery is what makes this usable for a live editor: a half-typed
//! declaration costs that declaration, not the whole preview.
//!
//! [`parse_outline`] is its structural counterpart: where the UI model says
//! what a recipe *means* (and drops what it cannot model), the outline says
//! where every declaration and statement physically *is*, so a structured
//! editor can rewrite one in place without disturbing its neighbours.

mod narrative;
mod offsets;
mod outline;
mod ui_model;

pub use narrative::UiNarrative;
pub use outline::UiOutline;
pub use ui_model::{UiDiagnostic, UiRecipeModel};

use culinator_core::UnitSystem;
use culinator_narrative::NumberStyle;
use culinator_parser::parse_recipe_recovering;
use wasm_bindgen::prelude::*;

/// Parse `source` into the editor's UI model, recovering from syntax errors.
/// Returns a JSON string; parsing never fails, it degrades.
#[wasm_bindgen]
pub fn parse_ui_model(source: &str) -> String {
    serde_json::to_string(&parse_ui_model_native(source))
        .unwrap_or_else(|error| format!(r#"{{"error":"{error}"}}"#))
}

/// The same projection, without the JS boundary — used by tests and by any
/// native consumer that wants the editor's view of a recipe.
pub fn parse_ui_model_native(source: &str) -> UiRecipeModel {
    let outcome = parse_recipe_recovering(source);
    // Diagnostic spans are converted alongside declaration spans: the editor
    // underlines and scrolls to them with JavaScript string indices.
    let offsets = offsets::Utf16Offsets::new(source);
    let diagnostics: Vec<UiDiagnostic> = outcome
        .diagnostics
        .iter()
        .map(|diagnostic| UiDiagnostic {
            message: diagnostic.message.clone(),
            start: offsets.at(diagnostic.span.start),
            end: offsets.at(diagnostic.span.end),
        })
        .collect();
    match outcome.value {
        Some(recipe) => ui_model::project(source, &recipe, diagnostics),
        None => ui_model::empty(diagnostics),
    }
}

/// Map every declaration and statement in `source` to its byte range, so a
/// structured editor can splice one statement without regenerating the block
/// around it. Returns JSON; unwalkable source yields `parsed: false` rather
/// than an error.
#[wasm_bindgen]
pub fn parse_outline(source: &str) -> String {
    serde_json::to_string(&parse_outline_native(source))
        .unwrap_or_else(|error| format!(r#"{{"error":"{error}"}}"#))
}

/// The same projection, without the JS boundary.
pub fn parse_outline_native(source: &str) -> UiOutline {
    outline::project(source)
}

/// Build the reading-page narrative: ingredient groups, method sections with
/// rendered step prose, times, and per-section mise en place.
///
/// `unit_system` is `"metric"` or `"us_customary"`; anything else keeps the
/// amounts as authored. `number_style` is `"decimals"` or `"fractions"`. Returns JSON; never fails, it degrades to an empty
/// narrative for unparseable source.
#[wasm_bindgen]
pub fn narrative(source: &str, unit_system: &str, number_style: &str) -> String {
    serde_json::to_string(&narrative_native(source, unit_system, number_style))
        .unwrap_or_else(|error| format!(r#"{{"error":"{error}"}}"#))
}

pub fn narrative_native(
    source: &str,
    unit_system: &str,
    number_style: &str,
) -> narrative::UiNarrative {
    let system = match unit_system {
        "metric" => Some(UnitSystem::Metric),
        "us_customary" => Some(UnitSystem::UsCustomary),
        _ => None,
    };
    let style = match number_style {
        "decimals" => NumberStyle::Decimals,
        _ => NumberStyle::Fractions,
    };
    match parse_recipe_recovering(source).value {
        Some(recipe) => narrative::build(&recipe, system, style),
        None => narrative::build(&empty_recipe(), system, style),
    }
}

/// A recipe with nothing in it, so an unparseable document still renders an
/// (empty) page rather than throwing.
fn empty_recipe() -> culinator_core::Recipe {
    culinator_core::Recipe {
        id: uuid::Uuid::nil(),
        book_id: None,
        symbol: String::new(),
        declared_type: culinator_core::TypeRef::named("Recipe"),
        title: String::new(),
        protocol_version: String::new(),
        types: vec![],
        resources: vec![],
        processes: vec![],
        operations: vec![],
        servings: vec![],
        formulas: vec![],
        yields: vec![],
        properties: Default::default(),
    }
}

#[cfg(test)]
mod test;
