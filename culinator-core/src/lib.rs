use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

mod units;
pub use units::*;

mod formula_scale;
pub use formula_scale::*;

pub mod order;

pub type Symbol = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeRef {
    pub name: Symbol,
    pub arguments: Vec<TypeRef>,
}

impl TypeRef {
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arguments: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dimension {
    Mass,
    Volume,
    Count,
    Time,
    Temperature,
    Length,
    Area,
    Energy,
    Ratio,
    Concentration,
    Boolean,
    Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    pub value: f64,
    pub unit: Symbol,
    pub dimension: Dimension,
}

impl Dimension {
    /// Classify a cooking unit string into its physical dimension. This is the
    /// single source of truth for unit → dimension; parsers and consumers should
    /// delegate here rather than keep their own tables. Matching is
    /// case-insensitive. Ambiguous single letters (`T`/`t` for tablespoon vs.
    /// teaspoon) are deliberately excluded — use the spelled-out abbreviations.
    pub fn from_unit(unit: &str) -> Dimension {
        match unit
            .trim()
            .trim_end_matches('.')
            .to_ascii_lowercase()
            .as_str()
        {
            // --- Mass / weight ---
            "g" | "gram" | "grams" | "gm" | "gms" => Dimension::Mass,
            "kg" | "kilogram" | "kilograms" | "kilo" | "kilos" => Dimension::Mass,
            "mg" | "milligram" | "milligrams" => Dimension::Mass,
            "oz" | "ounce" | "ounces" => Dimension::Mass,
            "lb" | "lbs" | "pound" | "pounds" => Dimension::Mass,
            "dram" | "drams" => Dimension::Mass,

            // --- Volume ---
            "ml" | "milliliter" | "milliliters" | "millilitre" | "millilitres" | "cc" => {
                Dimension::Volume
            }
            "cl" | "centiliter" | "centiliters" => Dimension::Volume,
            "dl" | "deciliter" | "deciliters" => Dimension::Volume,
            "l" | "liter" | "liters" | "litre" | "litres" => Dimension::Volume,
            "cup" | "cups" => Dimension::Volume,
            "tbsp" | "tbsps" | "tbs" | "tablespoon" | "tablespoons" => Dimension::Volume,
            "tsp" | "tsps" | "teaspoon" | "teaspoons" => Dimension::Volume,
            "floz" | "fl_oz" | "fluid_ounce" | "fluid_ounces" => Dimension::Volume,
            "pt" | "pint" | "pints" => Dimension::Volume,
            "qt" | "quart" | "quarts" => Dimension::Volume,
            "gal" | "gallon" | "gallons" => Dimension::Volume,
            "dash" | "dashes" | "pinch" | "pinches" | "smidgen" | "smidgens" | "drop" | "drops" => {
                Dimension::Volume
            }

            // --- Temperature ---
            "c" | "celsius" | "centigrade" => Dimension::Temperature,
            "f" | "fahrenheit" => Dimension::Temperature,
            "k" | "kelvin" => Dimension::Temperature,

            // --- Time ---
            "s" | "sec" | "secs" | "second" | "seconds" => Dimension::Time,
            "min" | "mins" | "minute" | "minutes" => Dimension::Time,
            "h" | "hr" | "hrs" | "hour" | "hours" => Dimension::Time,
            "day" | "days" => Dimension::Time,
            "wk" | "week" | "weeks" => Dimension::Time,

            // --- Length ---
            "mm" | "millimeter" | "millimeters" => Dimension::Length,
            "cm" | "centimeter" | "centimeters" => Dimension::Length,
            "m" | "meter" | "meters" => Dimension::Length,
            "in" | "inch" | "inches" => Dimension::Length,
            "ft" | "foot" | "feet" => Dimension::Length,

            // --- Count / discrete cooking units ---
            "each" | "ea" | "count" | "ct" | "dozen" | "dozens" => Dimension::Count,
            "clove" | "cloves" | "stick" | "sticks" | "piece" | "pieces" | "pc" | "pcs" => {
                Dimension::Count
            }
            "slice" | "slices" | "can" | "cans" | "jar" | "jars" => Dimension::Count,
            "package" | "packages" | "pkg" | "pkgs" | "pack" | "packs" => Dimension::Count,
            "bunch" | "bunches" | "head" | "heads" | "bulb" | "bulbs" => Dimension::Count,
            "sprig" | "sprigs" | "stalk" | "stalks" | "stem" | "stems" => Dimension::Count,
            "leaf" | "leaves" | "ear" | "ears" | "rib" | "ribs" => Dimension::Count,
            "fillet" | "fillets" | "filet" | "filets" | "strip" | "strips" => Dimension::Count,
            "wedge" | "wedges" | "cube" | "cubes" | "ball" | "balls" => Dimension::Count,
            "loaf" | "loaves" | "sheet" | "sheets" | "scoop" | "scoops" => Dimension::Count,
            "knob" | "knobs" | "handful" | "handfuls" | "sprinkle" | "sprinkles" => {
                Dimension::Count
            }

            _ => Dimension::Ratio,
        }
    }

    /// True when `from_unit` would classify `unit` as this dimension.
    pub fn matches_unit(self, unit: &str) -> bool {
        Dimension::from_unit(unit) == self
    }
}

impl Quantity {
    /// Convert this quantity to grams, if it is a mass quantity with a known
    /// unit. Volume/count units return `None` (no density is assumed).
    pub fn as_grams(&self) -> Option<f64> {
        units::quantity_as_grams(self)
    }
}

/// Number of seconds in one of the given time unit, if recognized. Shared by
/// duration parsing so long forms ("minutes", "hours") behave like the short
/// ones. Returns `None` for non-time units.
pub fn time_unit_seconds(unit: &str) -> Option<f64> {
    Some(match unit.trim().to_ascii_lowercase().as_str() {
        "s" | "sec" | "secs" | "second" | "seconds" => 1.0,
        "min" | "mins" | "minute" | "minutes" => 60.0,
        "h" | "hr" | "hrs" | "hour" | "hours" => 3600.0,
        "day" | "days" => 86_400.0,
        "wk" | "week" | "weeks" => 604_800.0,
        _ => return None,
    })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value")]
pub enum Value {
    Text(String),
    Number(f64),
    Quantity(Quantity),
    Boolean(bool),
    Symbol(Symbol),
    List(Vec<Value>),
    Object(BTreeMap<Symbol, Value>),
    /// An inclusive range, e.g. `2 to 3 clove` or `15 min to 20 min`. Both
    /// bounds are normally `Number` or `Quantity` values of the same dimension.
    Range {
        min: Box<Value>,
        max: Box<Value>,
    },
}

/// Wire form that accepts both the tagged `Value` shape and plain JSON
/// scalars. The desktop formula editor sends `properties: { role: "salt",
/// sourceQuantity: "400 g" }`; without this, serde rejects those strings.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ValueWire {
    Tagged(TaggedValue),
    String(String),
    Bool(bool),
    Number(f64),
    List(Vec<Value>),
    Map(BTreeMap<Symbol, Value>),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", content = "value")]
enum TaggedValue {
    Text(String),
    Number(f64),
    Quantity(Quantity),
    Boolean(bool),
    Symbol(Symbol),
    List(Vec<Value>),
    Object(BTreeMap<Symbol, Value>),
    Range {
        min: Box<Value>,
        max: Box<Value>,
    },
}

impl From<TaggedValue> for Value {
    fn from(value: TaggedValue) -> Self {
        match value {
            TaggedValue::Text(text) => Self::Text(text),
            TaggedValue::Number(number) => Self::Number(number),
            TaggedValue::Quantity(quantity) => Self::Quantity(quantity),
            TaggedValue::Boolean(flag) => Self::Boolean(flag),
            TaggedValue::Symbol(symbol) => Self::Symbol(symbol),
            TaggedValue::List(items) => Self::List(items),
            TaggedValue::Object(object) => Self::Object(object),
            TaggedValue::Range { min, max } => Self::Range { min, max },
        }
    }
}

impl From<ValueWire> for Value {
    fn from(value: ValueWire) -> Self {
        match value {
            ValueWire::Tagged(tagged) => tagged.into(),
            ValueWire::String(text) => quantity_from_text(&text)
                .map(Self::Quantity)
                .unwrap_or(Self::Text(text)),
            ValueWire::Bool(flag) => Self::Boolean(flag),
            ValueWire::Number(number) => Self::Number(number),
            ValueWire::List(items) => Self::List(items),
            ValueWire::Map(map) => quantity_from_map(&map)
                .map(Self::Quantity)
                .unwrap_or(Self::Object(map)),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        ValueWire::deserialize(deserializer).map(Self::from)
    }
}

/// Parse a free-standing quantity string such as `"400 g"` or `"1.5 cup"`.
fn quantity_from_text(text: &str) -> Option<Quantity> {
    let trimmed = text.trim();
    let (number, rest) = trimmed.split_once(char::is_whitespace)?;
    let value: f64 = number.parse().ok()?;
    let unit = rest.trim();
    if unit.is_empty() {
        return None;
    }
    let dimension = Dimension::from_unit(unit);
    if matches!(dimension, Dimension::Text | Dimension::Boolean) {
        return None;
    }
    Some(Quantity {
        value,
        unit: unit.to_owned(),
        dimension,
    })
}

fn quantity_from_map(map: &BTreeMap<Symbol, Value>) -> Option<Quantity> {
    let value = match map.get("value")? {
        Value::Number(number) => *number,
        _ => return None,
    };
    let unit = match map.get("unit")? {
        Value::Text(text) | Value::Symbol(text) => text.clone(),
        _ => return None,
    };
    let dimension = match map.get("dimension") {
        Some(Value::Text(text) | Value::Symbol(text)) => dimension_from_name(text)?,
        Some(Value::Number(_)) => return None,
        None => Dimension::from_unit(&unit),
        _ => Dimension::from_unit(&unit),
    };
    Some(Quantity {
        value,
        unit,
        dimension,
    })
}

fn dimension_from_name(name: &str) -> Option<Dimension> {
    Some(match name.trim().to_ascii_lowercase().as_str() {
        "mass" => Dimension::Mass,
        "volume" => Dimension::Volume,
        "count" => Dimension::Count,
        "time" => Dimension::Time,
        "temperature" => Dimension::Temperature,
        "length" => Dimension::Length,
        "area" => Dimension::Area,
        "energy" => Dimension::Energy,
        "ratio" => Dimension::Ratio,
        "concentration" => Dimension::Concentration,
        "boolean" => Dimension::Boolean,
        "text" => Dimension::Text,
        _ => return None,
    })
}

#[cfg(test)]
mod value_wire_test {
    use super::*;

    #[test]
    fn plain_string_quantity_becomes_quantity() {
        let value: Value = serde_json::from_str(r#""400 g""#).expect("parse");
        assert_eq!(
            value,
            Value::Quantity(Quantity {
                value: 400.0,
                unit: "g".into(),
                dimension: Dimension::Mass,
            })
        );
    }

    #[test]
    fn plain_string_role_stays_text() {
        let value: Value = serde_json::from_str(r#""salt""#).expect("parse");
        assert_eq!(value, Value::Text("salt".into()));
    }

    #[test]
    fn plain_number_and_bool() {
        assert_eq!(
            serde_json::from_str::<Value>("2").expect("number"),
            Value::Number(2.0)
        );
        assert_eq!(
            serde_json::from_str::<Value>("true").expect("bool"),
            Value::Boolean(true)
        );
    }

    #[test]
    fn tagged_form_still_round_trips() {
        let original = Value::Text("hello".into());
        let json = serde_json::to_string(&original).expect("serialize");
        let back: Value = serde_json::from_str(&json).expect("parse");
        assert_eq!(back, original);
    }

    #[test]
    fn formula_properties_from_desktop_shape() {
        let json = r#"{
            "id": "00000000-0000-0000-0000-000000000001",
            "recipe_id": null,
            "symbol": "dough",
            "name": "Pizza Dough formula",
            "basis": "reference_percent",
            "ingredients": [{
                "id": "00000000-0000-0000-0000-000000000002",
                "symbol": "flour",
                "name": "wheat flour",
                "stage": "final",
                "basis": "reference_percent",
                "percentage": 100.0,
                "mass_grams": 400.0,
                "is_reference": true,
                "is_flour": true,
                "water_fraction": 0.0,
                "scalable": true,
                "properties": { "sourceQuantity": "400 g" }
            }],
            "properties": { "target": "628 g", "pieces": 2 }
        }"#;
        let formula: Formula = serde_json::from_str(json).expect("formula");
        assert!(matches!(
            formula.ingredients[0].properties.get("sourceQuantity"),
            Some(Value::Quantity(q)) if (q.value - 400.0).abs() < f64::EPSILON
        ));
        assert!(matches!(
            formula.properties.get("target"),
            Some(Value::Quantity(q)) if (q.value - 628.0).abs() < f64::EPSILON
        ));
        assert_eq!(
            formula.properties.get("pieces"),
            Some(&Value::Number(2.0))
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDeclaration {
    pub id: Uuid,
    pub name: Symbol,
    pub base: TypeRef,
    pub states: BTreeMap<Symbol, TypeRef>,
    pub properties: BTreeMap<Symbol, Value>,
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Ingredient,
    Material,
    Container,
    Equipment,
    Environment,
    Labor,
    /// A product created by an operation partway through the recipe (declared
    /// implicitly via an operation's `produces` clause) rather than supplied as
    /// an input. Lets downstream operations reference it without a warning.
    Intermediate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: Uuid,
    pub symbol: Symbol,
    pub declared_type: TypeRef,
    pub kind: ResourceKind,
    /// True when the ingredient is optional (e.g. an optional garnish, or
    /// "plus more for serving").
    #[serde(default)]
    pub optional: bool,
    /// True when a single ingredient is split across multiple steps
    /// ("divided" / "remaining 1½ sticks"). Per-step amounts live on the
    /// operation's `ResourceBinding.quantity`.
    #[serde(default)]
    pub divided: bool,
    /// Acceptable substitutions for this ingredient (symbols or free text).
    #[serde(default)]
    pub substitutes: Vec<Value>,
    /// True when the amount is "to taste" / "plus more to taste": a base
    /// `quantity` may still be declared, but the cook adjusts the rest. Keeps
    /// the open-ended seasoning intent that a fixed quantity would drop.
    #[serde(default)]
    pub to_taste: bool,
    /// Qualitative size grade for count-measured ingredients ("small",
    /// "medium", "large"), e.g. "2 small Roma tomatoes". A bare `count` cannot
    /// carry this; keeping it structured lets scaling math reason about it.
    #[serde(default)]
    pub size: Option<String>,
    /// Variant-group label. Ingredients that share a label form one
    /// mutually-exclusive set of alternatives (e.g. "sweet" vs. "savory"
    /// crepes). Ingredients with no label are always included.
    #[serde(default)]
    pub variant: Option<String>,
    /// Free-text technique / handling notes ("seed before dicing", "measured
    /// after chopping") that would otherwise be lost from a prose line.
    #[serde(default)]
    pub notes: Vec<String>,
    pub properties: BTreeMap<Symbol, Value>,
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LaborMode {
    Active,
    Passive,
    Monitor,
    Automated,
}

/// Stovetop / burner heat level, distinct from a numeric temperature setpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HeatLevel {
    Low,
    MediumLow,
    Medium,
    MediumHigh,
    High,
}

/// The kind of sensory / measured signal a doneness cue is based on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DonenessKind {
    /// Measured internal temperature (e.g. `165 f` in the thickest part).
    InternalTemp,
    /// Visual appearance (e.g. "golden brown", "coats the back of a spoon").
    Visual,
    /// A physical test (e.g. "toothpick comes out clean").
    Tester,
    /// Texture / feel (e.g. "stiff peaks", "fork-tender").
    Texture,
    /// Rise / expansion (e.g. "doubled in size").
    Rise,
}

/// A structured "cook until…" stopping condition on an operation. Unlike a
/// free-text `requires` predicate, the kind is machine-readable so kitchen mode
/// and HACCP tooling can reason about it (e.g. surface the target temperature).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DonenessCue {
    pub kind: DonenessKind,
    pub value: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyKind {
    FinishStart,
    StartStart,
    FinishFinish,
    StartFinish,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub predecessor: Symbol,
    pub kind: DependencyKind,
    pub minimum_lag_seconds: Option<u64>,
    pub maximum_lag_seconds: Option<u64>,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBinding {
    pub resource: Symbol,
    pub role: BindingRole,
    pub quantity: Option<Quantity>,
    pub exclusive: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingRole {
    Input,
    Output,
    Target,
    Tool,
    Container,
    Equipment,
    Environment,
    Labor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Predicate {
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub target_path: String,
    pub operator: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: Uuid,
    pub symbol: Symbol,
    pub declared_type: TypeRef,
    pub process: Symbol,
    pub labor: Option<LaborMode>,
    pub duration_min_seconds: Option<u64>,
    pub duration_max_seconds: Option<u64>,
    /// True when the duration was authored with the `estimated` keyword, i.e. a
    /// soft guideline rather than a hard time. Ranges (`min != max`) are the
    /// other, orthogonal way to express timing uncertainty.
    #[serde(default)]
    pub duration_estimated: bool,
    /// Numeric temperature setpoint for the step (oven/oil/target temperature),
    /// e.g. `350 f`. Distinct from `heat_level`.
    #[serde(default)]
    pub target_temperature: Option<Quantity>,
    /// Stovetop heat level, e.g. `medium_high`. Distinct from a numeric setpoint.
    #[serde(default)]
    pub heat_level: Option<HeatLevel>,
    /// Structured "cook until…" stopping conditions.
    #[serde(default)]
    pub doneness: Vec<DonenessCue>,
    /// True when the step itself is optional (e.g. trussing a chicken).
    #[serde(default)]
    pub optional: bool,
    /// Number of times the step is repeated (batching, e.g. cooking crepes one
    /// at a time). The `duration_*` fields describe a *single* repetition; the
    /// effective wall-clock cost is `duration * repeat`. `None` means one pass.
    #[serde(default)]
    pub repeat: Option<u32>,
    /// Free-text technique notes that carry method detail a structured field
    /// cannot ("press plastic wrap directly on the surface", "do not rinse the
    /// pasta", "leave some larger chunks for texture").
    #[serde(default)]
    pub notes: Vec<String>,
    pub dependencies: Vec<Dependency>,
    pub bindings: Vec<ResourceBinding>,
    pub requirements: Vec<Predicate>,
    pub effects: Vec<Effect>,
    pub properties: BTreeMap<Symbol, Value>,
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub id: Uuid,
    pub symbol: Symbol,
    pub declared_type: TypeRef,
    pub parent: Option<Symbol>,
    pub operations: Vec<Symbol>,
    pub properties: BTreeMap<Symbol, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Serving {
    pub symbol: Symbol,
    pub declared_type: TypeRef,
    pub amount: Value,
    pub mass_grams: Option<f64>,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: Uuid,
    pub book_id: Option<Uuid>,
    pub symbol: Symbol,
    pub declared_type: TypeRef,
    pub title: String,
    pub protocol_version: String,
    pub types: Vec<TypeDeclaration>,
    pub resources: Vec<Resource>,
    pub processes: Vec<Process>,
    pub operations: Vec<Operation>,
    pub servings: Vec<Serving>,
    pub formulas: Vec<Formula>,
    pub yields: Vec<YieldDefinition>,
    pub properties: BTreeMap<Symbol, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeBook {
    pub id: Uuid,
    pub symbol: Symbol,
    pub declared_type: TypeRef,
    pub title: String,
    pub description: Option<String>,
    pub protocol_version: String,
    pub recipes: Vec<Recipe>,
    pub properties: BTreeMap<Symbol, Value>,
}

impl RecipeBook {
    pub fn empty(
        symbol: impl Into<String>,
        title: impl Into<String>,
        protocol_version: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol: symbol.into(),
            declared_type: TypeRef::named("RecipeBook"),
            title: title.into(),
            description: None,
            protocol_version: protocol_version.into(),
            recipes: Vec::new(),
            properties: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Document {
    Recipe { recipe: Recipe },
    RecipeBook { book: RecipeBook },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldDefinition {
    pub symbol: Symbol,
    pub declared_type: TypeRef,
    pub amount: Value,
    pub mass_grams: Option<f64>,
    pub properties: BTreeMap<Symbol, Value>,
}

// --- Formula model ---------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormulaBasis {
    /// Percentage relative to one or more reference ingredients. Baker's
    /// percentage is this mode with flour ingredients marked as references.
    #[serde(alias = "bakers_percent")]
    ReferencePercent,
    PercentOfTotal,
    AbsoluteMass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PercentageView {
    Reference,
    Total,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrefermentKind {
    Poolish,
    Biga,
    Levain,
    Sponge,
    Soaker,
    Tangzhong,
    /// Cooked flour paste; treated like tangzhong for stage emission.
    Scald,
    /// Mature dough carried into a new mix.
    OldDough,
}

impl PrefermentKind {
    pub fn parse(value: &str) -> Result<Self, FormulaError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "poolish" => Ok(Self::Poolish),
            "biga" => Ok(Self::Biga),
            "levain" => Ok(Self::Levain),
            "sponge" => Ok(Self::Sponge),
            "soaker" => Ok(Self::Soaker),
            "tangzhong" => Ok(Self::Tangzhong),
            "scald" => Ok(Self::Scald),
            "old_dough" | "olddough" | "old-dough" => Ok(Self::OldDough),
            _ => Err(FormulaError::InvalidPrefermentKind {
                kind: value.to_owned(),
            }),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Poolish => "poolish",
            Self::Biga => "biga",
            Self::Levain => "levain",
            Self::Sponge => "sponge",
            Self::Soaker => "soaker",
            Self::Tangzhong => "tangzhong",
            Self::Scald => "scald",
            Self::OldDough => "old_dough",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Preferment {
    pub kind: PrefermentKind,
    /// Share of total recipe flour allocated to this stage (reference percent).
    pub flour_pct: f64,
    /// Preferment hydration: water as a percentage of preferment flour.
    pub hydration: f64,
    /// Culture load relative to preferment flour (yeast % or starter %).
    #[serde(default)]
    pub inoculation: f64,
    /// Stage symbol consumed by the multi-stage formula solver.
    #[serde(default = "default_preferment_stage")]
    pub stage: Symbol,
}

fn default_preferment_stage() -> Symbol {
    "preferment".to_owned()
}

impl Preferment {
    /// Emit reference-percent ingredients tagged with this preferment's stage.
    pub fn build_stage(&self) -> Vec<FormulaIngredient> {
        let stage = self.stage.clone();
        let kind = self.kind.label();
        let mut ingredients = Vec::with_capacity(if self.inoculation > 0.0 { 3 } else { 2 });

        let mut flour_props = BTreeMap::new();
        flour_props.insert(
            "preferment_kind".into(),
            Value::Text(self.kind.label().into()),
        );
        flour_props.insert("role".into(), Value::Text("flour".into()));
        ingredients.push(FormulaIngredient {
            id: Uuid::new_v4(),
            symbol: format!("{stage}_flour"),
            name: format!("{kind} flour"),
            stage: stage.clone(),
            basis: FormulaBasis::ReferencePercent,
            percentage: Some(self.flour_pct),
            mass_grams: None,
            is_reference: false,
            is_flour: true,
            water_fraction: 0.0,
            scalable: true,
            properties: flour_props,
        });

        let mut water_props = BTreeMap::new();
        water_props.insert(
            "preferment_kind".into(),
            Value::Text(self.kind.label().into()),
        );
        water_props.insert("role".into(), Value::Text("water".into()));
        ingredients.push(FormulaIngredient {
            id: Uuid::new_v4(),
            symbol: format!("{stage}_water"),
            name: format!("{kind} water"),
            stage: stage.clone(),
            basis: FormulaBasis::ReferencePercent,
            percentage: Some(self.flour_pct * self.hydration / 100.0),
            mass_grams: None,
            is_reference: false,
            is_flour: false,
            water_fraction: 1.0,
            scalable: true,
            properties: water_props,
        });

        if self.inoculation > 0.0
            && !matches!(
                self.kind,
                PrefermentKind::Soaker | PrefermentKind::Scald | PrefermentKind::Tangzhong
            )
        {
            let culture_pct = self.flour_pct * self.inoculation / 100.0;
            let water_fraction = match self.kind {
                PrefermentKind::Levain | PrefermentKind::OldDough => 0.5,
                _ => 0.0,
            };
            let mut culture_props = BTreeMap::new();
            culture_props.insert(
                "preferment_kind".into(),
                Value::Text(self.kind.label().into()),
            );
            culture_props.insert("role".into(), Value::Text("culture".into()));
            ingredients.push(FormulaIngredient {
                id: Uuid::new_v4(),
                symbol: format!("{stage}_culture"),
                name: format!("{kind} culture"),
                stage,
                basis: FormulaBasis::ReferencePercent,
                percentage: Some(culture_pct),
                mass_grams: None,
                is_reference: false,
                is_flour: false,
                water_fraction,
                scalable: true,
                properties: culture_props,
            });
        }

        ingredients
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IngredientRole {
    Salt,
    Fat,
    Sugar,
    Other,
}

fn property_truthy(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Boolean(true)) => true,
        Some(Value::Number(n)) => *n != 0.0,
        _ => false,
    }
}

fn ingredient_role(item: &FormulaIngredient) -> IngredientRole {
    if let Some(Value::Text(role)) = item.properties.get("role") {
        match role.to_ascii_lowercase().as_str() {
            "salt" => return IngredientRole::Salt,
            "fat" => return IngredientRole::Fat,
            "sugar" => return IngredientRole::Sugar,
            _ => {}
        }
    }
    if property_truthy(item.properties.get("is_salt")) {
        return IngredientRole::Salt;
    }
    if property_truthy(item.properties.get("is_fat")) {
        return IngredientRole::Fat;
    }
    if property_truthy(item.properties.get("is_sugar")) {
        return IngredientRole::Sugar;
    }
    IngredientRole::Other
}

fn liquid_equivalent_fraction(item: &FormulaIngredient) -> f64 {
    if let Some(Value::Number(value)) = item.properties.get("liquid_equivalent") {
        return *value;
    }
    if item.water_fraction > 0.0 {
        return item.water_fraction;
    }
    match ingredient_role(item) {
        IngredientRole::Fat => 0.16,
        IngredientRole::Sugar => 0.0,
        _ => 0.0,
    }
}

/// Standard desired dough temperature (DDT) calculation. Without a preferment,
/// `water_temp = 3×DDT − (flour + room + friction)`. With a preferment,
/// multiply DDT by four and subtract the preferment temperature as well.
pub fn desired_dough_temperature(
    desired_dough_temp: f64,
    friction_factor: f64,
    flour_temp: f64,
    room_temp: f64,
    preferment_temp: Option<f64>,
) -> f64 {
    let factor = if preferment_temp.is_some() { 4.0 } else { 3.0 };
    let known = flour_temp + room_temp + friction_factor + preferment_temp.unwrap_or(0.0);
    factor * desired_dough_temp - known
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulaIngredient {
    pub id: Uuid,
    pub symbol: Symbol,
    pub name: String,
    pub stage: Symbol,
    pub basis: FormulaBasis,
    pub percentage: Option<f64>,
    pub mass_grams: Option<f64>,
    /// Reference ingredients collectively represent 100% in reference mode.
    /// Flour is normally a reference ingredient for dough formulas.
    #[serde(default)]
    pub is_reference: bool,
    /// Retained as domain metadata and for bakery-specific metrics.
    #[serde(default)]
    pub is_flour: bool,
    #[serde(default)]
    pub water_fraction: f64,
    #[serde(default = "default_true")]
    pub scalable: bool,
    pub properties: BTreeMap<Symbol, Value>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Formula {
    pub id: Uuid,
    pub recipe_id: Option<Uuid>,
    pub symbol: Symbol,
    pub name: String,
    pub basis: FormulaBasis,
    pub ingredients: Vec<FormulaIngredient>,
    pub properties: BTreeMap<Symbol, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulaLineResult {
    pub ingredient_id: Uuid,
    pub symbol: Symbol,
    pub name: String,
    pub stage: Symbol,
    pub percentage: Option<f64>,
    pub total_percentage: f64,
    pub mass_grams: f64,
    pub is_reference: bool,
    pub is_flour: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulaResult {
    pub target_mass_grams: f64,
    pub reference_mass_grams: f64,
    pub total_flour_grams: f64,
    pub total_mass_grams: f64,
    pub hydration_percent: f64,
    pub prefermented_flour_percent: f64,
    #[serde(default)]
    pub salt_percent: f64,
    #[serde(default)]
    pub fat_percent: f64,
    #[serde(default)]
    pub sugar_percent: f64,
    #[serde(default)]
    pub effective_hydration_percent: f64,
    pub lines: Vec<FormulaLineResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PercentageConversion {
    pub view: PercentageView,
    pub reference_mass_grams: f64,
    pub total_mass_grams: f64,
    pub lines: Vec<FormulaLineResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormulaError {
    EmptyFormula,
    InvalidTargetMass,
    MissingPercentage { symbol: Symbol },
    MissingMass { symbol: Symbol },
    NegativeValue { symbol: Symbol },
    ReferencePercentMustEqualOneHundred,
    MissingReferenceIngredients,
    PercentOfTotalExceedsOneHundred,
    InvalidPrefermentKind { kind: Symbol },
    InvalidPieceCount,
    MissingPieceMass,
    InvalidPanGeometry,
    MissingDoughDensity,
    MissingReferenceGroup { group: Symbol },
    InvalidConcentration,
    InvalidServings,
    UnsolvableConstraint { reason: String },
}

impl std::fmt::Display for FormulaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyFormula => write!(f, "formula has no ingredients"),
            Self::InvalidTargetMass => write!(f, "target mass must be greater than zero"),
            Self::MissingPercentage { symbol } => {
                write!(f, "ingredient `{symbol}` is missing a percentage")
            }
            Self::MissingMass { symbol } => write!(f, "ingredient `{symbol}` is missing a mass"),
            Self::NegativeValue { symbol } => {
                write!(f, "ingredient `{symbol}` has a negative percentage or mass")
            }
            Self::ReferencePercentMustEqualOneHundred => {
                write!(f, "reference ingredients must total 100 reference percent")
            }
            Self::MissingReferenceIngredients => write!(
                f,
                "reference-percentage formulas require at least one reference ingredient"
            ),
            Self::PercentOfTotalExceedsOneHundred => write!(
                f,
                "percent-of-total ingredients must total less than 100% when other scalable ingredients exist"
            ),
            Self::InvalidPrefermentKind { kind } => {
                write!(f, "unknown preferment kind `{kind}`")
            }
            Self::InvalidPieceCount => write!(f, "piece count must be greater than zero"),
            Self::MissingPieceMass => write!(
                f,
                "piece scaling needs a piece_mass property (or set a target mass and pieces together)"
            ),
            Self::InvalidPanGeometry => {
                write!(f, "pan diameter/depth/area/volume must be greater than zero")
            }
            Self::MissingDoughDensity => write!(
                f,
                "pan scaling needs a dough_density property in grams per millilitre"
            ),
            Self::MissingReferenceGroup { group } => {
                write!(f, "no ingredients belong to reference group `{group}`")
            }
            Self::InvalidConcentration => {
                write!(f, "concentration must be greater than zero and less than 100%")
            }
            Self::InvalidServings => write!(f, "serving count and mass must be greater than zero"),
            Self::UnsolvableConstraint { reason } => {
                write!(f, "cannot solve formula constraint: {reason}")
            }
        }
    }
}
impl std::error::Error for FormulaError {}

impl Formula {
    /// Resolve any recipe formula to a target final mass. Reference-relative,
    /// percent-of-total and fixed-mass lines may coexist.
    pub fn solve_for_target_mass(
        &self,
        target_mass_grams: f64,
    ) -> Result<FormulaResult, FormulaError> {
        if self.ingredients.is_empty() {
            return Err(FormulaError::EmptyFormula);
        }
        if !target_mass_grams.is_finite() || target_mass_grams <= 0.0 {
            return Err(FormulaError::InvalidTargetMass);
        }

        let mut fixed = 0.0;
        let mut total_pct = 0.0;
        let mut reference_line_pct = 0.0;
        let mut reference_members_pct = 0.0;
        for item in &self.ingredients {
            if !(0.0..=1.0).contains(&item.water_fraction) {
                return Err(FormulaError::NegativeValue {
                    symbol: item.symbol.clone(),
                });
            }
            match item.basis {
                FormulaBasis::ReferencePercent => {
                    let pct = item
                        .percentage
                        .ok_or_else(|| FormulaError::MissingPercentage {
                            symbol: item.symbol.clone(),
                        })?;
                    if pct < 0.0 {
                        return Err(FormulaError::NegativeValue {
                            symbol: item.symbol.clone(),
                        });
                    }
                    reference_line_pct += pct;
                    if item.is_reference {
                        reference_members_pct += pct;
                    }
                }
                FormulaBasis::PercentOfTotal => {
                    let pct = item
                        .percentage
                        .ok_or_else(|| FormulaError::MissingPercentage {
                            symbol: item.symbol.clone(),
                        })?;
                    if pct < 0.0 {
                        return Err(FormulaError::NegativeValue {
                            symbol: item.symbol.clone(),
                        });
                    }
                    total_pct += pct;
                }
                FormulaBasis::AbsoluteMass => {
                    let mass = item.mass_grams.ok_or_else(|| FormulaError::MissingMass {
                        symbol: item.symbol.clone(),
                    })?;
                    if mass < 0.0 {
                        return Err(FormulaError::NegativeValue {
                            symbol: item.symbol.clone(),
                        });
                    }
                    fixed += mass;
                }
            }
        }
        if reference_line_pct > 0.0 {
            if reference_members_pct == 0.0 {
                return Err(FormulaError::MissingReferenceIngredients);
            }
            if (reference_members_pct - 100.0).abs() > 0.001 {
                return Err(FormulaError::ReferencePercentMustEqualOneHundred);
            }
        }
        if total_pct >= 100.0 && (reference_line_pct > 0.0 || fixed > 0.0) {
            return Err(FormulaError::PercentOfTotalExceedsOneHundred);
        }

        let total_based_mass = target_mass_grams * total_pct / 100.0;
        let reference_target = target_mass_grams - fixed - total_based_mass;
        if reference_target < -0.001 {
            return Err(FormulaError::InvalidTargetMass);
        }
        let reference_mass = if reference_line_pct > 0.0 {
            reference_target / (reference_line_pct / 100.0)
        } else {
            0.0
        };

        let mut lines = Vec::with_capacity(self.ingredients.len());
        let mut total = 0.0;
        let mut flour = 0.0;
        let mut water = 0.0;
        let mut effective_water = 0.0;
        let mut salt = 0.0;
        let mut fat = 0.0;
        let mut sugar = 0.0;
        let mut prefermented_flour = 0.0;
        for item in &self.ingredients {
            let mass = match item.basis {
                FormulaBasis::ReferencePercent => {
                    reference_mass * item.percentage.unwrap_or(0.0) / 100.0
                }
                FormulaBasis::PercentOfTotal => {
                    target_mass_grams * item.percentage.unwrap_or(0.0) / 100.0
                }
                FormulaBasis::AbsoluteMass => item.mass_grams.unwrap_or(0.0),
            };
            total += mass;
            if item.is_flour {
                flour += mass;
            }
            water += mass * item.water_fraction;
            effective_water += mass * liquid_equivalent_fraction(item);
            match ingredient_role(item) {
                IngredientRole::Salt => salt += mass,
                IngredientRole::Fat => fat += mass,
                IngredientRole::Sugar => sugar += mass,
                IngredientRole::Other => {}
            }
            if item.is_flour && !matches!(item.stage.as_str(), "final" | "main") {
                prefermented_flour += mass;
            }
            lines.push(FormulaLineResult {
                ingredient_id: item.id,
                symbol: item.symbol.clone(),
                name: item.name.clone(),
                stage: item.stage.clone(),
                percentage: item.percentage,
                total_percentage: mass / target_mass_grams * 100.0,
                mass_grams: mass,
                is_reference: item.is_reference,
                is_flour: item.is_flour,
            });
        }
        Ok(FormulaResult {
            target_mass_grams,
            reference_mass_grams: reference_mass,
            total_flour_grams: flour,
            total_mass_grams: total,
            hydration_percent: if flour > 0.0 {
                water / flour * 100.0
            } else {
                0.0
            },
            prefermented_flour_percent: if flour > 0.0 {
                prefermented_flour / flour * 100.0
            } else {
                0.0
            },
            salt_percent: if flour > 0.0 {
                salt / flour * 100.0
            } else {
                0.0
            },
            fat_percent: if flour > 0.0 {
                fat / flour * 100.0
            } else {
                0.0
            },
            sugar_percent: if flour > 0.0 {
                sugar / flour * 100.0
            } else {
                0.0
            },
            effective_hydration_percent: if flour > 0.0 {
                effective_water / flour * 100.0
            } else {
                0.0
            },
            lines,
        })
    }

    /// Grams per piece from `piece_mass`, or `target` mass ÷ `pieces` when both
    /// are declared on the formula.
    pub fn piece_mass_grams(&self) -> Option<f64> {
        property_mass_grams(&self.properties, "piece_mass").or_else(|| {
            let pieces = property_number(&self.properties, "pieces")?;
            let target = property_mass_grams(&self.properties, "target")?;
            if pieces > 0.0 {
                Some(target / pieces)
            } else {
                None
            }
        })
    }

    /// Dough density in g/ml. Defaults to 1.1 when unset — typical lean dough.
    pub fn dough_density_g_per_ml(&self) -> f64 {
        property_number(&self.properties, "dough_density").unwrap_or(1.1)
    }

    /// Target dough mass for a piece count, using `piece_mass` (or target÷pieces).
    pub fn solve_for_pieces(&self, pieces: f64) -> Result<FormulaResult, FormulaError> {
        if !pieces.is_finite() || pieces <= 0.0 {
            return Err(FormulaError::InvalidPieceCount);
        }
        let piece_mass = self
            .piece_mass_grams()
            .ok_or(FormulaError::MissingPieceMass)?;
        self.solve_for_target_mass(pieces * piece_mass)
    }

    /// Target dough mass for a circular pan: π·(d/2)²·depth · density.
    /// `depth_cm` is dough thickness (pizza) or pan fill depth.
    pub fn solve_for_round_pan(
        &self,
        diameter_cm: f64,
        depth_cm: f64,
    ) -> Result<FormulaResult, FormulaError> {
        if !diameter_cm.is_finite()
            || diameter_cm <= 0.0
            || !depth_cm.is_finite()
            || depth_cm <= 0.0
        {
            return Err(FormulaError::InvalidPanGeometry);
        }
        let density = self.dough_density_g_per_ml();
        if !density.is_finite() || density <= 0.0 {
            return Err(FormulaError::MissingDoughDensity);
        }
        let radius = diameter_cm / 2.0;
        let volume_ml = std::f64::consts::PI * radius * radius * depth_cm;
        self.solve_for_target_mass(volume_ml * density)
    }

    /// Target dough mass for an explicit pan volume in millilitres.
    pub fn solve_for_pan_volume(&self, volume_ml: f64) -> Result<FormulaResult, FormulaError> {
        if !volume_ml.is_finite() || volume_ml <= 0.0 {
            return Err(FormulaError::InvalidPanGeometry);
        }
        let density = self.dough_density_g_per_ml();
        if !density.is_finite() || density <= 0.0 {
            return Err(FormulaError::MissingDoughDensity);
        }
        self.solve_for_target_mass(volume_ml * density)
    }

    /// Target dough mass for a rectangular pan footprint × depth.
    pub fn solve_for_pan_area(
        &self,
        area_cm2: f64,
        depth_cm: f64,
    ) -> Result<FormulaResult, FormulaError> {
        if !area_cm2.is_finite() || area_cm2 <= 0.0 || !depth_cm.is_finite() || depth_cm <= 0.0 {
            return Err(FormulaError::InvalidPanGeometry);
        }
        let density = self.dough_density_g_per_ml();
        if !density.is_finite() || density <= 0.0 {
            return Err(FormulaError::MissingDoughDensity);
        }
        self.solve_for_target_mass(area_cm2 * depth_cm * density)
    }

    /// Convert entered ingredient weights back to either percentages of the
    /// selected reference group or percentages of total recipe mass.
    pub fn weights_to_percentages(
        &self,
        view: PercentageView,
    ) -> Result<PercentageConversion, FormulaError> {
        if self.ingredients.is_empty() {
            return Err(FormulaError::EmptyFormula);
        }
        let mut total = 0.0;
        let mut reference = 0.0;
        for item in &self.ingredients {
            let mass = item.mass_grams.ok_or_else(|| FormulaError::MissingMass {
                symbol: item.symbol.clone(),
            })?;
            if mass < 0.0 {
                return Err(FormulaError::NegativeValue {
                    symbol: item.symbol.clone(),
                });
            }
            total += mass;
            if item.is_reference {
                reference += mass;
            }
        }
        if total <= 0.0 {
            return Err(FormulaError::InvalidTargetMass);
        }
        if view == PercentageView::Reference && reference <= 0.0 {
            return Err(FormulaError::MissingReferenceIngredients);
        }
        let divisor = match view {
            PercentageView::Reference => reference,
            PercentageView::Total => total,
        };
        let lines = self
            .ingredients
            .iter()
            .map(|item| {
                let mass = item.mass_grams.unwrap_or(0.0);
                FormulaLineResult {
                    ingredient_id: item.id,
                    symbol: item.symbol.clone(),
                    name: item.name.clone(),
                    stage: item.stage.clone(),
                    percentage: Some(mass / divisor * 100.0),
                    total_percentage: mass / total * 100.0,
                    mass_grams: mass,
                    is_reference: item.is_reference,
                    is_flour: item.is_flour,
                }
            })
            .collect();
        Ok(PercentageConversion {
            view,
            reference_mass_grams: reference,
            total_mass_grams: total,
            lines,
        })
    }
}

pub(crate) fn property_number(properties: &BTreeMap<Symbol, Value>, key: &str) -> Option<f64> {
    match properties.get(key)? {
        Value::Number(n) => Some(*n),
        Value::Quantity(q) => Some(q.value),
        _ => None,
    }
}

pub(crate) fn property_mass_grams(properties: &BTreeMap<Symbol, Value>, key: &str) -> Option<f64> {
    match properties.get(key)? {
        Value::Quantity(q) => q.as_grams(),
        Value::Number(n) => Some(*n),
        Value::Text(text) => quantity_from_text(text).and_then(|q| q.as_grams()),
        _ => None,
    }
}

#[cfg(test)]
mod test;
