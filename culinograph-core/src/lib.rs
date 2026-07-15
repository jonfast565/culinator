use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum Value {
    Text(String),
    Number(f64),
    Quantity(Quantity),
    Boolean(bool),
    Symbol(Symbol),
    List(Vec<Value>),
    Object(BTreeMap<Symbol, Value>),
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
            lines,
        })
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
#[cfg(test)]
mod test;
