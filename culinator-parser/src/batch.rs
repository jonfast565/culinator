//! A lossless recipe-batch preview shared by the editor and catalog front ends.
use crate::{Outline, OutlineNode, TextEdit, apply_text_edits, parse_recipe};
use culinator_core::{
    Dimension, Formula, FormulaBasis, FormulaConstraint, FormulaResult, IngredientDensity,
    Quantity, RoundingPolicy, Value,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchRequest {
    #[serde(default)]
    pub source_text: String,
    pub formula_symbol: String,
    /// Full parsed formula plus the editor's changes. None means use the source formula.
    pub formula: Option<Formula>,
    pub constraint: FormulaConstraint,
    #[serde(default)]
    pub rounding_increment_grams: Option<f64>,
    #[serde(default)]
    pub apply_minimums: bool,
    /// Set only when the author explicitly changes the number of pieces.
    #[serde(default)]
    pub piece_count: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchChange {
    pub path: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchPreview {
    pub source_fingerprint: String,
    pub proposed_source: String,
    pub source_diff: String,
    pub result: Option<FormulaResult>,
    pub changes: Vec<BatchChange>,
    pub warnings: Vec<String>,
    pub blockers: Vec<String>,
    pub out_of_sync: Vec<String>,
}

/// A source hash used to reject a preview after another edit has changed the document.
fn fingerprint(source: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in source.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// Return the exact preview being applied; reject stale or incomplete previews.
pub fn apply_batch(
    request: &BatchRequest,
    expected_fingerprint: &str,
) -> Result<BatchPreview, String> {
    if fingerprint(&request.source_text) != expected_fingerprint {
        return Err("Recipe source changed after the batch preview".into());
    }
    let preview = plan_batch(request);
    if !preview.blockers.is_empty() {
        return Err(preview.blockers.join("; "));
    }
    Ok(preview)
}

pub fn plan_batch(request: &BatchRequest) -> BatchPreview {
    let source = &request.source_text;
    let mut preview = BatchPreview {
        source_fingerprint: fingerprint(source),
        proposed_source: source.clone(),
        source_diff: String::new(),
        result: None,
        changes: Vec::new(),
        warnings: Vec::new(),
        blockers: Vec::new(),
        out_of_sync: Vec::new(),
    };
    let recipe = match parse_recipe(source) {
        Ok(recipe) => recipe,
        Err(error) => {
            preview
                .blockers
                .push(format!("Invalid recipe source: {error}"));
            return preview;
        }
    };
    let Some(base) = recipe
        .formulas
        .iter()
        .find(|f| f.symbol == request.formula_symbol)
    else {
        if let Some(formula) = &request.formula {
            match create_formula_source(source, formula) {
                Ok(created) => {
                    if !parse_recipe(&created).ok().is_some_and(|parsed| {
                        parsed
                            .formulas
                            .iter()
                            .any(|item| item.symbol == formula.symbol)
                    }) {
                        preview
                            .blockers
                            .push("New formula could not be parsed".into());
                        return preview;
                    }
                    let mut nested = request.clone();
                    nested.source_text = created;
                    let mut calculated = plan_batch(&nested);
                    calculated.source_fingerprint = preview.source_fingerprint;
                    calculated.changes.insert(
                        0,
                        BatchChange {
                            path: format!("formula.{}", formula.symbol),
                            before: String::new(),
                            after: "created".into(),
                        },
                    );
                    if !calculated.blockers.is_empty() {
                        calculated.proposed_source = source.clone();
                        calculated.source_diff.clear();
                    } else {
                        calculated.source_diff = source_diff(source, &calculated.proposed_source);
                    }
                    return calculated;
                }
                Err(error) => preview.blockers.push(error),
            }
        } else {
            preview
                .blockers
                .push(format!("Formula `{}` is missing", request.formula_symbol));
        }
        return preview;
    };
    let formula = request.formula.as_ref().unwrap_or(base);
    if formula.symbol != base.symbol {
        preview
            .blockers
            .push("Formula symbol cannot change during batch planning".into());
        return preview;
    }
    let mut result = match formula.solve(&request.constraint) {
        Ok(result) => result,
        Err(error) => {
            preview.blockers.push(error.to_string());
            return preview;
        }
    };
    if let Some(increment) = request.rounding_increment_grams {
        if !increment.is_finite() || increment <= 0.0 {
            preview
                .blockers
                .push("Rounding increment must be positive".into());
            return preview;
        }
        result = formula.apply_rounding(result, RoundingPolicy::grams(increment));
    }
    if request.apply_minimums {
        result = formula.apply_minimums(result);
    }
    preview.result = Some(result.clone());

    let mut seen = HashSet::new();
    let mut factors = HashMap::new();
    let density = IngredientDensity::new();
    for line in &result.lines {
        if !seen.insert(&line.symbol) {
            preview
                .blockers
                .push(format!("Duplicate formula row `{}`", line.symbol));
            continue;
        }
        let Some(resource) = recipe.resources.iter().find(|r| {
            r.symbol == line.symbol && r.kind == culinator_core::ResourceKind::Ingredient
        }) else {
            preview.blockers.push(format!(
                "No recipe ingredient matches formula row `{}`",
                line.symbol
            ));
            continue;
        };
        let old = resource.properties.get("quantity");
        let old_quantity = match old {
            Some(Value::Quantity(q)) => Some(q),
            None => None,
            _ => {
                preview.blockers.push(format!(
                    "Ingredient `{}` has a range or unsupported quantity",
                    line.symbol
                ));
                continue;
            }
        };
        let hint = resource_hint(resource);
        let old_grams = old_quantity.and_then(|q| mass_grams(q, hint, &density));
        if let Some(total) = old_grams {
            let used: f64 = recipe
                .operations
                .iter()
                .flat_map(|op| op.bindings.iter())
                .filter(|binding| {
                    binding.role == culinator_core::BindingRole::Input
                        && binding.resource == line.symbol
                })
                .filter_map(|binding| {
                    binding
                        .quantity
                        .as_ref()
                        .and_then(|q| mass_grams(q, hint, &density))
                })
                .sum();
            if used > total + 0.11 {
                preview.blockers.push(format!(
                    "Explicit step amounts for `{}` exceed its declared ingredient quantity",
                    line.symbol
                ));
            }
        }
        let unit = old_quantity.map_or("g", |q| q.unit.as_str());
        let new_quantity = match quantity_for_mass(line.mass_grams, unit, hint, &density) {
            Some(q) => q,
            None => {
                preview.blockers.push(format!(
                    "Cannot convert `{}` to its authored unit `{unit}`",
                    line.symbol
                ));
                continue;
            }
        };
        if let Some(old) = old_grams {
            if old > 0.0 {
                factors.insert(line.symbol.as_str(), line.mass_grams / old);
            } else if line.mass_grams > 0.0 {
                preview.blockers.push(format!(
                    "Ingredient `{}` has zero mass, so its step amounts cannot be scaled",
                    line.symbol
                ));
            }
        }
        if let Some(before) = old_quantity {
            let before_text = format_quantity(before);
            let after_text = format_quantity(&new_quantity);
            if before_text != after_text {
                preview.changes.push(BatchChange {
                    path: format!("ingredient.{}.quantity", line.symbol),
                    before: before_text,
                    after: after_text.clone(),
                });
                if let Err(error) = set_child_statement(
                    &mut preview.proposed_source,
                    "ingredient",
                    &line.symbol,
                    "quantity",
                    &after_text,
                ) {
                    preview.blockers.push(error);
                }
            }
        } else {
            let after_text = format_quantity(&new_quantity);
            preview.changes.push(BatchChange {
                path: format!("ingredient.{}.quantity", line.symbol),
                before: String::new(),
                after: after_text.clone(),
            });
            if let Err(error) = set_child_statement(
                &mut preview.proposed_source,
                "ingredient",
                &line.symbol,
                "quantity",
                &after_text,
            ) {
                preview.blockers.push(error);
            }
        }
    }

    // Diagnose drift against the *saved* formula and its own target, before editing it.
    if let Some(Value::Quantity(target)) = base.properties.get("target") {
        if let Some(target_grams) = target.as_grams()
            && let Ok(saved) = base.solve_for_target_mass(target_grams)
        {
            for line in &saved.lines {
                if let Some(resource) = recipe.resources.iter().find(|r| r.symbol == line.symbol)
                    && let Some(Value::Quantity(q)) = resource.properties.get("quantity")
                    && let Some(grams) = mass_grams(q, resource_hint(resource), &density)
                    && (grams - line.mass_grams).abs() > 0.11
                {
                    preview.out_of_sync.push(format!(
                        "{}: recipe has {}, formula predicts {} g",
                        line.symbol,
                        format_quantity(q),
                        number(line.mass_grams)
                    ));
                }
            }
        }
    }

    scale_inputs(&recipe, formula, &factors, &mut preview);
    update_yields(&recipe, request, &mut preview);
    if let Err(error) = patch_formula(base, formula, request, &result, &mut preview) {
        preview.blockers.push(error);
    }
    warn_prose(&recipe, &mut preview);
    if !preview.blockers.is_empty() {
        preview.proposed_source = source.clone();
        return preview;
    }
    if let Err(error) = parse_recipe(&preview.proposed_source) {
        preview
            .blockers
            .push(format!("Proposed recipe does not parse: {error}"));
        preview.proposed_source = source.clone();
    }
    if preview.blockers.is_empty() {
        preview.source_diff = source_diff(source, &preview.proposed_source);
    }
    preview
}

fn source_diff(before: &str, after: &str) -> String {
    if before == after {
        return String::new();
    }
    let old: Vec<_> = before.split_inclusive('\n').collect();
    let new: Vec<_> = after.split_inclusive('\n').collect();
    let prefix = old
        .iter()
        .zip(&new)
        .take_while(|(left, right)| left == right)
        .count();
    let suffix = old[prefix..]
        .iter()
        .rev()
        .zip(new[prefix..].iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    let old_end = old.len() - suffix;
    let new_end = new.len() - suffix;
    let mut diff = format!(
        "--- current\n+++ proposed\n@@ -{},{} +{},{} @@\n",
        prefix + 1,
        old_end - prefix,
        prefix + 1,
        new_end - prefix
    );
    for line in &old[prefix..old_end] {
        diff.push('-');
        diff.push_str(line);
        if !line.ends_with('\n') {
            diff.push('\n');
        }
    }
    for line in &new[prefix..new_end] {
        diff.push('+');
        diff.push_str(line);
        if !line.ends_with('\n') {
            diff.push('\n');
        }
    }
    diff
}

fn create_formula_source(source: &str, formula: &Formula) -> Result<String, String> {
    let outline = Outline::parse(source).map_err(|e| e.to_string())?;
    let recipe = outline.recipe().ok_or("No recipe block")?;
    let inner = recipe.block_inner_range.ok_or("Recipe has no block")?;
    let basis =
        match formula.basis {
            FormulaBasis::ReferencePercent => "relative to flour",
            FormulaBasis::PercentOfTotal => "of total",
            FormulaBasis::AbsoluteMass => return Err(
                "An absolute-mass formula needs an explicit formula basis in the recipe builder"
                    .into(),
            ),
        };
    let mut block = format!("\n    formula {} {} {{", formula.symbol, basis);
    for item in &formula.ingredients {
        block.push_str(&format!("\n        ingredient {} {{", item.symbol));
        if let Some(pct) = item.percentage {
            block.push_str(&format!("\n            percentage {}%;", number(pct)));
        }
        if item.basis == FormulaBasis::AbsoluteMass {
            if let Some(mass) = item.mass_grams {
                block.push_str(&format!("\n            mass {} g;", number(mass)));
            }
            block.push_str("\n            basis absolute;");
        }
        if item.is_reference {
            block.push_str("\n            reference true;");
        }
        if item.is_flour {
            block.push_str("\n            flour true;");
        }
        if item.water_fraction > 0.0 {
            block.push_str(&format!(
                "\n            water_fraction {};",
                number(item.water_fraction)
            ));
        }
        if item.stage != "final" {
            block.push_str(&format!("\n            stage {};", item.stage));
        }
        for (key, value) in &item.properties {
            if key == "sourceQuantity"
                || matches!(
                    key.as_str(),
                    "percentage"
                        | "baker"
                        | "mass"
                        | "quantity"
                        | "reference"
                        | "flour"
                        | "water_fraction"
                        | "stage"
                )
            {
                continue;
            }
            if let Some(value) = value_text(value) {
                block.push_str(&format!("\n            {key} {value};"));
            }
        }
        block.push_str("\n        }");
    }
    block.push_str("\n    }\n");
    apply_text_edits(source, &[TextEdit::insert(inner.end, block)]).map_err(|e| e.to_string())
}

fn mass_grams(q: &Quantity, symbol: &str, density: &IngredientDensity) -> Option<f64> {
    match q.dimension {
        Dimension::Mass => q.as_grams(),
        Dimension::Volume => q
            .to_mass(density.density_g_per_ml(symbol)?)
            .ok()?
            .as_grams(),
        _ => None,
    }
}

fn resource_hint(resource: &culinator_core::Resource) -> &str {
    match resource.properties.get("name") {
        Some(Value::Text(name)) | Some(Value::Symbol(name)) => name,
        _ => &resource.symbol,
    }
}

fn quantity_for_mass(
    grams: f64,
    unit: &str,
    symbol: &str,
    density: &IngredientDensity,
) -> Option<Quantity> {
    if !grams.is_finite() || grams < 0.0 {
        return None;
    }
    let mass = Quantity {
        value: grams,
        unit: "g".into(),
        dimension: Dimension::Mass,
    };
    match Dimension::from_unit(unit) {
        Dimension::Mass => mass.convert_to(unit).ok(),
        Dimension::Volume => mass
            .to_volume(density.density_g_per_ml(symbol)?)
            .ok()?
            .convert_to(unit)
            .ok(),
        _ => None,
    }
}

fn number(n: f64) -> String {
    let rounded = (n * 1000.0).round() / 1000.0;
    format!("{rounded}")
}
fn format_quantity(q: &Quantity) -> String {
    format!("{} {}", number(q.value), q.unit)
}

fn node_for<'a>(outline: &'a Outline, kind: &str, symbol: &str) -> Option<&'a OutlineNode> {
    outline
        .recipe()?
        .children
        .iter()
        .find(|n| n.keyword == kind && n.symbol.as_deref() == Some(symbol))
}
fn set_child_statement(
    source: &mut String,
    kind: &str,
    symbol: &str,
    key: &str,
    value: &str,
) -> Result<(), String> {
    let outline = Outline::parse(source).map_err(|e| e.to_string())?;
    let node =
        node_for(&outline, kind, symbol).ok_or_else(|| format!("Missing {kind} `{symbol}`"))?;
    set_statement(source, node, key, value)
}
fn set_statement(
    source: &mut String,
    node: &OutlineNode,
    key: &str,
    value: &str,
) -> Result<(), String> {
    if let Some(child) = node.child(key) {
        *source = apply_text_edits(
            source,
            &[TextEdit::replace(
                child.code_range,
                format!("{key} {value};"),
            )],
        )
        .map_err(|e| e.to_string())?;
    } else {
        let inner = node.block_inner_range.ok_or("Declaration has no block")?;
        let inline = !source[inner.start..inner.end].contains('\n');
        let addition = if inline {
            format!(" {key} {value};")
        } else {
            format!("\n{}    {key} {value};", node.indent)
        };
        *source = apply_text_edits(source, &[TextEdit::insert(inner.end, addition)])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn remove_statement(source: &mut String, node: &OutlineNode, key: &str) -> Result<(), String> {
    if let Some(child) = node.child(key) {
        *source = apply_text_edits(source, &[TextEdit::delete(child.range)])
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn scale_inputs(
    recipe: &culinator_core::Recipe,
    formula: &Formula,
    factors: &HashMap<&str, f64>,
    preview: &mut BatchPreview,
) {
    for operation in &recipe.operations {
        for binding in &operation.bindings {
            if binding.role != culinator_core::BindingRole::Input {
                continue;
            }
            let Some(q) = &binding.quantity else {
                continue;
            };
            let Some(factor) = factors.get(binding.resource.as_str()) else {
                if formula
                    .ingredients
                    .iter()
                    .any(|i| i.symbol == binding.resource)
                {
                    preview.blockers.push(format!(
                        "Cannot scale step amount for `{}` without a weighable ingredient total",
                        binding.resource
                    ));
                }
                continue;
            };
            if !matches!(q.dimension, Dimension::Mass | Dimension::Volume) {
                preview.blockers.push(format!(
                    "Step `{}` has an unsupported amount for `{}`",
                    operation.symbol, binding.resource
                ));
                continue;
            }
            let next = Quantity {
                value: q.value * factor,
                ..q.clone()
            };
            if (next.value - q.value).abs() < 0.0005 {
                continue;
            }
            let before = format_quantity(q);
            let after = format_quantity(&next);
            let outline = match Outline::parse(&preview.proposed_source) {
                Ok(o) => o,
                Err(e) => {
                    preview.blockers.push(e.to_string());
                    return;
                }
            };
            let Some(op) = find_recursive(
                outline.recipe().map_or(&[], |r| r.children.as_slice()),
                "operation",
                &operation.symbol,
            ) else {
                preview
                    .blockers
                    .push(format!("Cannot locate step `{}`", operation.symbol));
                continue;
            };
            let Some(input) = op.children.iter().find(|n| {
                n.keyword == "input"
                    && n.symbol.as_deref() == Some(binding.resource.as_str())
                    && n.value_range
                        .is_some_and(|r| preview.proposed_source[r.start..r.end].contains(&before))
            }) else {
                preview.blockers.push(format!(
                    "Cannot locate amount for `{}` in step `{}`",
                    binding.resource, operation.symbol
                ));
                continue;
            };
            let replacement = format!("input {} {after};", binding.resource);
            match apply_text_edits(
                &preview.proposed_source,
                &[TextEdit::replace(input.code_range, replacement)],
            ) {
                Ok(next_source) => {
                    preview.proposed_source = next_source;
                    preview.changes.push(BatchChange {
                        path: format!("operation.{}.input.{}", operation.symbol, binding.resource),
                        before,
                        after,
                    });
                }
                Err(e) => preview.blockers.push(e.to_string()),
            }
        }
    }
}
fn find_recursive<'a>(
    nodes: &'a [OutlineNode],
    kind: &str,
    symbol: &str,
) -> Option<&'a OutlineNode> {
    for node in nodes {
        if node.keyword == kind && node.symbol.as_deref() == Some(symbol) {
            return Some(node);
        }
        if let Some(found) = find_recursive(&node.children, kind, symbol) {
            return Some(found);
        }
    }
    None
}

fn update_yields(
    recipe: &culinator_core::Recipe,
    request: &BatchRequest,
    preview: &mut BatchPreview,
) {
    let count = match &request.constraint {
        FormulaConstraint::Pieces { count } => Some(*count),
        FormulaConstraint::Servings { count, .. } => Some(*count),
        _ => request.piece_count,
    };
    let Some(count) = count else {
        return;
    };
    if !count.is_finite() || count <= 0.0 || (count - count.round()).abs() > 0.0001 {
        preview
            .blockers
            .push("Piece or serving count must be a positive whole number".into());
        return;
    }
    let use_serving = matches!(request.constraint, FormulaConstraint::Servings { .. });
    let yields: Vec<_> = recipe
        .yields
        .iter()
        .filter(|y| matches!(&y.amount, Value::Quantity(q) if q.dimension == Dimension::Count))
        .collect();
    let servings: Vec<_> = recipe
        .servings
        .iter()
        .filter(|s| matches!(&s.amount, Value::Quantity(q) if q.dimension == Dimension::Count))
        .collect();
    let (kind, symbol, amount) = if use_serving && !servings.is_empty() {
        if servings.len() != 1 {
            preview
                .blockers
                .push("Expected exactly one count serving for this batch".into());
            return;
        }
        ("serving", &servings[0].symbol, &servings[0].amount)
    } else {
        if yields.len() != 1 {
            preview
                .blockers
                .push("Expected exactly one count yield for this batch".into());
            return;
        }
        ("yield", &yields[0].symbol, &yields[0].amount)
    };
    let Value::Quantity(old) = amount else {
        return;
    };
    let after = format!("{} {}", number(count), old.unit);
    let before = format_quantity(old);
    if before != after {
        if let Err(error) =
            set_child_statement(&mut preview.proposed_source, kind, symbol, "amount", &after)
        {
            preview.blockers.push(error);
        } else {
            preview.changes.push(BatchChange {
                path: format!("{kind}.{symbol}.amount"),
                before,
                after,
            });
        }
    }
}

fn formula_mass_property(base: &Formula, key: &str, grams: f64) -> Result<Value, String> {
    let mass = Quantity {
        value: grams,
        unit: "g".into(),
        dimension: Dimension::Mass,
    };
    if let Some(Value::Quantity(old)) = base.properties.get(key) {
        if old
            .as_grams()
            .is_some_and(|before| (before - grams).abs() < 0.0005)
        {
            return Ok(Value::Quantity(old.clone()));
        }
        return mass
            .convert_to(&old.unit)
            .map(Value::Quantity)
            .map_err(|error| format!("Cannot update formula `{key}`: {error}"));
    }
    Ok(Value::Quantity(mass))
}

fn patch_formula(
    base: &Formula,
    edited: &Formula,
    request: &BatchRequest,
    result: &FormulaResult,
    preview: &mut BatchPreview,
) -> Result<(), String> {
    let mut props = edited.properties.clone();
    if base.name != edited.name {
        props.insert("name".into(), Value::Text(edited.name.clone()));
    }
    props.insert(
        "target".into(),
        formula_mass_property(base, "target", result.total_mass_grams)?,
    );
    if let Some(count) = request.piece_count.or_else(|| match request.constraint {
        FormulaConstraint::Pieces { count } => Some(count),
        _ => None,
    }) {
        props.insert(
            "pieces".into(),
            Value::Quantity(Quantity {
                value: count,
                unit: "count".into(),
                dimension: Dimension::Count,
            }),
        );
        props.insert(
            "piece_mass".into(),
            formula_mass_property(base, "piece_mass", result.total_mass_grams / count)?,
        );
    } else if let Some(Value::Quantity(q)) = props.get("pieces") {
        if q.value > 0.0 {
            props.insert(
                "piece_mass".into(),
                formula_mass_property(base, "piece_mass", result.total_mass_grams / q.value)?,
            );
        }
    }
    for (key, value) in &props {
        if base.properties.get(key) == Some(value) {
            continue;
        }
        let Some(text) = value_text(value) else {
            return Err(format!("Cannot write formula property `{key}`"));
        };
        set_child_statement(
            &mut preview.proposed_source,
            "formula",
            &base.symbol,
            key,
            &text,
        )?;
        preview.changes.push(BatchChange {
            path: format!("formula.{}.{key}", base.symbol),
            before: base
                .properties
                .get(key)
                .and_then(value_text)
                .unwrap_or_default(),
            after: text,
        });
    }
    for key in base.properties.keys() {
        if props.contains_key(key) {
            continue;
        }
        let outline = Outline::parse(&preview.proposed_source).map_err(|e| e.to_string())?;
        let formula_node =
            node_for(&outline, "formula", &base.symbol).ok_or("Missing formula block")?;
        remove_statement(&mut preview.proposed_source, formula_node, key)?;
        preview.changes.push(BatchChange {
            path: format!("formula.{}.{key}", base.symbol),
            before: base
                .properties
                .get(key)
                .and_then(value_text)
                .unwrap_or_default(),
            after: String::new(),
        });
    }
    let old: BTreeMap<_, _> = base
        .ingredients
        .iter()
        .map(|i| (i.symbol.as_str(), i))
        .collect();
    for item in &edited.ingredients {
        let Some(previous) = old.get(item.symbol.as_str()) else {
            let outline = Outline::parse(&preview.proposed_source).map_err(|e| e.to_string())?;
            let formula_node =
                node_for(&outline, "formula", &base.symbol).ok_or("Missing formula block")?;
            let inner = formula_node
                .block_inner_range
                .ok_or("Formula has no block")?;
            let row = formula_row_text(item)?;
            preview.proposed_source = apply_text_edits(
                &preview.proposed_source,
                &[TextEdit::insert(inner.end, row)],
            )
            .map_err(|e| e.to_string())?;
            preview.changes.push(BatchChange {
                path: format!("formula.{}.ingredient.{}", base.symbol, item.symbol),
                before: String::new(),
                after: "created".into(),
            });
            continue;
        };
        if item.name != previous.name {
            let outline = Outline::parse(&preview.proposed_source).map_err(|e| e.to_string())?;
            let formula_node =
                node_for(&outline, "formula", &base.symbol).ok_or("Missing formula block")?;
            let row = formula_node
                .children
                .iter()
                .find(|n| n.keyword == "ingredient" && n.symbol.as_deref() == Some(&item.symbol))
                .ok_or_else(|| format!("Missing formula row `{}`", item.symbol))?;
            set_statement(
                &mut preview.proposed_source,
                row,
                "name",
                &format!("\"{}\"", item.name.replace('"', "'")),
            )?;
            preview.changes.push(BatchChange {
                path: format!("formula.{}.ingredient.{}.name", base.symbol, item.symbol),
                before: previous.name.clone(),
                after: item.name.clone(),
            });
        }
        let fields = [
            (
                "percentage",
                previous.percentage.map(|n| format!("{}%", number(n))),
                item.percentage.map(|n| format!("{}%", number(n))),
            ),
            (
                "mass",
                previous.mass_grams.map(|n| format!("{} g", number(n))),
                item.mass_grams.map(|n| format!("{} g", number(n))),
            ),
            (
                "stage",
                Some(previous.stage.clone()),
                Some(item.stage.clone()),
            ),
            (
                "reference",
                Some(previous.is_reference.to_string()),
                Some(item.is_reference.to_string()),
            ),
            (
                "flour",
                Some(previous.is_flour.to_string()),
                Some(item.is_flour.to_string()),
            ),
            (
                "water_fraction",
                Some(number(previous.water_fraction)),
                Some(number(item.water_fraction)),
            ),
            (
                "scalable",
                Some(previous.scalable.to_string()),
                Some(item.scalable.to_string()),
            ),
        ];
        for (key, before, after) in fields {
            if before == after {
                continue;
            }
            let Some(after) = after else {
                return Err(format!("Cannot clear `{key}` on `{}`", item.symbol));
            };
            let outline = Outline::parse(&preview.proposed_source).map_err(|e| e.to_string())?;
            let formula_node =
                node_for(&outline, "formula", &base.symbol).ok_or("Missing formula block")?;
            let row = formula_node
                .children
                .iter()
                .find(|n| n.keyword == "ingredient" && n.symbol.as_deref() == Some(&item.symbol))
                .ok_or_else(|| format!("Missing formula row `{}`", item.symbol))?;
            set_statement(&mut preview.proposed_source, row, key, &after)?;
            preview.changes.push(BatchChange {
                path: format!("formula.{}.ingredient.{}.{}", base.symbol, item.symbol, key),
                before: before.unwrap_or_default(),
                after,
            });
        }
        if previous.is_flour && !item.is_flour {
            let outline = Outline::parse(&preview.proposed_source).map_err(|e| e.to_string())?;
            let formula_node =
                node_for(&outline, "formula", &base.symbol).ok_or("Missing formula block")?;
            let row = formula_node
                .children
                .iter()
                .find(|n| n.keyword == "ingredient" && n.symbol.as_deref() == Some(&item.symbol))
                .ok_or_else(|| format!("Missing formula row `{}`", item.symbol))?;
            if let Some(header) = row.header_range {
                let text = &preview.proposed_source[header.start..header.end];
                if text.contains("Flour") {
                    if !text.contains("Flour<BakersPercent>") {
                        return Err(format!(
                            "Change the custom flour type of `{}` in the recipe builder",
                            item.symbol
                        ));
                    }
                    let replacement =
                        text.replace("Flour<BakersPercent>", "Ingredient<BakersPercent>");
                    preview.proposed_source = apply_text_edits(
                        &preview.proposed_source,
                        &[TextEdit::replace(header, replacement)],
                    )
                    .map_err(|e| e.to_string())?;
                    preview.changes.push(BatchChange {
                        path: format!("formula.{}.ingredient.{}.type", base.symbol, item.symbol),
                        before: "Flour<BakersPercent>".into(),
                        after: "Ingredient<BakersPercent>".into(),
                    });
                }
            }
        }
        if item.basis != previous.basis {
            return Err(format!(
                "Change the basis of `{}` in the recipe builder before scaling",
                item.symbol
            ));
        }
        for (key, value) in &item.properties {
            if matches!(
                key.as_str(),
                "percentage"
                    | "baker"
                    | "mass"
                    | "quantity"
                    | "stage"
                    | "reference"
                    | "flour"
                    | "water_fraction"
                    | "scalable"
            ) {
                continue;
            }
            if previous.properties.get(key) == Some(value) {
                continue;
            }
            let Some(text) = value_text(value) else {
                return Err(format!("Cannot write `{key}` on `{}`", item.symbol));
            };
            let outline = Outline::parse(&preview.proposed_source).map_err(|e| e.to_string())?;
            let formula_node =
                node_for(&outline, "formula", &base.symbol).ok_or("Missing formula block")?;
            let row = formula_node
                .children
                .iter()
                .find(|n| n.keyword == "ingredient" && n.symbol.as_deref() == Some(&item.symbol))
                .ok_or_else(|| format!("Missing formula row `{}`", item.symbol))?;
            set_statement(&mut preview.proposed_source, row, key, &text)?;
            preview.changes.push(BatchChange {
                path: format!("formula.{}.ingredient.{}.{}", base.symbol, item.symbol, key),
                before: previous
                    .properties
                    .get(key)
                    .and_then(value_text)
                    .unwrap_or_default(),
                after: text,
            });
        }
        for key in previous.properties.keys() {
            if item.properties.contains_key(key)
                || matches!(
                    key.as_str(),
                    "percentage"
                        | "baker"
                        | "mass"
                        | "quantity"
                        | "stage"
                        | "reference"
                        | "flour"
                        | "water_fraction"
                        | "scalable"
                )
            {
                continue;
            }
            let outline = Outline::parse(&preview.proposed_source).map_err(|e| e.to_string())?;
            let formula_node =
                node_for(&outline, "formula", &base.symbol).ok_or("Missing formula block")?;
            let row = formula_node
                .children
                .iter()
                .find(|n| n.keyword == "ingredient" && n.symbol.as_deref() == Some(&item.symbol))
                .ok_or_else(|| format!("Missing formula row `{}`", item.symbol))?;
            remove_statement(&mut preview.proposed_source, row, key)?;
            preview.changes.push(BatchChange {
                path: format!("formula.{}.ingredient.{}.{}", base.symbol, item.symbol, key),
                before: previous
                    .properties
                    .get(key)
                    .and_then(value_text)
                    .unwrap_or_default(),
                after: String::new(),
            });
        }
    }
    if base.ingredients.iter().any(|old| {
        !edited
            .ingredients
            .iter()
            .any(|item| item.symbol == old.symbol)
    }) {
        return Err(
            "Remove the matching recipe ingredient in the builder before removing its formula row"
                .into(),
        );
    }
    if edited.basis != base.basis {
        return Err("Change the formula basis in the recipe builder before scaling".into());
    }
    Ok(())
}
fn formula_row_text(item: &culinator_core::FormulaIngredient) -> Result<String, String> {
    let mut row = format!("\n        ingredient {} {{", item.symbol);
    match item.basis {
        FormulaBasis::ReferencePercent => {
            if let Some(pct) = item.percentage {
                row.push_str(&format!("\n            percentage {}%;", number(pct)));
            }
        }
        FormulaBasis::PercentOfTotal => {
            if let Some(pct) = item.percentage {
                row.push_str(&format!("\n            percentage {}%;", number(pct)));
            }
            row.push_str("\n            basis total;");
        }
        FormulaBasis::AbsoluteMass => {
            let mass = item
                .mass_grams
                .ok_or_else(|| format!("New row `{}` needs a mass", item.symbol))?;
            row.push_str(&format!(
                "\n            mass {} g;\n            basis absolute;",
                number(mass)
            ));
        }
    }
    if item.is_reference {
        row.push_str("\n            reference true;");
    }
    if item.is_flour {
        row.push_str("\n            flour true;");
    }
    if item.water_fraction > 0.0 {
        row.push_str(&format!(
            "\n            water_fraction {};",
            number(item.water_fraction)
        ));
    }
    if item.stage != "final" {
        row.push_str(&format!("\n            stage {};", item.stage));
    }
    for (key, value) in &item.properties {
        if key == "sourceQuantity"
            || matches!(
                key.as_str(),
                "percentage"
                    | "baker"
                    | "mass"
                    | "quantity"
                    | "basis"
                    | "reference"
                    | "flour"
                    | "water_fraction"
                    | "stage"
            )
        {
            continue;
        }
        let value = value_text(value)
            .ok_or_else(|| format!("Cannot emit property `{key}` on `{}`", item.symbol))?;
        row.push_str(&format!("\n            {key} {value};"));
    }
    row.push_str("\n        }");
    Ok(row)
}
fn value_text(value: &Value) -> Option<String> {
    match value {
        Value::Text(s) => Some(format!("\"{}\"", s.replace('"', "'"))),
        Value::Symbol(s) => Some(s.clone()),
        Value::Number(n) => Some(number(*n)),
        Value::Boolean(b) => Some(b.to_string()),
        Value::Quantity(q) => Some(format_quantity(q)),
        _ => None,
    }
}
fn warn_prose(recipe: &culinator_core::Recipe, preview: &mut BatchPreview) {
    if let Some(Value::Text(description)) = recipe.properties.get("description") {
        if numeric_amount(description) {
            preview
                .warnings
                .push(format!("Review recipe description: {description}"));
        }
    }
    for resource in &recipe.resources {
        for note in &resource.notes {
            if numeric_amount(note) {
                preview.warnings.push(format!(
                    "Review note on ingredient `{}`: {note}",
                    resource.symbol
                ));
            }
        }
    }
    for operation in &recipe.operations {
        for note in &operation.notes {
            if numeric_amount(note) {
                preview.warnings.push(format!(
                    "Review note in step `{}`: {note}",
                    operation.symbol
                ));
            }
        }
    }
}
fn numeric_amount(s: &str) -> bool {
    s.bytes().any(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod test;
