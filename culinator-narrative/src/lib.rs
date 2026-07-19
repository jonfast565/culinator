//! Recipe prose: the single generator for human-readable method steps,
//! ingredient lines, times, and section grouping.
//!
//! Both the static exporters (`culinator-export`) and the desktop reading page
//! (via `culinator-wasm`) render from this, so a sentence reads identically
//! wherever it appears. It used to be duplicated in TypeScript and drifted.

use culinator_core::{
    BindingRole, DonenessKind, HeatLevel, LaborMode, Operation, Quantity, Recipe, Resource,
    ResourceKind, UnitSystem, Value, order,
};
use std::collections::{BTreeMap, BTreeSet};

pub struct RecipeContent {
    /// "10 ingredients · 9 steps · ~2 h 17 min total"
    pub summary: String,
    /// Variant grouping for display; the unlabeled base group comes first.
    pub ingredient_groups: Vec<IngredientGroup>,
    /// Flat ingredient lines, kept for the CSV export and JSON-LD.
    pub ingredients: Vec<String>,
    /// Overall equipment/container list, declaration order.
    pub equipment: Vec<String>,
    /// Method steps grouped into process sections, dependency-sorted.
    pub sections: Vec<Section>,
}

pub struct IngredientGroup {
    pub label: Option<String>,
    pub items: Vec<String>,
}

pub struct Section {
    /// Process symbol this section covers.
    pub process: String,
    /// Humanized process name; `None` when the recipe has ≤1 distinct process.
    pub title: Option<String>,
    /// Parallelism guidance for the section as a whole.
    pub note: Option<String>,
    pub steps: Vec<Step>,
}

pub struct Step {
    /// Operation symbol this step renders.
    pub symbol: String,
    /// Global step number, continuing across sections.
    pub number: usize,
    /// Full instruction prose, equipment woven in, "Meanwhile, " lead-in
    /// when the step can overlap the previous one.
    pub text: String,
    /// "3 min", "8–10 min", "up to 8 h", "about 15 min".
    pub time: Option<String>,
    /// Secondary annotations: labor label, "repeat N×", "makes …".
    pub meta: Vec<String>,
    /// Tool/container/equipment names for this step (JSON-LD `tool`).
    pub tools: Vec<String>,
}

impl Step {
    /// Combined annotation line: "3 min · hands-on · makes chunky mashed avocado".
    pub fn annotation(&self) -> Option<String> {
        let mut parts = Vec::new();
        if let Some(time) = &self.time {
            parts.push(time.clone());
        }
        parts.extend(self.meta.iter().cloned());
        (!parts.is_empty()).then(|| parts.join(" · "))
    }
}

/// Render the recipe's prose using cooking fractions. Use [`extract_with`] to
/// choose decimals instead.
pub fn extract(recipe: &Recipe) -> RecipeContent {
    extract_with(recipe, NumberStyle::Fractions)
}

/// Render with an explicit number style.
pub fn extract_with(recipe: &Recipe, style: NumberStyle) -> RecipeContent {
    let labels = label_map(recipe, style);
    let ingredient_resources: Vec<&Resource> = recipe
        .resources
        .iter()
        .filter(|resource| resource.kind == ResourceKind::Ingredient)
        .collect();
    let ingredients: Vec<String> = ingredient_resources
        .iter()
        .map(|resource| format_ingredient(resource, style))
        .collect();
    let ingredient_groups = group_ingredients(&ingredient_resources, style);
    let equipment: Vec<String> = recipe
        .resources
        .iter()
        .filter(|resource| {
            matches!(
                resource.kind,
                ResourceKind::Equipment | ResourceKind::Container
            )
        })
        .map(|resource| {
            labels
                .get(&resource.symbol)
                .cloned()
                .unwrap_or_else(|| humanize(&resource.symbol))
        })
        .collect();

    let ordered = order::sort_operations_for_display(&recipe.operations);
    let predecessors = order::transitive_predecessors(&recipe.operations);
    let sections = build_sections(recipe, &labels, &ordered, &predecessors, style);
    let summary = summary_line(&ingredients, &ordered);

    RecipeContent {
        summary,
        ingredient_groups,
        ingredients,
        equipment,
        sections,
    }
}

/// Group process-mates together (in order of each process's first appearance in
/// the dependency-sorted sequence) so a recipe whose processes interleave in
/// the schedule still reads as coherent phases.
fn build_sections(
    recipe: &Recipe,
    labels: &BTreeMap<String, String>,
    ordered: &[&Operation],
    predecessors: &BTreeMap<String, BTreeSet<String>>,
    style: NumberStyle,
) -> Vec<Section> {
    let mut groups: Vec<(String, Vec<&Operation>)> = Vec::new();
    for operation in ordered {
        match groups.iter_mut().find(|(p, _)| *p == operation.process) {
            Some((_, ops)) => ops.push(operation),
            None => groups.push((operation.process.clone(), vec![operation])),
        }
    }
    let show_titles = groups.len() > 1;

    let mut sections = Vec::new();
    let mut number = 0;
    let mut previous: Option<&Operation> = None;
    for (index, (process, operations)) in groups.iter().enumerate() {
        let title = show_titles.then(|| title_case(&humanize(process)));
        let mut steps = Vec::new();
        for operation in operations {
            number += 1;
            let mut text = describe_operation(labels, operation, style);
            if overlaps_previous(operation, previous, predecessors) {
                text = format!("Meanwhile, {}", lowercase_first(&text));
            }
            steps.push(Step {
                symbol: operation.symbol.clone(),
                number,
                text,
                time: step_time(operation),
                meta: step_meta(recipe, operation, style),
                tools: step_tools(labels, operation),
            });
            previous = Some(operation);
        }
        let note = show_titles
            .then(|| section_note(&groups, index, predecessors))
            .flatten();
        sections.push(Section {
            process: process.clone(),
            title,
            note,
            steps,
        });
    }
    sections
}

/// A step can start while the previous displayed step runs when that step is
/// unattended and is not one of this step's (transitive) prerequisites.
fn overlaps_previous(
    operation: &Operation,
    previous: Option<&Operation>,
    predecessors: &BTreeMap<String, BTreeSet<String>>,
) -> bool {
    let Some(previous) = previous else {
        return false;
    };
    // An optional previous step may not happen at all, so "Meanwhile" would
    // point at nothing.
    let unattended = !previous.optional
        && matches!(
            previous.labor,
            Some(LaborMode::Passive | LaborMode::Monitor | LaborMode::Automated)
        );
    unattended
        && predecessors
            .get(&operation.symbol)
            .is_none_or(|set| !set.contains(&previous.symbol))
}

fn section_note(
    groups: &[(String, Vec<&Operation>)],
    index: usize,
    predecessors: &BTreeMap<String, BTreeSet<String>>,
) -> Option<String> {
    let (_, operations) = &groups[index];
    let mut notes = Vec::new();
    if index > 0 {
        let (previous_process, previous_ops) = &groups[index - 1];
        let previous_symbols: BTreeSet<&str> = previous_ops
            .iter()
            .map(|operation| operation.symbol.as_str())
            .collect();
        let depends_on_previous = operations.iter().any(|operation| {
            predecessors
                .get(&operation.symbol)
                .is_some_and(|set| set.iter().any(|p| previous_symbols.contains(p.as_str())))
        });
        if !depends_on_previous {
            notes.push(format!(
                "You can work on this while {} is under way.",
                title_case(&humanize(previous_process))
            ));
        }
    }
    if operations.len() > 1 {
        let section_symbols: BTreeSet<&str> = operations
            .iter()
            .map(|operation| operation.symbol.as_str())
            .collect();
        let independent = operations.iter().all(|operation| {
            predecessors
                .get(&operation.symbol)
                .is_none_or(|set| !set.iter().any(|p| section_symbols.contains(p.as_str())))
        });
        if independent {
            notes.push("These steps are independent — do them in any order.".to_owned());
        }
    }
    (!notes.is_empty()).then(|| notes.join(" "))
}

/// Actions where the operation symbol carries the real verb ("mash", "fold").
const GENERIC_ACTIONS: [&str; 7] = ["mix", "heat", "rest", "move", "strain", "coat", "operation"];

/// Loose word equality that tolerates simple plurals, so "pancakes" matches
/// "pancake batter" and "cloves" matches "clove".
fn word_eq(a: &str, b: &str) -> bool {
    let a = a.to_ascii_lowercase();
    let b = b.to_ascii_lowercase();
    a == b
        || format!("{a}s") == b
        || format!("{b}s") == a
        || format!("{a}es") == b
        || format!("{b}es") == a
}

fn any_word_matches(text: &str, word: &str) -> bool {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .any(|candidate| !candidate.is_empty() && word_eq(candidate, word))
}

/// Verbs that lay later ingredients onto the first one ("Top the dish with the
/// panko", "Dip the bread in the custard"), with the preposition each takes.
fn lay_on_preposition(verb: &str) -> Option<&'static str> {
    match verb {
        "top" | "coat" | "cover" | "garnish" | "sprinkle" | "drizzle" | "brush" | "baste"
        | "glaze" | "rub" | "spread" | "dust" | "season" | "oil" => Some("with"),
        "dip" => Some("in"),
        _ => None,
    }
}

fn verb_for(action: &str) -> Option<&'static str> {
    let verb = match action {
        "heat" => "Heat",
        "cook" => "Cook",
        "bake" => "Bake",
        "simmer" => "Simmer",
        "boil" => "Boil",
        "mix" | "combine" => "Combine",
        "blend" => "Blend",
        "whisk" => "Whisk",
        "fold" => "Fold",
        "rest" => "Rest",
        "cool" => "Cool",
        "chill" => "Chill",
        "cut" => "Cut",
        "prepare" => "Prepare",
        "setstate" => "Set up",
        "pit" => "Pit",
        "dice" => "Dice",
        "chop" => "Chop",
        "mince" => "Mince",
        "mash" => "Mash",
        "grate" => "Grate",
        "drain" => "Drain",
        "strain" => "Strain",
        "grease" => "Grease",
        "coat" => "Coat",
        "move" | "transfer" => "Transfer",
        _ => return None,
    };
    Some(verb)
}

fn step_verb(operation: &Operation) -> String {
    let action = operation.declared_type.name.to_ascii_lowercase();
    if GENERIC_ACTIONS.contains(&action.as_str()) {
        return title_case(&humanize(&operation.symbol));
    }
    verb_for(&action)
        .map(str::to_owned)
        .unwrap_or_else(|| title_case(&humanize(&action)))
}

/// Human-readable instruction sentence: verb, inputs with per-step amounts,
/// tools/containers woven in, temperature/heat, doneness, then notes as
/// follow-on sentences.
fn describe_operation(
    labels: &BTreeMap<String, String>,
    operation: &Operation,
    style: NumberStyle,
) -> String {
    let inputs: Vec<(String, bool)> = operation
        .bindings
        .iter()
        .filter(|binding| binding.role == BindingRole::Input)
        .map(|binding| {
            (
                format_binding(labels, binding, style),
                binding.quantity.is_some(),
            )
        })
        .collect();
    let target = operation
        .bindings
        .iter()
        .find(|binding| binding.role == BindingRole::Target)
        .map(|binding| binding_label(labels, &binding.resource));
    let container = operation
        .bindings
        .iter()
        .find(|binding| binding.role == BindingRole::Container)
        .map(|binding| binding_label(labels, &binding.resource));
    let tool = operation
        .bindings
        .iter()
        .find(|binding| binding.role == BindingRole::Tool)
        .map(|binding| binding_label(labels, &binding.resource));

    let mut equipment: Vec<String> = operation
        .bindings
        .iter()
        .filter(|binding| binding.role == BindingRole::Equipment)
        .map(|binding| binding_label(labels, &binding.resource))
        .collect();
    let output_name = operation_output(operation).map(|symbol| humanize(&symbol));
    let action = operation.declared_type.name.to_ascii_lowercase();

    let mut verb = step_verb(operation);
    let mut container = container;
    let mut tool = tool;
    // Trailing state adverbs ("bake_covered", "keep_warm") read best after the
    // object: "Bake the dough covered", "Keep the pancakes warm".
    let mut suffix = String::new();
    // Connector between verb and inputs when nothing better applies.
    let mut connector = " ";
    // Set when the tool turns out to be the verb's real object ("insert_probe").
    let mut tool_is_object = false;

    // Multi-word symbol verbs ("melt_butter", "mix_dry", "warm_up") would
    // repeat or swallow their object; decide what the trailing word is doing
    // before assembling the sentence.
    let verb_snapshot = verb.clone();
    let words: Vec<&str> = verb_snapshot.split(' ').collect();
    if let [head @ .., last] = words.as_slice()
        && !head.is_empty()
    {
        {
            let object = (*last).to_owned();
            let object_lower = object.to_ascii_lowercase();
            let head_verb = head[0].to_ascii_lowercase();
            let in_inputs = inputs
                .iter()
                .any(|(name, _)| any_word_matches(name, &object));
            let in_output = output_name
                .as_deref()
                .is_some_and(|output| any_word_matches(output, &object));
            let in_vessel = container
                .as_deref()
                .is_some_and(|label| any_word_matches(label, &object))
                || equipment
                    .iter()
                    .any(|label| any_word_matches(label, &object));
            let in_tool = tool
                .as_deref()
                .is_some_and(|label| any_word_matches(label, &object));
            // "make_roux" names its product, and dropping it would leave "Make
            // the butter and flour" — creation verbs keep their object.
            let creation_verb = matches!(
                head_verb.as_str(),
                "make" | "build" | "form" | "shape" | "create" | "prepare"
            );
            if object_lower == "again" {
                verb = head.join(" ");
                suffix = " again".to_owned();
            } else if matches!(object_lower.as_str(), "covered" | "uncovered" | "warm") {
                // Checked before the output match: "bake_covered" produces a
                // "covered loaf", but the tail is a state adverb, not the object.
                verb = head.join(" ");
                suffix = format!(" {object_lower}");
            } else if in_inputs || in_vessel || (in_output && !creation_verb) {
                // The trailing word restates something already in the
                // sentence ("cook_pancakes" over pancake batter).
                verb = head.join(" ");
            } else if in_tool && matches!(head_verb.as_str(), "insert" | "stick" | "press") {
                verb = head.join(" ");
                tool_is_object = true;
            } else if matches!(
                object_lower.as_str(),
                "up" | "down"
                    | "in"
                    | "out"
                    | "off"
                    | "on"
                    | "over"
                    | "together"
                    | "back"
                    | "through"
            ) {
                // Phrasal verb ("warm_up", "stir_in") — keep it intact.
            } else if head.contains(&"and") || object_lower.ends_with("ing") {
                // "rinse_and_dry", "finish_baking" — compound verbs take their
                // objects directly.
            } else {
                connector = " with ";
            }
        }
    }

    let mut sentence = if inputs.is_empty() {
        if let Some(target) = &target {
            format!("{verb} the {target}")
        } else if let Some(container) = container.take() {
            // A bare verb reads best acting on its container:
            // "Grease the casserole dish" rather than "In the dish, grease".
            format!("{verb} the {container}")
        } else if action == "coat" && !equipment.is_empty() {
            // "Line the sheet pan" — a bare coat verb acts on its surface.
            format!("{verb} the {}", equipment.remove(0))
        } else {
            verb.clone()
        }
    } else {
        // A leading article when the first input has no per-step amount:
        // "Mash the avocado pulp, lime juice and salt" / "Boil the macaroni".
        let article = if inputs[0].1 { "" } else { "the " };
        let verb_lower = verb.to_ascii_lowercase();
        if tool_is_object {
            // "Insert the probe thermometer into the seasoned chicken."
            let names: Vec<String> = inputs.iter().map(|(name, _)| name.clone()).collect();
            let object = tool.take().unwrap_or_default();
            format!("{verb} the {object} into the {}", join_list(&names))
        } else if inputs.len() >= 2 && lay_on_preposition(&verb_lower).is_some() {
            // "Top the dish with the panko, jack and butter".
            let preposition = lay_on_preposition(&verb_lower).unwrap_or("with");
            let additions: Vec<String> = inputs
                .iter()
                .skip(1)
                .map(|(name, _)| name.clone())
                .collect();
            format!(
                "{verb} {article}{}{suffix} {preposition} the {}",
                inputs[0].0,
                join_list(&additions)
            )
        } else if action == "coat"
            && inputs.len() == 1
            && (container.is_some() || !equipment.is_empty())
        {
            // A coat op's single input is the substance applied; the vessel is
            // the real object: "Grease the loaf pan with the butter".
            let object = container.take().unwrap_or_else(|| equipment.remove(0));
            format!("{verb} the {object} with {article}{}", inputs[0].0)
        } else {
            let names: Vec<String> = inputs.iter().map(|(name, _)| name.clone()).collect();
            format!("{verb}{connector}{article}{}{suffix}", join_list(&names))
        }
    };
    if let Some(tool) = tool.take() {
        // Avoid stacking two "with" clauses: "…with the butter using the brush".
        if sentence.contains(" with ") {
            sentence.push_str(&format!(" using the {tool}"));
        } else {
            sentence.push_str(&format!(" with the {tool}"));
        }
    }
    // Move-like steps take their container as a destination, not a location.
    if matches!(action.as_str(), "move" | "transfer")
        && let Some(container) = container.take()
    {
        {
            let preposition = if verb.eq_ignore_ascii_case("serve") {
                "on"
            } else {
                "into"
            };
            sentence.push_str(&format!(" {preposition} the {container}"));
        }
    }
    for label in &equipment {
        sentence.push_str(&equipment_phrase(label));
    }
    if let Some(temperature) = &operation.target_temperature {
        let preposition = if operation
            .bindings
            .iter()
            .any(|binding| binding.role == BindingRole::Input)
        {
            "at"
        } else {
            "to"
        };
        sentence.push_str(&format!(
            " {preposition} {}",
            display_temperature(temperature, style)
        ));
    }
    if let Some(level) = operation.heat_level {
        sentence.push_str(&format!(" over {} heat", heat_label(level)));
    }
    if !operation.doneness.is_empty() {
        let phrases: Vec<String> = operation
            .doneness
            .iter()
            .map(|cue| match cue.kind {
                DonenessKind::InternalTemp => {
                    let shown = match &cue.value {
                        Value::Quantity(quantity) => display_temperature(quantity, style),
                        value => display_value(value, style),
                    };
                    format!("it reaches {shown} internal")
                }
                _ => display_value(&cue.value, style),
            })
            .collect();
        sentence.push_str(&format!(", until {}", phrases.join(" and ")));
    }
    if let Some(container) = container.take() {
        sentence = format!("In the {container}, {}", lowercase_first(&sentence));
    }
    if operation.optional {
        sentence.push_str(" (optional)");
    }
    sentence.push('.');
    for note in &operation.notes {
        sentence.push_str(&format!(" {}", sentence_case(note)));
    }
    sentence
}

fn step_time(operation: &Operation) -> Option<String> {
    let min = operation.duration_min_seconds.unwrap_or(0);
    let max = operation.duration_max_seconds;
    let formatted = match max {
        Some(max) if max != min => {
            if min == 0 {
                format!("up to {}", format_duration(max))
            } else if min < 60 && max <= 180 {
                // Keep sub-minute ranges in seconds: "45–90 sec", not
                // "45 sec–2 min" (which rounds 90 s up).
                format!("{min}\u{2013}{max} sec")
            } else {
                collapse_range(&format_duration(min), &format_duration(max))
            }
        }
        _ => format_duration(min.max(max.unwrap_or(0))),
    };
    if formatted.is_empty() {
        return None;
    }
    if operation.duration_estimated {
        return Some(format!("about {formatted}"));
    }
    Some(formatted)
}

fn step_meta(recipe: &Recipe, operation: &Operation, style: NumberStyle) -> Vec<String> {
    let mut parts = Vec::new();
    if let Some(labor) = operation.labor {
        parts.push(labor_label(labor).to_owned());
    }
    if let Some(repeat) = operation.repeat
        && repeat > 1
    {
        parts.push(format!("repeat {repeat}×"));
    }
    if let Some(output) = operation_output(operation) {
        let product = humanize(&output);
        let state = recipe
            .resources
            .iter()
            .find(|resource| resource.symbol == output)
            .and_then(|resource| resource.properties.get("state"))
            .and_then(|value| value_text(value, style));
        match state {
            // Skip a state the product name already spells out, so
            // "caramelized_onions" doesn't read "caramelized caramelized onions".
            Some(state)
                if !product
                    .to_ascii_lowercase()
                    .contains(&state.to_ascii_lowercase()) =>
            {
                parts.push(format!("makes {state} {product}"));
            }
            Some(_) | None => parts.push(format!("makes {product}")),
        }
    }
    parts
}

fn step_tools(labels: &BTreeMap<String, String>, operation: &Operation) -> Vec<String> {
    operation
        .bindings
        .iter()
        .filter(|binding| {
            matches!(
                binding.role,
                BindingRole::Tool | BindingRole::Container | BindingRole::Equipment
            )
        })
        .map(|binding| binding_label(labels, &binding.resource))
        .collect()
}

fn summary_line(ingredients: &[String], ordered: &[&Operation]) -> String {
    let total_seconds: u64 = ordered
        .iter()
        .map(|operation| {
            operation.duration_min_seconds.unwrap_or(0)
                * u64::from(operation.repeat.unwrap_or(1).max(1))
        })
        .sum();
    let mut parts = vec![
        format!(
            "{} ingredient{}",
            ingredients.len(),
            if ingredients.len() == 1 { "" } else { "s" }
        ),
        format!(
            "{} step{}",
            ordered.len(),
            if ordered.len() == 1 { "" } else { "s" }
        ),
    ];
    let time = format_duration(total_seconds);
    if !time.is_empty() {
        parts.push(format!("~{time} total"));
    }
    parts.join(" · ")
}

fn group_ingredients(resources: &[&Resource], style: NumberStyle) -> Vec<IngredientGroup> {
    let mut base = Vec::new();
    let mut variants: Vec<(String, Vec<String>)> = Vec::new();
    for resource in resources {
        let line = format_ingredient(resource, style);
        match &resource.variant {
            Some(variant) => match variants.iter_mut().find(|(label, _)| label == variant) {
                Some((_, items)) => items.push(line),
                None => variants.push((variant.clone(), vec![line])),
            },
            None => base.push(line),
        }
    }
    let mut groups = Vec::new();
    if !base.is_empty() {
        groups.push(IngredientGroup {
            label: None,
            items: base,
        });
    }
    for (label, items) in variants {
        groups.push(IngredientGroup {
            label: Some(title_case(&label)),
            items,
        });
    }
    groups
}

/// An ingredient line split into its amount and everything else, so a UI can
/// align quantities in their own column. `flat()` rejoins them for the plain
/// exporters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngredientLine {
    pub symbol: String,
    /// "3 tbsp", "1/2", or empty when the amount is the cook's call.
    pub quantity: String,
    /// Size, state, and name: "large Hass avocados".
    pub description: String,
    /// Trailing modifiers for a UI that renders them apart from the name:
    /// "divided, seeded before dicing, optional".
    pub aside: Option<String>,
    /// The exporters' single-line rendering, where the same modifiers carry
    /// their own punctuation ("kosher salt (divided)").
    pub flat: String,
}

impl IngredientLine {
    pub fn flat(&self) -> String {
        self.flat.clone()
    }
}

/// Per-step amount override: what *this* step takes of a divided ingredient.
pub fn ingredient_line_with(
    resource: &Resource,
    step_quantity: Option<&Quantity>,
    style: NumberStyle,
) -> IngredientLine {
    let name = resource
        .properties
        .get("name")
        .map(|value| display_value(value, style))
        .unwrap_or_else(|| resource.symbol.replace('_', " "));
    let source = step_quantity
        .map(|quantity| Value::Quantity(quantity.clone()))
        .or_else(|| resource.properties.get("quantity").cloned());
    let quantity = match &source {
        // "1 clove garlic clove": drop a unit the name already spells out.
        Some(Value::Quantity(quantity)) if any_word_matches(&name, &quantity.unit) => {
            format_number(quantity.value, style)
        }
        Some(value) => display_value(value, style),
        None => String::new(),
    };
    let has_quantity = !quantity.is_empty();
    let state = resource
        .properties
        .get("state")
        .and_then(|value| value_text(value, style));
    let size = resource.size.as_deref().unwrap_or("");
    let parts = vec![size.to_owned(), state.unwrap_or_default(), name];
    let description = parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    // `flat` keeps the exporters' punctuation; `aside` is the same information
    // as a plain list for a UI that gives it its own column.
    let mut line = description.clone();
    let mut aside: Vec<String> = Vec::new();
    if resource.divided {
        if has_quantity {
            line.push_str(", divided");
        } else {
            line.push_str(" (divided)");
        }
        aside.push("divided".to_owned());
    }
    for note in &resource.notes {
        line.push_str(&format!(", {note}"));
        aside.push(note.clone());
    }
    if resource.to_taste {
        // "Plus more" implies a written base amount; without one the whole
        // quantity is the cook's call.
        line.push_str(if has_quantity {
            ", plus more to taste"
        } else {
            ", to taste"
        });
        aside.push(
            if has_quantity {
                "plus more to taste"
            } else {
                "to taste"
            }
            .to_owned(),
        );
    }
    if resource.optional {
        line.push_str(" (optional)");
        aside.push("optional".to_owned());
    }
    let flat = if quantity.is_empty() {
        line
    } else {
        format!("{quantity} {line}")
    };
    IngredientLine {
        symbol: resource.symbol.clone(),
        quantity,
        description,
        aside: (!aside.is_empty()).then(|| aside.join(", ")),
        flat,
    }
}

fn format_ingredient(resource: &Resource, style: NumberStyle) -> String {
    ingredient_line_with(resource, None, style).flat()
}

fn format_binding(
    labels: &BTreeMap<String, String>,
    binding: &culinator_core::ResourceBinding,
    style: NumberStyle,
) -> String {
    let name = binding_label(labels, &binding.resource);
    match &binding.quantity {
        Some(quantity) => format!("{} {}", display_quantity(quantity, style), name),
        None => name,
    }
}

fn binding_label(labels: &BTreeMap<String, String>, resource: &str) -> String {
    labels
        .get(resource)
        .cloned()
        .unwrap_or_else(|| humanize(resource))
}

fn operation_output(operation: &Operation) -> Option<String> {
    operation
        .bindings
        .iter()
        .find(|binding| binding.role == BindingRole::Output)
        .map(|binding| binding.resource.clone())
}

fn label_map(recipe: &Recipe, style: NumberStyle) -> BTreeMap<String, String> {
    recipe
        .resources
        .iter()
        .map(|resource| {
            let name = resource
                .properties
                .get("name")
                .and_then(|value| value_text(value, style))
                .unwrap_or_else(|| humanize(&resource.symbol));
            (resource.symbol.clone(), name)
        })
        .collect()
}

fn join_list(items: &[String]) -> String {
    match items {
        [] => String::new(),
        [only] => only.clone(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

fn format_duration(seconds: u64) -> String {
    if seconds == 0 {
        return String::new();
    }
    if seconds < 60 {
        return format!("{seconds} sec");
    }
    let total_minutes = (seconds + 30) / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    if hours > 0 && minutes > 0 {
        return format!("{hours} h {minutes} min");
    }
    if hours >= 24 && hours.is_multiple_of(24) {
        return format!("{} day{}", hours / 24, if hours == 24 { "" } else { "s" });
    }
    if hours > 0 {
        return format!("{hours} h");
    }
    format!("{minutes} min")
}

/// Collapse a shared unit so "8 min"–"10 min" reads "8–10 min".
fn collapse_range(min: &str, max: &str) -> String {
    if let (Some((min_value, min_unit)), Some((max_value, max_unit))) =
        (min.split_once(' '), max.split_once(' '))
        && min_unit == max_unit
        && !min_value.contains(' ')
        && !max_value.contains(' ')
    {
        return format!("{min_value}\u{2013}{max_value} {max_unit}");
    }
    format!("{min}\u{2013}{max}")
}

fn labor_label(labor: LaborMode) -> &'static str {
    match labor {
        LaborMode::Active => "hands-on",
        LaborMode::Passive => "unattended",
        LaborMode::Monitor => "keep an eye on it",
        LaborMode::Automated => "hands-off",
    }
}

/// Preposition-aware clause for a bound piece of equipment: cooking happens
/// "in the frying pan", "on the baking sheet", "under the broiler", and only
/// falls back to "using the …" for hand tools.
fn equipment_phrase(label: &str) -> String {
    let lower = label.to_ascii_lowercase();
    if lower.contains("broiler") {
        return format!(" under the {label}");
    }
    if ["rack", "sheet", "board", "griddle", "stone"]
        .iter()
        .any(|word| lower.contains(word))
    {
        return format!(" on the {label}");
    }
    if [
        "pan",
        "skillet",
        "pot",
        "oven",
        "processor",
        "blender",
        "mortar",
        "jar",
        "dish",
        "bowl",
    ]
    .iter()
    .any(|word| lower.contains(word))
    {
        return format!(" in the {label}");
    }
    format!(" using the {label}")
}

fn heat_label(level: HeatLevel) -> &'static str {
    match level {
        HeatLevel::Low => "low",
        HeatLevel::MediumLow => "medium-low",
        HeatLevel::Medium => "medium",
        HeatLevel::MediumHigh => "medium-high",
        HeatLevel::High => "high",
    }
}

fn display_temperature(quantity: &Quantity, style: NumberStyle) -> String {
    let unit = match quantity.unit.to_ascii_lowercase().as_str() {
        "fahrenheit" | "f" => "°F",
        "celsius" | "c" => "°C",
        _ => return display_quantity(quantity, style),
    };
    format!("{} {unit}", quantity.value)
}

fn humanize(symbol: &str) -> String {
    symbol.replace('_', " ")
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn lowercase_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

/// Promote a free-text note to a follow-on sentence: capitalize and make sure
/// it ends with a period.
fn sentence_case(note: &str) -> String {
    let mut sentence = title_case(note.trim());
    if !sentence.ends_with(['.', '!', '?']) {
        sentence.push('.');
    }
    sentence
}

fn value_text(value: &Value, style: NumberStyle) -> Option<String> {
    match value {
        Value::Text(value) | Value::Symbol(value) => Some(value.clone()),
        Value::Quantity(quantity) => Some(display_quantity(quantity, style)),
        _ => None,
    }
}

/// Cook-style number: quarters render as fractions ("1/4", "1 1/2"); anything
/// else keeps its plain decimal form.
/// How amounts are written out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumberStyle {
    /// Cooking style: "1/2 tsp", "1 1/2 cup". What a recipe card uses.
    #[default]
    Fractions,
    /// Plain decimals: "0.5 tsp", "1.5 cup". Easier to scale and to read on a
    /// scale's display.
    Decimals,
}

fn format_number(value: f64, style: NumberStyle) -> String {
    match style {
        NumberStyle::Decimals => format_decimal(value),
        NumberStyle::Fractions => format_fraction(value),
    }
}

/// Trim a float to at most two decimals without a trailing `.0`.
fn format_decimal(value: f64) -> String {
    if (value - value.round()).abs() < 1e-9 {
        return format!("{}", value.round() as i64);
    }
    let rounded = (value * 100.0).round() / 100.0;
    format!("{rounded:.2}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

/// Cooking fractions for the denominators a kitchen actually uses. Anything
/// that is not close to one of them (a converted 236.59 ml, say) falls back to
/// a decimal rather than inventing a fraction nobody can measure.
fn format_fraction(value: f64) -> String {
    const DENOMINATORS: [u32; 4] = [2, 3, 4, 8];
    let whole = value.trunc();
    let remainder = value - whole;
    if remainder.abs() < 1e-9 {
        return format_decimal(value);
    }
    for denominator in DENOMINATORS {
        let numerator = remainder * f64::from(denominator);
        let rounded = numerator.round();
        if (numerator - rounded).abs() < 0.01 && rounded > 0.0 && rounded < f64::from(denominator) {
            let fraction = format!("{}/{denominator}", rounded as u32);
            if whole >= 1.0 {
                return format!("{} {fraction}", whole as i64);
            }
            return fraction;
        }
    }
    format_decimal(value)
}

/// The displayed unit, or `None` when the unit is a bare counter ("2 eggs",
/// not "2 count eggs"). Count nouns pluralize past one: "3 cloves garlic".
fn display_unit(value: f64, unit: &str) -> Option<String> {
    let lower = unit.to_ascii_lowercase();
    match lower.as_str() {
        "count" | "each" | "ea" => None,
        "clove" | "slice" | "stick" | "piece" | "cube" | "can" | "sprig" | "stalk" | "wedge"
        | "sheet" | "scoop" | "handful" | "fillet" | "strip" | "ear" | "head" | "bulb" => {
            Some(if value > 1.0 {
                format!("{lower}s")
            } else {
                lower
            })
        }
        "bunch" => Some(if value > 1.0 {
            "bunches".to_owned()
        } else {
            lower
        }),
        "loaf" => Some(if value > 1.0 {
            "loaves".to_owned()
        } else {
            lower
        }),
        "leaf" => Some(if value > 1.0 {
            "leaves".to_owned()
        } else {
            lower
        }),
        _ => Some(unit.to_owned()),
    }
}

fn display_quantity(quantity: &Quantity, style: NumberStyle) -> String {
    let number = format_number(quantity.value, style);
    match display_unit(quantity.value, &quantity.unit) {
        Some(unit) => format!("{number} {unit}"),
        None => number,
    }
}

pub fn display_value(value: &Value, style: NumberStyle) -> String {
    match value {
        Value::Text(value) | Value::Symbol(value) => value.clone(),
        Value::Number(value) => format_number(*value, style),
        Value::Boolean(value) => value.to_string(),
        Value::Quantity(quantity) => display_quantity(quantity, style),
        Value::List(values) => values
            .iter()
            .map(|value| display_value(value, style))
            .collect::<Vec<_>>()
            .join(", "),
        // Collapse a range's shared unit: "4–5 bananas", "100–200 g", not
        // "4 count–5 count" / "100 g–200 g".
        Value::Range { min, max } => match (min.as_ref(), max.as_ref()) {
            (Value::Quantity(low), Value::Quantity(high))
                if low.unit.eq_ignore_ascii_case(&high.unit) =>
            {
                let numbers = format!(
                    "{}\u{2013}{}",
                    format_number(low.value, style),
                    format_number(high.value, style)
                );
                match display_unit(high.value, &high.unit) {
                    Some(unit) => format!("{numbers} {unit}"),
                    None => numbers,
                }
            }
            _ => format!(
                "{}\u{2013}{}",
                display_value(min, style),
                display_value(max, style)
            ),
        },
        Value::Object(_) => String::new(),
    }
}

#[cfg(test)]
mod test;

/// Restate every quantity in `recipe` in the units `system` would use, leaving
/// the rest of the model untouched.
///
/// Conversion happens once, up front, rather than being threaded through each
/// rendering function — so ingredient lines, per-step amounts, oven
/// temperatures, and internal-temp doneness cues all convert consistently, and
/// presentation code stays unit-agnostic. Durations are stored as seconds
/// rather than quantities and are deliberately unaffected.
///
/// A quantity that cannot be converted (an unknown or count-based unit like
/// `clove`) is left exactly as authored.
pub fn convert_recipe_units(recipe: &Recipe, system: UnitSystem) -> Recipe {
    let mut converted = recipe.clone();
    for resource in &mut converted.resources {
        for value in resource.properties.values_mut() {
            convert_value(value, system);
        }
    }
    for operation in &mut converted.operations {
        for binding in &mut operation.bindings {
            if let Some(quantity) = &binding.quantity {
                binding.quantity = Some(convert_quantity(quantity, system));
            }
        }
        if let Some(temperature) = &operation.target_temperature {
            operation.target_temperature = Some(convert_quantity(temperature, system));
        }
        for cue in &mut operation.doneness {
            convert_value(&mut cue.value, system);
        }
        for value in operation.properties.values_mut() {
            convert_value(value, system);
        }
    }
    converted
}

fn convert_value(value: &mut Value, system: UnitSystem) {
    match value {
        Value::Quantity(quantity) => *quantity = convert_quantity(quantity, system),
        Value::Range { min, max } => {
            convert_value(min, system);
            convert_value(max, system);
        }
        Value::List(items) => {
            for item in items {
                convert_value(item, system);
            }
        }
        _ => {}
    }
}

fn convert_quantity(quantity: &Quantity, system: UnitSystem) -> Quantity {
    match culinator_core::convert_for_system(quantity, system) {
        Ok((value, unit)) => Quantity {
            value,
            unit,
            dimension: quantity.dimension,
        },
        // Count-based and unrecognized units have no metric/US equivalent.
        Err(_) => quantity.clone(),
    }
}

/// What one method section needs on hand before it starts: the ingredients its
/// own steps consume and the vessels they bind.
///
/// Backs the reading page's "mise en place" layout, where these replace the
/// single top-matter lists. A divided ingredient contributes its **per-step**
/// amount here rather than its whole-recipe total — that is the entire point of
/// the layout. Inputs that resolve to a material are earlier steps' products,
/// not things to have on hand, so they are skipped.
#[derive(Debug, Clone, Default)]
pub struct Mise {
    pub ingredients: Vec<IngredientLine>,
    pub equipment: Vec<String>,
}

pub fn section_mise(recipe: &Recipe, process: &str, style: NumberStyle) -> Mise {
    let labels = label_map(recipe, style);
    let by_symbol: BTreeMap<&str, &Resource> = recipe
        .resources
        .iter()
        .map(|resource| (resource.symbol.as_str(), resource))
        .collect();
    let ordered = order::sort_operations_for_display(&recipe.operations);
    let mut mise = Mise::default();
    for operation in ordered
        .iter()
        .filter(|operation| operation.process == process)
    {
        for binding in &operation.bindings {
            match binding.role {
                BindingRole::Input => {
                    let Some(resource) = by_symbol.get(binding.resource.as_str()) else {
                        continue;
                    };
                    if resource.kind != ResourceKind::Ingredient {
                        continue;
                    }
                    let line = ingredient_line_with(resource, binding.quantity.as_ref(), style);
                    // A divided ingredient legitimately appears more than once
                    // when steps take different amounts; identical lines do not.
                    if !mise.ingredients.contains(&line) {
                        mise.ingredients.push(line);
                    }
                }
                BindingRole::Tool
                | BindingRole::Container
                | BindingRole::Equipment
                | BindingRole::Target => {
                    let label = binding_label(&labels, &binding.resource);
                    if !mise.equipment.contains(&label) {
                        mise.equipment.push(label);
                    }
                }
                _ => {}
            }
        }
    }
    mise
}
