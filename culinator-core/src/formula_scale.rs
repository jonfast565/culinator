//! Advanced formula scaling: reference groups, constraint solving, rounding,
//! minimums, serving/concentration targets, and inheritance (§7 ENHANCEMENTS).

use crate::{
    Formula, FormulaError, FormulaIngredient, FormulaLineResult, FormulaResult, PercentageView,
    Symbol, Value, property_mass_grams, property_number,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// How finely masses are rounded after a solve (scale precision).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RoundingPolicy {
    /// Smallest mass step in grams (e.g. `0.1` for a 0.1 g scale, `1.0` for 1 g).
    pub increment_grams: f64,
}

impl RoundingPolicy {
    pub fn grams(increment_grams: f64) -> Self {
        Self { increment_grams }
    }

    pub fn round(self, mass: f64) -> f64 {
        if !(self.increment_grams > 0.0) || !mass.is_finite() {
            return mass;
        }
        (mass / self.increment_grams).round() * self.increment_grams
    }
}

/// Any supported input that can determine a batch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum FormulaConstraint {
    TargetMass { grams: f64 },
    /// Flour (or named reference-group) mass is known; scale the rest.
    ReferenceMass {
        grams: f64,
        #[serde(default)]
        group: Option<Symbol>,
    },
    Pieces { count: f64 },
    RoundPan { diameter_cm: f64, depth_cm: f64 },
    PanVolume { millilitres: f64 },
    PanArea { area_cm2: f64, depth_cm: f64 },
    Servings {
        count: f64,
        grams_per_serving: f64,
    },
    /// Solute as a percent of total batch mass (brine / syrup style).
    Concentration {
        solute: Symbol,
        percent_of_total: f64,
    },
}

/// Percentages against one named reference group (or `"total"`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NamedBasisView {
    pub name: Symbol,
    pub reference_mass_grams: f64,
    pub lines: Vec<FormulaLineResult>,
}

fn ingredient_group(item: &FormulaIngredient) -> Symbol {
    match item.properties.get("reference_group") {
        Some(Value::Text(name)) | Some(Value::Symbol(name)) if !name.is_empty() => name.clone(),
        _ if item.is_reference => "main".into(),
        _ => String::new(),
    }
}

impl Formula {
    /// Named reference groups present on this formula (includes `"main"` when
    /// any ingredient is flagged `is_reference` without an explicit group).
    pub fn reference_groups(&self) -> BTreeSet<Symbol> {
        let mut groups = BTreeSet::new();
        for item in &self.ingredients {
            let group = ingredient_group(item);
            if !group.is_empty() {
                groups.insert(group);
            }
        }
        groups
    }

    /// Restate weights as percentages of a named reference group, or of total
    /// when `group` is `"total"` / `None` with [`PercentageView::Total`].
    pub fn weights_to_percentages_of_group(
        &self,
        group: &str,
    ) -> Result<PercentageConversionExt, FormulaError> {
        if self.ingredients.is_empty() {
            return Err(FormulaError::EmptyFormula);
        }
        let mut total = 0.0;
        let mut reference = 0.0;
        let use_total = group.eq_ignore_ascii_case("total");
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
            if use_total {
                continue;
            }
            // Named group: members flagged `is_reference` (or every member of the
            // group when none are flagged) form the 100% pool.
            if ingredient_group(item) == group && item.is_reference {
                reference += mass;
            }
        }
        if !use_total && reference <= 0.0 {
            for item in &self.ingredients {
                if ingredient_group(item) == group {
                    reference += item.mass_grams.unwrap_or(0.0);
                }
            }
        }
        if total <= 0.0 {
            return Err(FormulaError::InvalidTargetMass);
        }
        let divisor = if use_total {
            total
        } else {
            if reference <= 0.0 {
                return Err(FormulaError::MissingReferenceGroup {
                    group: group.to_owned(),
                });
            }
            reference
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
        Ok(PercentageConversionExt {
            view: if use_total {
                PercentageView::Total
            } else {
                PercentageView::Reference
            },
            group: group.to_owned(),
            reference_mass_grams: if use_total { 0.0 } else { reference },
            total_mass_grams: total,
            lines,
        })
    }

    /// Compute percentages for every reference group plus total in one pass.
    pub fn simultaneous_bases_from_weights(&self) -> Result<Vec<NamedBasisView>, FormulaError> {
        let mut views = Vec::new();
        for group in self.reference_groups() {
            let conversion = self.weights_to_percentages_of_group(&group)?;
            views.push(NamedBasisView {
                name: group,
                reference_mass_grams: conversion.reference_mass_grams,
                lines: conversion.lines,
            });
        }
        let total = self.weights_to_percentages_of_group("total")?;
        views.push(NamedBasisView {
            name: "total".into(),
            reference_mass_grams: total.total_mass_grams,
            lines: total.lines,
        });
        Ok(views)
    }

    /// Target mass for N servings of a known piece mass.
    pub fn solve_for_servings(
        &self,
        servings: f64,
        grams_per_serving: f64,
    ) -> Result<FormulaResult, FormulaError> {
        if !(servings > 0.0) || !(grams_per_serving > 0.0) {
            return Err(FormulaError::InvalidServings);
        }
        self.solve_for_target_mass(servings * grams_per_serving)
    }

    /// Scale so `solute` is `percent_of_total` of the batch (e.g. 2% salt brine
    /// relative to total, when salt is on a percent-of-total or reference basis
    /// that we can invert).
    pub fn solve_for_concentration(
        &self,
        solute: &str,
        percent_of_total: f64,
    ) -> Result<FormulaResult, FormulaError> {
        if !(percent_of_total > 0.0) || percent_of_total >= 100.0 {
            return Err(FormulaError::InvalidConcentration);
        }
        let item = self
            .ingredients
            .iter()
            .find(|row| row.symbol == solute)
            .ok_or_else(|| FormulaError::UnsolvableConstraint {
                reason: format!("no ingredient `{solute}`"),
            })?;
        // Prefer an absolute mass for the solute: target = mass / (pct/100).
        if let Some(mass) = item.mass_grams.filter(|m| *m > 0.0) {
            return self.solve_for_target_mass(mass / (percent_of_total / 100.0));
        }
        // Otherwise assume reference-percent baker's formula and pick any
        // target, then rescale so solute share matches — use flour mass = 1000
        // as a probe then rescale.
        let probe = self.solve_for_target_mass(1000.0)?;
        let line = probe
            .lines
            .iter()
            .find(|line| line.symbol == solute)
            .ok_or_else(|| FormulaError::UnsolvableConstraint {
                reason: format!("solute `{solute}` missing from solve"),
            })?;
        if line.mass_grams <= 0.0 {
            return Err(FormulaError::UnsolvableConstraint {
                reason: format!("solute `{solute}` solved to zero mass"),
            });
        }
        let actual_pct = line.mass_grams / probe.total_mass_grams * 100.0;
        if actual_pct <= 0.0 {
            return Err(FormulaError::InvalidConcentration);
        }
        let target = 1000.0 * (actual_pct / percent_of_total);
        self.solve_for_target_mass(target)
    }

    /// When flour (or a reference group) mass is known, derive the batch.
    pub fn solve_for_reference_mass(
        &self,
        reference_mass_grams: f64,
        group: Option<&str>,
    ) -> Result<FormulaResult, FormulaError> {
        if !(reference_mass_grams > 0.0) {
            return Err(FormulaError::InvalidTargetMass);
        }
        let group_name = group.unwrap_or("main");
        let mut reference_pct = 0.0;
        let mut scalable_pct = 0.0;
        let mut fixed = 0.0;
        for item in &self.ingredients {
            match item.basis {
                crate::FormulaBasis::AbsoluteMass => {
                    fixed += item.mass_grams.unwrap_or(0.0);
                }
                crate::FormulaBasis::PercentOfTotal => {
                    // Handled after we know target — approximate via iteration below.
                    scalable_pct += item.percentage.unwrap_or(0.0);
                }
                crate::FormulaBasis::ReferencePercent => {
                    let pct = item.percentage.unwrap_or(0.0);
                    let in_group = if group.is_some() {
                        ingredient_group(item) == group_name
                    } else {
                        item.is_reference
                    };
                    if in_group {
                        reference_pct += pct;
                    }
                }
            }
        }
        if reference_pct <= 0.0 {
            // Fall back: all flour rows.
            reference_pct = self
                .ingredients
                .iter()
                .filter(|item| item.is_flour)
                .map(|item| item.percentage.unwrap_or(0.0))
                .sum();
        }
        if reference_pct <= 0.0 {
            return Err(FormulaError::MissingReferenceIngredients);
        }
        // reference_mass corresponds to reference_pct of the reference basis.
        // Total reference-basis mass = reference_mass / (reference_members/100)
        // For baker's %, reference members total 100, so reference_mass is flour.
        let reference_basis_mass = reference_mass_grams / (reference_pct / 100.0);
        let mut reference_line_pct = 0.0;
        for item in &self.ingredients {
            if item.basis == crate::FormulaBasis::ReferencePercent {
                reference_line_pct += item.percentage.unwrap_or(0.0);
            }
        }
        let reference_portion = reference_basis_mass * (reference_line_pct / 100.0);
        // percent-of-total share: target = (reference_portion + fixed) / (1 - total_pct/100)
        let target = if scalable_pct > 0.0 && scalable_pct < 100.0 {
            (reference_portion + fixed) / (1.0 - scalable_pct / 100.0)
        } else {
            reference_portion + fixed
        };
        self.solve_for_target_mass(target)
    }

    /// Dispatch any supported constraint to a concrete solve.
    pub fn solve(&self, constraint: &FormulaConstraint) -> Result<FormulaResult, FormulaError> {
        match constraint {
            FormulaConstraint::TargetMass { grams } => self.solve_for_target_mass(*grams),
            FormulaConstraint::ReferenceMass { grams, group } => {
                self.solve_for_reference_mass(*grams, group.as_deref())
            }
            FormulaConstraint::Pieces { count } => self.solve_for_pieces(*count),
            FormulaConstraint::RoundPan {
                diameter_cm,
                depth_cm,
            } => self.solve_for_round_pan(*diameter_cm, *depth_cm),
            FormulaConstraint::PanVolume { millilitres } => self.solve_for_pan_volume(*millilitres),
            FormulaConstraint::PanArea { area_cm2, depth_cm } => {
                self.solve_for_pan_area(*area_cm2, *depth_cm)
            }
            FormulaConstraint::Servings {
                count,
                grams_per_serving,
            } => self.solve_for_servings(*count, *grams_per_serving),
            FormulaConstraint::Concentration {
                solute,
                percent_of_total,
            } => self.solve_for_concentration(solute, *percent_of_total),
        }
    }

    /// Apply scale-precision rounding to every line (and recompute totals).
    pub fn apply_rounding(
        &self,
        mut result: FormulaResult,
        policy: RoundingPolicy,
    ) -> FormulaResult {
        for line in &mut result.lines {
            line.mass_grams = policy.round(line.mass_grams);
        }
        recompute_totals(&mut result);
        result
    }

    /// Raise any line below its `min_mass_grams` property (or the formula-wide
    /// `min_mass` property) up to that floor. Totals drift slightly by design —
    /// bakers prefer a usable pinch over a theoretically exact 0.05 g.
    pub fn apply_minimums(&self, mut result: FormulaResult) -> FormulaResult {
        let formula_min = property_mass_grams(&self.properties, "min_mass").unwrap_or(0.0);
        for (index, item) in self.ingredients.iter().enumerate() {
            let min = property_mass_grams(&item.properties, "min_mass")
                .or_else(|| property_number(&item.properties, "min_mass_grams"))
                .unwrap_or(formula_min);
            if min > 0.0 {
                if let Some(line) = result.lines.get_mut(index) {
                    if line.mass_grams > 0.0 && line.mass_grams < min {
                        line.mass_grams = min;
                    }
                }
            }
        }
        recompute_totals(&mut result);
        result
    }

    /// Overlay `self` onto `parent`: ingredients matched by symbol replace the
    /// parent row; unmatched parent rows are kept; properties merge with child
    /// winning on key collision. Used for formula inheritance / versioning.
    pub fn inherit_from(&self, parent: &Formula) -> Formula {
        let mut by_symbol: BTreeMap<Symbol, FormulaIngredient> = BTreeMap::new();
        for item in &parent.ingredients {
            by_symbol.insert(item.symbol.clone(), item.clone());
        }
        for item in &self.ingredients {
            by_symbol.insert(item.symbol.clone(), item.clone());
        }
        let mut properties = parent.properties.clone();
        for (key, value) in &self.properties {
            properties.insert(key.clone(), value.clone());
        }
        Formula {
            id: self.id,
            recipe_id: self.recipe_id.or(parent.recipe_id),
            symbol: self.symbol.clone(),
            name: if self.name.is_empty() {
                parent.name.clone()
            } else {
                self.name.clone()
            },
            basis: self.basis,
            ingredients: by_symbol.into_values().collect(),
            properties,
        }
    }

    /// Diff two formula versions: symbols added, removed, or whose baker % changed.
    pub fn compare_versions(&self, other: &Formula) -> FormulaVersionDiff {
        let left: BTreeMap<&str, &FormulaIngredient> = self
            .ingredients
            .iter()
            .map(|item| (item.symbol.as_str(), item))
            .collect();
        let right: BTreeMap<&str, &FormulaIngredient> = other
            .ingredients
            .iter()
            .map(|item| (item.symbol.as_str(), item))
            .collect();
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();
        for (symbol, item) in &right {
            match left.get(symbol) {
                None => added.push((*symbol).to_owned()),
                Some(previous) => {
                    if previous.percentage != item.percentage
                        || previous.mass_grams != item.mass_grams
                        || previous.basis != item.basis
                    {
                        changed.push(FormulaIngredientChange {
                            symbol: (*symbol).to_owned(),
                            from_percentage: previous.percentage,
                            to_percentage: item.percentage,
                            from_mass_grams: previous.mass_grams,
                            to_mass_grams: item.mass_grams,
                        });
                    }
                }
            }
        }
        for symbol in left.keys() {
            if !right.contains_key(symbol) {
                removed.push((*symbol).to_owned());
            }
        }
        FormulaVersionDiff {
            added,
            removed,
            changed,
        }
    }
}

/// Like [`crate::PercentageConversion`] but tagged with the group name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PercentageConversionExt {
    pub view: PercentageView,
    pub group: Symbol,
    pub reference_mass_grams: f64,
    pub total_mass_grams: f64,
    pub lines: Vec<FormulaLineResult>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulaIngredientChange {
    pub symbol: Symbol,
    pub from_percentage: Option<f64>,
    pub to_percentage: Option<f64>,
    pub from_mass_grams: Option<f64>,
    pub to_mass_grams: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormulaVersionDiff {
    pub added: Vec<Symbol>,
    pub removed: Vec<Symbol>,
    pub changed: Vec<FormulaIngredientChange>,
}

fn recompute_totals(result: &mut FormulaResult) {
    let total: f64 = result.lines.iter().map(|line| line.mass_grams).sum();
    result.total_mass_grams = total;
    result.target_mass_grams = total;
    let mut flour = 0.0;
    for line in &mut result.lines {
        if line.is_flour {
            flour += line.mass_grams;
        }
        line.total_percentage = if total > 0.0 {
            line.mass_grams / total * 100.0
        } else {
            0.0
        };
    }
    result.total_flour_grams = flour;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{FormulaBasis, Quantity, Dimension};
    use uuid::Uuid;

    fn item(symbol: &str, pct: f64, reference: bool, flour: bool, water: f64) -> FormulaIngredient {
        FormulaIngredient {
            id: Uuid::new_v4(),
            symbol: symbol.into(),
            name: symbol.into(),
            stage: "final".into(),
            basis: FormulaBasis::ReferencePercent,
            percentage: Some(pct),
            mass_grams: None,
            is_reference: reference,
            is_flour: flour,
            water_fraction: water,
            scalable: true,
            properties: BTreeMap::new(),
        }
    }

    fn dough() -> Formula {
        Formula {
            id: Uuid::new_v4(),
            recipe_id: None,
            symbol: "dough".into(),
            name: "Dough".into(),
            basis: FormulaBasis::ReferencePercent,
            ingredients: vec![
                item("flour", 100.0, true, true, 0.0),
                item("water", 70.0, false, false, 1.0),
                item("salt", 2.0, false, false, 0.0),
            ],
            properties: BTreeMap::new(),
        }
    }

    #[test]
    fn solves_from_flour_mass_constraint() {
        let result = dough()
            .solve(&FormulaConstraint::ReferenceMass {
                grams: 500.0,
                group: None,
            })
            .expect("solves");
        assert!((result.total_flour_grams - 500.0).abs() < 0.5);
        assert!((result.hydration_percent - 70.0).abs() < 0.1);
    }

    #[test]
    fn solves_servings() {
        let result = dough()
            .solve(&FormulaConstraint::Servings {
                count: 4.0,
                grams_per_serving: 200.0,
            })
            .expect("solves");
        assert!((result.target_mass_grams - 800.0).abs() < 0.001);
    }

    #[test]
    fn rounds_to_scale_precision() {
        let formula = dough();
        let raw = formula.solve_for_target_mass(1000.0).expect("solves");
        let rounded = formula.apply_rounding(raw, RoundingPolicy::grams(1.0));
        for line in &rounded.lines {
            assert_eq!(line.mass_grams, line.mass_grams.round());
        }
    }

    #[test]
    fn applies_minimum_mass() {
        let mut formula = dough();
        formula.ingredients[2].properties.insert(
            "min_mass".into(),
            Value::Quantity(Quantity {
                value: 15.0,
                unit: "g".into(),
                dimension: Dimension::Mass,
            }),
        );
        let raw = formula.solve_for_target_mass(500.0).expect("solves");
        // Salt at 2% of ~291 g flour ≈ 5.8 g → bumped to 15 g.
        let adjusted = formula.apply_minimums(raw);
        let salt = adjusted
            .lines
            .iter()
            .find(|line| line.symbol == "salt")
            .expect("salt");
        assert!((salt.mass_grams - 15.0).abs() < 0.001);
    }

    #[test]
    fn inherit_and_compare_versions() {
        let parent = dough();
        let mut child = dough();
        child.ingredients[1].percentage = Some(75.0);
        child.ingredients.push(item("oil", 3.0, false, false, 0.0));
        let merged = child.inherit_from(&parent);
        assert_eq!(merged.ingredients.len(), 4);
        let diff = parent.compare_versions(&merged);
        assert!(diff.added.contains(&"oil".to_owned()));
        assert!(diff.changed.iter().any(|c| c.symbol == "water"));
    }

    #[test]
    fn reference_group_percentages() {
        let mut preferment_flour = item("levain_flour", 20.0, true, true, 0.0);
        preferment_flour
            .properties
            .insert("reference_group".into(), Value::Text("levain".into()));
        preferment_flour.mass_grams = Some(100.0);
        let mut preferment_water = item("levain_water", 20.0, false, false, 1.0);
        preferment_water
            .properties
            .insert("reference_group".into(), Value::Text("levain".into()));
        preferment_water.mass_grams = Some(100.0);
        let mut final_flour = item("flour", 80.0, true, true, 0.0);
        final_flour.mass_grams = Some(400.0);
        let formula = Formula {
            id: Uuid::new_v4(),
            recipe_id: None,
            symbol: "dough".into(),
            name: "Dough".into(),
            basis: FormulaBasis::ReferencePercent,
            ingredients: vec![preferment_flour, preferment_water, final_flour],
            properties: BTreeMap::new(),
        };
        let levain = formula
            .weights_to_percentages_of_group("levain")
            .expect("group");
        assert!((levain.reference_mass_grams - 100.0).abs() < 0.001);
        let bases = formula.simultaneous_bases_from_weights().expect("bases");
        assert!(bases.iter().any(|b| b.name == "levain"));
        assert!(bases.iter().any(|b| b.name == "total"));
    }

    #[test]
    fn concentration_from_absolute_solute() {
        let mut salt = item("salt", 0.0, false, false, 0.0);
        salt.basis = FormulaBasis::AbsoluteMass;
        salt.percentage = None;
        salt.mass_grams = Some(20.0);
        let mut water = item("water", 0.0, false, false, 1.0);
        water.basis = FormulaBasis::PercentOfTotal;
        water.percentage = Some(98.0);
        // Actually for concentration with absolute salt 20g at 2% → target 1000g.
        let formula = Formula {
            id: Uuid::new_v4(),
            recipe_id: None,
            symbol: "brine".into(),
            name: "Brine".into(),
            basis: FormulaBasis::PercentOfTotal,
            ingredients: vec![salt, water],
            properties: BTreeMap::new(),
        };
        // Fix water to fill remaining — use absolute salt only for concentration solve.
        let formula = Formula {
            ingredients: vec![FormulaIngredient {
                id: Uuid::new_v4(),
                symbol: "salt".into(),
                name: "salt".into(),
                stage: "final".into(),
                basis: FormulaBasis::AbsoluteMass,
                percentage: None,
                mass_grams: Some(20.0),
                is_reference: false,
                is_flour: false,
                water_fraction: 0.0,
                scalable: false,
                properties: BTreeMap::new(),
            }],
            ..formula
        };
        let result = formula
            .solve_for_concentration("salt", 2.0)
            .expect("concentration");
        assert!((result.target_mass_grams - 1000.0).abs() < 0.001);
    }
}
