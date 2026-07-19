//! Projection of the shared prose generator (`culinator-narrative`) onto the
//! shape the reading page renders.
//!
//! The desktop app used to derive this itself in `narrative.ts`, which meant
//! every sentence heuristic existed twice and drifted. Now the exporter and the
//! reading page render from the same generator, so a step reads identically in
//! the app, in the EPUB, and in the printed page.

use culinator_core::{Recipe, ResourceKind, UnitSystem};
use culinator_narrative as narrative;
use culinator_narrative::NumberStyle;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiIngredientLine {
    pub symbol: String,
    /// Amount as displayed; empty when the amount is the cook's call.
    pub quantity: String,
    /// Size, state, and name: "large Hass avocados".
    pub description: String,
    /// Trailing modifiers, for a UI that renders them apart from the name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aside: Option<String>,
}

impl From<&narrative::IngredientLine> for UiIngredientLine {
    fn from(line: &narrative::IngredientLine) -> Self {
        Self {
            symbol: line.symbol.clone(),
            quantity: line.quantity.clone(),
            description: line.description.clone(),
            aside: line.aside.clone(),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiIngredientGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub items: Vec<UiIngredientLine>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiMise {
    pub ingredients: Vec<UiIngredientLine>,
    pub equipment: Vec<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiStep {
    /// Operation symbol, so the editor can act on the right declaration.
    pub symbol: String,
    pub number: usize,
    /// Full instruction prose with equipment and doneness woven in.
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    /// "hands-on · makes roux", already joined.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<String>,
    pub tools: Vec<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiSection {
    pub process: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Parallelism guidance ("You can work on this while Prep is under way.").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub steps: Vec<UiStep>,
    /// What this section alone needs on hand, for the colocated layout.
    pub mise: UiMise,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiNarrative {
    /// "10 ingredients · 9 steps · ~2 h 17 min total"
    pub summary: String,
    pub ingredient_groups: Vec<UiIngredientGroup>,
    /// Whole-recipe equipment list for the traditional top-matter layout.
    pub equipment: Vec<String>,
    pub sections: Vec<UiSection>,
}

fn mise(recipe: &Recipe, process: &str, style: NumberStyle) -> UiMise {
    let derived = narrative::section_mise(recipe, process, style);
    UiMise {
        ingredients: derived
            .ingredients
            .iter()
            .map(UiIngredientLine::from)
            .collect(),
        equipment: derived.equipment,
    }
}

/// Build the reading-page narrative. `unit_system` restates every quantity in
/// metric or US customary; `None` keeps the amounts exactly as authored.
pub fn build(recipe: &Recipe, unit_system: Option<UnitSystem>, style: NumberStyle) -> UiNarrative {
    // Convert once, up front, so prose and mise agree on every amount.
    let converted;
    let recipe = match unit_system {
        Some(system) => {
            converted = narrative::convert_recipe_units(recipe, system);
            &converted
        }
        None => recipe,
    };
    let content = narrative::extract_with(recipe, style);

    // Ingredient groups come back as flat strings; re-derive the structured
    // lines so the UI can column-align amounts.
    let lines: Vec<narrative::IngredientLine> = recipe
        .resources
        .iter()
        .filter(|resource| resource.kind == ResourceKind::Ingredient)
        .map(|resource| narrative::ingredient_line_with(resource, None, style))
        .collect();
    let ingredient_groups = content
        .ingredient_groups
        .iter()
        .map(|group| UiIngredientGroup {
            label: group.label.clone(),
            items: group
                .items
                .iter()
                .filter_map(|flat| {
                    lines
                        .iter()
                        .find(|line| &line.flat() == flat)
                        .map(UiIngredientLine::from)
                })
                .collect(),
        })
        .collect();

    UiNarrative {
        summary: content.summary.clone(),
        ingredient_groups,
        equipment: content.equipment.clone(),
        sections: content
            .sections
            .iter()
            .map(|section| UiSection {
                process: section.process.clone(),
                title: section.title.clone(),
                note: section.note.clone(),
                steps: section
                    .steps
                    .iter()
                    .map(|step| UiStep {
                        symbol: step.symbol.clone(),
                        number: step.number,
                        text: step.text.clone(),
                        time: step.time.clone(),
                        meta: (!step.meta.is_empty()).then(|| step.meta.join(" · ")),
                        tools: step.tools.clone(),
                    })
                    .collect(),
                mise: mise(recipe, &section.process, style),
            })
            .collect(),
    }
}
