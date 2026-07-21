//! Projection from the domain [`Recipe`] onto the shape the desktop editor
//! consumes (`UiRecipeModel` in `culinator-desktop/src/features/recipe-editor/model.ts`).
//!
//! This exists so the frontend does not need its own parser. Field names are
//! camelCase to match the TypeScript interface exactly; `Option::None` is
//! skipped rather than serialized as `null` so optional TS fields stay optional.

use crate::offsets::Utf16Offsets;
use culinator_core::{
    BindingRole, DonenessCue, HeatLevel, LaborMode, Operation, Quantity, Recipe, Resource,
    ResourceKind, SourceSpan, Value,
};
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiRange {
    pub start: usize,
    pub end: usize,
}

impl UiRange {
    /// Offsets are converted to UTF-16 so `String.prototype.slice` on the JS
    /// side lands where the parser meant. Emitting raw byte offsets used to
    /// corrupt any recipe containing a non-ASCII character — see `offsets.rs`.
    fn new(span: &SourceSpan, offsets: &Utf16Offsets) -> Self {
        Self {
            start: offsets.at(span.start),
            end: offsets.at(span.end),
        }
    }
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiResource {
    pub symbol: String,
    pub name: String,
    pub kind: String,
    pub measurement: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allergen: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub divided: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub substitutes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_taste: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<UiRange>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiProcess {
    pub symbol: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiInputBinding {
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiDonenessCue {
    pub kind: String,
    pub value: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiOperation {
    pub symbol: String,
    pub action: String,
    pub process: String,
    pub duration_minutes: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_max_minutes: Option<f64>,
    pub labor: String,
    pub after: Vec<String>,
    pub inputs: Vec<String>,
    pub input_bindings: Vec<UiInputBinding>,
    pub equipment: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub produces: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_temperature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heat_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doneness: Option<Vec<UiDonenessCue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<UiRange>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiDiagnostic {
    pub message: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UiRecipeModel {
    pub title: String,
    pub symbol: String,
    pub resources: Vec<UiResource>,
    pub processes: Vec<UiProcess>,
    pub operations: Vec<UiOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<String>,
    /// Recovery diagnostics. Empty for well-formed source.
    pub diagnostics: Vec<UiDiagnostic>,
}

/// Render a value the way the editor's regex parser used to: verbatim text for
/// strings/symbols, and `"<value> <unit>"` for quantities.
fn value_text(value: &Value) -> Option<String> {
    match value {
        Value::Text(text) | Value::Symbol(text) => Some(text.clone()),
        Value::Number(number) => Some(format_number(*number)),
        Value::Boolean(flag) => Some(flag.to_string()),
        Value::Quantity(quantity) => Some(quantity_text(quantity)),
        Value::Range { min, max } => Some(format!(
            "{} to {}",
            value_text(min).unwrap_or_default(),
            value_text(max).unwrap_or_default()
        )),
        Value::List(_) | Value::Object(_) => None,
    }
}

/// Trim the trailing `.0` that `f64::to_string` leaves on whole numbers, so a
/// quantity reads "500 g" rather than "500.0 g".
fn format_number(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        return format!("{}", value as i64);
    }
    value.to_string()
}

fn quantity_text(quantity: &Quantity) -> String {
    format!("{} {}", format_number(quantity.value), quantity.unit)
}

fn kind_text(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Ingredient => "ingredient",
        ResourceKind::Material => "material",
        ResourceKind::Intermediate => "material",
        ResourceKind::Container => "container",
        ResourceKind::Equipment => "equipment",
        ResourceKind::Environment => "environment",
        ResourceKind::Labor => "labor",
    }
}

fn labor_text(labor: Option<LaborMode>) -> String {
    match labor {
        Some(LaborMode::Active) => "active",
        Some(LaborMode::Passive) => "passive",
        Some(LaborMode::Monitor) => "monitor",
        Some(LaborMode::Automated) => "automated",
        None => "unspecified",
    }
    .to_owned()
}

fn heat_text(level: HeatLevel) -> &'static str {
    match level {
        HeatLevel::Low => "low",
        HeatLevel::MediumLow => "medium_low",
        HeatLevel::Medium => "medium",
        HeatLevel::MediumHigh => "medium_high",
        HeatLevel::High => "high",
    }
}

fn doneness_kind_text(cue: &DonenessCue) -> String {
    format!("{:?}", cue.kind)
        .chars()
        .fold(String::new(), |mut out, ch| {
            if ch.is_uppercase() && !out.is_empty() {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
            out
        })
}

fn resource(resource: &Resource, offsets: &Utf16Offsets) -> UiResource {
    let properties = &resource.properties;
    UiResource {
        symbol: resource.symbol.clone(),
        name: properties
            .get("name")
            .and_then(value_text)
            .unwrap_or_else(|| resource.symbol.replace('_', " ")),
        kind: kind_text(resource.kind).to_owned(),
        // `measured by <dim>` is parsed onto the declared type as a type
        // argument (`ingredient salt measured by volume` -> `Ingredient<Volume>`,
        // see `semantic.rs`), so read it back from there. This used to guess the
        // dimension from the declared quantity instead, which reported
        // "unspecified" for a divided ingredient — those carry `measured by`
        // but no `quantity`, because the amounts live on the step bindings.
        measurement: resource
            .declared_type
            .arguments
            .first()
            .map(|argument| argument.name.to_lowercase())
            .or_else(|| {
                properties.get("quantity").and_then(|value| match value {
                    Value::Quantity(quantity) => {
                        Some(format!("{:?}", quantity.dimension).to_lowercase())
                    }
                    _ => None,
                })
            })
            .unwrap_or_else(|| "unspecified".to_owned()),
        quantity: properties.get("quantity").and_then(value_text),
        state: properties.get("state").and_then(value_text),
        allergen: properties.get("allergen").and_then(value_text),
        optional: resource.optional.then_some(true),
        divided: resource.divided.then_some(true),
        substitutes: (!resource.substitutes.is_empty()).then(|| {
            resource
                .substitutes
                .iter()
                .filter_map(value_text)
                .collect::<Vec<_>>()
        }),
        to_taste: resource.to_taste.then_some(true),
        size: resource.size.clone(),
        variant: resource.variant.clone(),
        notes: (!resource.notes.is_empty()).then(|| resource.notes.clone()),
        range: resource
            .span
            .as_ref()
            .map(|span| UiRange::new(span, offsets)),
    }
}

fn operation(operation: &Operation, offsets: &Utf16Offsets) -> UiOperation {
    let inputs: Vec<&culinator_core::ResourceBinding> = operation
        .bindings
        .iter()
        .filter(|binding| binding.role == BindingRole::Input)
        .collect();
    UiOperation {
        symbol: operation.symbol.clone(),
        // The editor keys its verb table off the lowercased action name.
        action: operation.declared_type.name.to_lowercase(),
        process: operation.process.clone(),
        duration_minutes: operation.duration_min_seconds.unwrap_or(0) as f64 / 60.0,
        duration_max_minutes: operation
            .duration_max_seconds
            .map(|seconds| seconds as f64 / 60.0),
        labor: labor_text(operation.labor),
        after: operation
            .dependencies
            .iter()
            .map(|dependency| dependency.predecessor.clone())
            .collect(),
        inputs: inputs
            .iter()
            .map(|binding| binding.resource.clone())
            .collect(),
        input_bindings: inputs
            .iter()
            .map(|binding| UiInputBinding {
                symbol: binding.resource.clone(),
                quantity: binding.quantity.as_ref().map(quantity_text),
            })
            .collect(),
        equipment: operation
            .bindings
            .iter()
            .filter(|binding| {
                matches!(
                    binding.role,
                    BindingRole::Tool
                        | BindingRole::Container
                        | BindingRole::Equipment
                        | BindingRole::Target
                )
            })
            .map(|binding| binding.resource.clone())
            .collect(),
        produces: operation
            .bindings
            .iter()
            .find(|binding| binding.role == BindingRole::Output)
            .map(|binding| binding.resource.clone()),
        target_temperature: operation.target_temperature.as_ref().map(quantity_text),
        heat_level: operation
            .heat_level
            .map(|level| heat_text(level).to_owned()),
        doneness: (!operation.doneness.is_empty()).then(|| {
            operation
                .doneness
                .iter()
                .map(|cue| UiDonenessCue {
                    kind: doneness_kind_text(cue),
                    value: value_text(&cue.value).unwrap_or_default(),
                })
                .collect()
        }),
        photo: operation.properties.get("photo").and_then(value_text),
        repeat: operation.repeat,
        notes: (!operation.notes.is_empty()).then(|| operation.notes.clone()),
        range: operation
            .span
            .as_ref()
            .map(|span| UiRange::new(span, offsets)),
    }
}

pub fn project(source: &str, recipe: &Recipe, diagnostics: Vec<UiDiagnostic>) -> UiRecipeModel {
    let offsets = Utf16Offsets::new(source);
    let property = |key: &str| recipe.properties.get(key).and_then(value_text);
    UiRecipeModel {
        title: recipe.title.clone(),
        symbol: recipe.symbol.clone(),
        resources: recipe
            .resources
            .iter()
            .map(|item| resource(item, &offsets))
            .collect(),
        processes: recipe
            .processes
            .iter()
            .map(|process| UiProcess {
                symbol: process.symbol.clone(),
            })
            .collect(),
        operations: recipe
            .operations
            .iter()
            .map(|item| operation(item, &offsets))
            .collect(),
        source: property("source"),
        source_url: property("source_url"),
        attribution: property("attribution"),
        section: property("section"),
        cover_image: property("image"),
        diagnostics,
    }
}

/// The model an unparseable document projects to, so the editor always has
/// something to render.
pub fn empty(diagnostics: Vec<UiDiagnostic>) -> UiRecipeModel {
    UiRecipeModel {
        title: String::new(),
        symbol: String::new(),
        resources: vec![],
        processes: vec![],
        operations: vec![],
        source: None,
        source_url: None,
        attribution: None,
        section: None,
        cover_image: None,
        diagnostics,
    }
}
