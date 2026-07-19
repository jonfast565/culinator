use super::*;
use culinator_core::{
    BindingRole, Dependency, DependencyKind, Recipe, Resource, ResourceBinding, ResourceKind,
    TypeRef, Value,
};
use std::collections::BTreeMap;
use uuid::Uuid;

fn resource(symbol: &str, kind: ResourceKind, name: &str) -> Resource {
    Resource {
        id: Uuid::new_v4(),
        symbol: symbol.into(),
        declared_type: TypeRef::named("Resource"),
        kind,
        optional: false,
        divided: false,
        substitutes: vec![],
        to_taste: false,
        size: None,
        variant: None,
        notes: vec![],
        properties: BTreeMap::from([("name".into(), Value::Text(name.into()))]),
        span: None,
    }
}

fn binding(symbol: &str, role: BindingRole) -> ResourceBinding {
    ResourceBinding {
        resource: symbol.into(),
        role,
        quantity: None,
        exclusive: false,
    }
}

fn operation(symbol: &str, action: &str, process: &str, after: &[&str]) -> Operation {
    Operation {
        id: Uuid::new_v4(),
        symbol: symbol.into(),
        declared_type: TypeRef::named(action),
        process: process.into(),
        labor: None,
        duration_min_seconds: None,
        duration_max_seconds: None,
        duration_estimated: false,
        target_temperature: None,
        heat_level: None,
        doneness: vec![],
        optional: false,
        repeat: None,
        notes: vec![],
        dependencies: after
            .iter()
            .map(|predecessor| Dependency {
                predecessor: (*predecessor).into(),
                kind: DependencyKind::FinishStart,
                minimum_lag_seconds: None,
                maximum_lag_seconds: None,
                optional: false,
            })
            .collect(),
        bindings: vec![],
        requirements: vec![],
        effects: vec![],
        properties: BTreeMap::new(),
        span: None,
    }
}

fn recipe(resources: Vec<Resource>, operations: Vec<Operation>) -> Recipe {
    Recipe {
        id: Uuid::new_v4(),
        book_id: None,
        symbol: "test".into(),
        declared_type: TypeRef::named("Recipe"),
        title: "Test".into(),
        protocol_version: "0.3".into(),
        types: vec![],
        resources,
        processes: vec![],
        operations,
        servings: vec![],
        formulas: vec![],
        yields: vec![],
        properties: BTreeMap::new(),
    }
}

fn all_steps(content: &RecipeContent) -> Vec<&Step> {
    content
        .sections
        .iter()
        .flat_map(|section| section.steps.iter())
        .collect()
}

#[test]
fn empty_recipe_has_empty_content() {
    let content = extract(&recipe(vec![], vec![]));
    assert!(content.ingredients.is_empty());
    assert!(content.equipment.is_empty());
    assert!(content.sections.is_empty());
}

#[test]
fn ingredient_line_includes_size_state_notes_and_to_taste() {
    let mut props = BTreeMap::new();
    props.insert("name".into(), Value::Text("Hass avocados".into()));
    props.insert("quantity".into(), Value::Text("3 count".into()));
    props.insert("state".into(), Value::Symbol("ripe".into()));
    let resource = Resource {
        id: Uuid::new_v4(),
        symbol: "avocados".into(),
        declared_type: TypeRef::named("Ingredient"),
        kind: ResourceKind::Ingredient,
        optional: false,
        divided: false,
        substitutes: vec![],
        to_taste: false,
        size: Some("medium".into()),
        variant: None,
        notes: vec!["seeded before dicing".into()],
        properties: props,
        span: None,
    };
    let line = format_ingredient(&resource, NumberStyle::Fractions);
    assert!(line.contains("3 count"));
    assert!(line.contains("medium"));
    assert!(line.contains("ripe"));
    assert!(line.contains("Hass avocados"));
    assert!(line.contains("seeded before dicing"));
}

#[test]
fn step_uses_symbol_verb_for_generic_action_and_promotes_notes_to_sentences() {
    let mut mash = operation("mash", "Mix", "mixing", &[]);
    mash.notes = vec!["leave some larger chunks for texture".into()];
    mash.bindings = vec![binding("avocado_pulp", BindingRole::Input)];
    let content = extract(&recipe(
        vec![resource(
            "avocado_pulp",
            ResourceKind::Material,
            "avocado pulp",
        )],
        vec![mash],
    ));
    let step = &content.sections[0].steps[0];
    assert_eq!(
        step.text,
        "Mash the avocado pulp. Leave some larger chunks for texture."
    );
}

#[test]
fn container_and_tool_are_woven_into_the_sentence() {
    let mut mash = operation("mash", "Mix", "mixing", &[]);
    mash.bindings = vec![
        binding("avocado_pulp", BindingRole::Input),
        binding("masher", BindingRole::Tool),
        binding("bowl", BindingRole::Container),
    ];
    let content = extract(&recipe(
        vec![
            resource("avocado_pulp", ResourceKind::Material, "avocado pulp"),
            resource("masher", ResourceKind::Equipment, "potato masher"),
            resource("bowl", ResourceKind::Container, "large mixing bowl"),
        ],
        vec![mash],
    ));
    let step = &content.sections[0].steps[0];
    assert_eq!(
        step.text,
        "In the large mixing bowl, mash the avocado pulp with the potato masher."
    );
    assert_eq!(step.tools, vec!["potato masher", "large mixing bowl"]);
}

#[test]
fn equipment_and_containers_are_listed() {
    let content = extract(&recipe(
        vec![
            resource("flour", ResourceKind::Ingredient, "flour"),
            resource("blender", ResourceKind::Equipment, "blender"),
            resource("pan", ResourceKind::Container, "nonstick pan"),
            resource("batter", ResourceKind::Material, "batter"),
        ],
        vec![],
    ));
    assert_eq!(content.equipment, vec!["blender", "nonstick pan"]);
    assert_eq!(content.ingredients, vec!["flour"]);
}

#[test]
fn sections_follow_processes_and_steps_sort_by_dependencies() {
    let ops = vec![
        operation("bake", "heat", "baking", &["combine"]),
        operation("boil", "heat", "pasta", &[]),
        operation("combine", "Mix", "baking", &["boil"]),
    ];
    let content = extract(&recipe(vec![], ops));
    let titles: Vec<Option<String>> = content
        .sections
        .iter()
        .map(|section| section.title.clone())
        .collect();
    assert_eq!(
        titles,
        vec![Some("Pasta".to_owned()), Some("Baking".to_owned())]
    );
    let numbers: Vec<(usize, String)> = all_steps(&content)
        .iter()
        .map(|step| (step.number, step.text.clone()))
        .collect();
    assert_eq!(numbers[0].0, 1);
    assert!(numbers[0].1.starts_with("Boil"));
    assert!(numbers[1].1.starts_with("Combine"));
    assert!(numbers[2].1.starts_with("Bake"));
}

#[test]
fn single_process_hides_section_title() {
    let content = extract(&recipe(vec![], vec![operation("mash", "Mix", "main", &[])]));
    assert_eq!(content.sections.len(), 1);
    assert!(content.sections[0].title.is_none());
}

#[test]
fn unattended_previous_step_gets_meanwhile_lead_in() {
    let mut preheat = operation("preheat", "heat", "setup", &[]);
    preheat.labor = Some(LaborMode::Automated);
    let grease = operation("grease", "coat", "setup", &[]);
    let mut bake = operation("bake", "heat", "setup", &["preheat", "grease"]);
    bake.labor = Some(LaborMode::Passive);
    let content = extract(&recipe(vec![], vec![preheat, grease, bake]));
    let steps = all_steps(&content);
    assert!(steps[1].text.starts_with("Meanwhile, grease"));
    // `bake` depends on both earlier steps, so no lead-in.
    assert!(steps[2].text.starts_with("Bake"));
}

#[test]
fn independent_section_and_overlapping_section_get_notes() {
    let ops = vec![
        operation("dice_onion", "dice", "prep", &[]),
        operation("dice_tomato", "dice", "prep", &[]),
        operation("preheat", "heat", "setup", &[]),
        operation("fold", "Mix", "mixing", &["dice_onion", "dice_tomato"]),
    ];
    let content = extract(&recipe(vec![], ops));
    assert_eq!(
        content.sections[0].note.as_deref(),
        Some("These steps are independent — do them in any order.")
    );
    // Setup does not depend on prep, so it can run alongside it.
    assert_eq!(
        content.sections[1].note.as_deref(),
        Some("You can work on this while Prep is under way.")
    );
    // Mixing depends on prep steps but not on setup's preheat.
    assert_eq!(
        content.sections[2].note.as_deref(),
        Some("You can work on this while Setup is under way.")
    );
}

#[test]
fn step_time_formats_ranges_ceilings_and_estimates() {
    let mut fixed = operation("chill", "rest", "main", &[]);
    fixed.duration_min_seconds = Some(3600);
    fixed.duration_max_seconds = Some(3600);
    assert_eq!(step_time(&fixed).as_deref(), Some("1 h"));

    let mut range = operation("boil", "heat", "main", &[]);
    range.duration_min_seconds = Some(480);
    range.duration_max_seconds = Some(600);
    assert_eq!(step_time(&range).as_deref(), Some("8\u{2013}10 min"));

    let mut seconds_range = operation("cook", "heat", "main", &[]);
    seconds_range.duration_min_seconds = Some(45);
    seconds_range.duration_max_seconds = Some(90);
    assert_eq!(
        step_time(&seconds_range).as_deref(),
        Some("45\u{2013}90 sec")
    );

    let mut ceiling = operation("store", "rest", "main", &[]);
    ceiling.duration_max_seconds = Some(8 * 3600);
    assert_eq!(step_time(&ceiling).as_deref(), Some("up to 8 h"));

    let mut estimated = operation("preheat", "heat", "main", &[]);
    estimated.duration_min_seconds = Some(900);
    estimated.duration_estimated = true;
    assert_eq!(step_time(&estimated).as_deref(), Some("about 15 min"));

    let untimed = operation("serve", "move", "main", &[]);
    assert_eq!(step_time(&untimed), None);
}

#[test]
fn meta_includes_labor_repeat_and_product() {
    let mut cook = operation("cook", "heat", "cooking", &[]);
    cook.labor = Some(LaborMode::Active);
    cook.repeat = Some(11);
    cook.bindings = vec![binding("crepes", BindingRole::Output)];
    let mut crepes = resource("crepes", ResourceKind::Material, "crepes");
    crepes
        .properties
        .insert("state".into(), Value::Symbol("golden".into()));
    let content = extract(&recipe(vec![crepes], vec![cook]));
    assert_eq!(
        content.sections[0].steps[0].meta,
        vec!["hands-on", "repeat 11×", "makes golden crepes"]
    );
}

#[test]
fn variant_ingredients_group_under_labels() {
    let mut sugar = resource("sugar", ResourceKind::Ingredient, "sugar");
    sugar.variant = Some("sweet".into());
    let mut herbs = resource("herbs", ResourceKind::Ingredient, "fresh herbs");
    herbs.variant = Some("savory".into());
    let flour = resource("flour", ResourceKind::Ingredient, "flour");
    let content = extract(&recipe(vec![sugar, herbs, flour], vec![]));
    let labels: Vec<Option<String>> = content
        .ingredient_groups
        .iter()
        .map(|group| group.label.clone())
        .collect();
    assert_eq!(
        labels,
        vec![None, Some("Sweet".to_owned()), Some("Savory".to_owned())]
    );
    assert_eq!(content.ingredient_groups[0].items, vec!["flour"]);
    assert_eq!(content.ingredients.len(), 3);
}

fn quantity(value: f64, unit: &str) -> Value {
    Value::Quantity(Quantity {
        value,
        unit: unit.into(),
        dimension: culinator_core::Dimension::from_unit(unit),
    })
}

#[test]
fn quantities_render_cook_style() {
    // Bare counters drop their unit; quarters read as fractions; count nouns
    // pluralize; ranges collapse a shared unit.
    assert_eq!(
        display_value(&quantity(2.0, "count"), NumberStyle::Fractions),
        "2"
    );
    assert_eq!(
        display_value(&quantity(0.5, "tsp"), NumberStyle::Fractions),
        "1/2 tsp"
    );
    assert_eq!(
        display_value(&quantity(1.5, "cup"), NumberStyle::Fractions),
        "1 1/2 cup"
    );
    assert_eq!(
        display_value(&quantity(3.0, "clove"), NumberStyle::Fractions),
        "3 cloves"
    );
    assert_eq!(
        display_value(&quantity(1.0, "clove"), NumberStyle::Fractions),
        "1 clove"
    );
    assert_eq!(
        display_value(
            &Value::Range {
                min: Box::new(quantity(4.0, "count")),
                max: Box::new(quantity(5.0, "count")),
            },
            NumberStyle::Fractions
        ),
        "4\u{2013}5"
    );
    assert_eq!(
        display_value(
            &Value::Range {
                min: Box::new(quantity(100.0, "g")),
                max: Box::new(quantity(200.0, "g")),
            },
            NumberStyle::Fractions
        ),
        "100\u{2013}200 g"
    );
}

#[test]
fn ingredient_drops_unit_already_in_name_and_bare_to_taste() {
    let mut garlic = resource("garlic", ResourceKind::Ingredient, "garlic clove");
    garlic
        .properties
        .insert("quantity".into(), quantity(1.0, "clove"));
    assert_eq!(
        format_ingredient(&garlic, NumberStyle::Fractions),
        "1 garlic clove"
    );

    let mut salt = resource("salt", ResourceKind::Ingredient, "salt");
    salt.to_taste = true;
    assert_eq!(
        format_ingredient(&salt, NumberStyle::Fractions),
        "salt, to taste"
    );
    salt.properties
        .insert("quantity".into(), quantity(0.5, "tsp"));
    assert_eq!(
        format_ingredient(&salt, NumberStyle::Fractions),
        "1/2 tsp salt, plus more to taste"
    );
}

#[test]
fn trailing_verb_word_matching_tolerates_plurals_and_output_names() {
    // "cook_pancakes" over pancake batter: plural trailing word matches.
    let mut cook = operation("cook_pancakes", "heat", "cooking", &[]);
    cook.bindings = vec![binding("batter", BindingRole::Input)];
    let content = extract(&recipe(
        vec![resource("batter", ResourceKind::Material, "pancake batter")],
        vec![cook],
    ));
    assert_eq!(all_steps(&content)[0].text, "Cook the pancake batter.");

    // "mix_dry" makes the dry mixture: the output supplies the match.
    let mut mix = operation("mix_dry", "Mix", "batter", &[]);
    mix.bindings = vec![
        binding("flour", BindingRole::Input),
        binding("dry_mixture", BindingRole::Output),
    ];
    let content = extract(&recipe(
        vec![resource("flour", ResourceKind::Ingredient, "flour")],
        vec![mix],
    ));
    assert_eq!(all_steps(&content)[0].text, "Mix the flour.");

    // Creation verbs keep their object: "Make roux with the butter".
    let mut roux = operation("make_roux", "heat", "sauce", &[]);
    roux.bindings = vec![
        binding("butter", BindingRole::Input),
        binding("roux", BindingRole::Output),
    ];
    let content = extract(&recipe(
        vec![resource("butter", ResourceKind::Ingredient, "butter")],
        vec![roux],
    ));
    assert_eq!(all_steps(&content)[0].text, "Make roux with the butter.");
}

#[test]
fn phrasal_compound_and_state_tail_verbs_read_naturally() {
    let mut warm = operation("warm_up", "heat", "main", &[]);
    warm.bindings = vec![binding("base", BindingRole::Input)];
    let mut rinse = operation("rinse_and_dry", "strain", "main", &["warm_up"]);
    rinse.bindings = vec![binding("base", BindingRole::Input)];
    let mut bake = operation("bake_covered", "heat", "main", &["rinse_and_dry"]);
    bake.bindings = vec![
        binding("base", BindingRole::Input),
        binding("covered_loaf", BindingRole::Output),
    ];
    let mut keep = operation("keep_warm", "heat", "main", &["bake_covered"]);
    keep.bindings = vec![binding("base", BindingRole::Input)];
    let content = extract(&recipe(
        vec![resource("base", ResourceKind::Material, "dough")],
        vec![warm, rinse, bake, keep],
    ));
    let texts: Vec<&str> = all_steps(&content)
        .iter()
        .map(|step| step.text.as_str())
        .collect();
    assert_eq!(
        texts,
        vec![
            "Warm up the dough.",
            "Rinse and dry the dough.",
            "Bake the dough covered.",
            "Keep the dough warm.",
        ]
    );
}

#[test]
fn coat_ops_take_their_vessel_as_the_object() {
    let mut grease = operation("grease", "coat", "setup", &[]);
    grease.bindings = vec![
        binding("butter", BindingRole::Input),
        binding("pan", BindingRole::Container),
    ];
    let content = extract(&recipe(
        vec![
            resource("butter", ResourceKind::Ingredient, "butter"),
            resource("pan", ResourceKind::Container, "loaf pan"),
        ],
        vec![grease],
    ));
    assert_eq!(
        all_steps(&content)[0].text,
        "Grease the loaf pan with the butter."
    );
}

#[test]
fn lay_on_verbs_and_tool_clause_avoid_stacked_withs() {
    let mut glaze = operation("glaze", "coat", "finish", &[]);
    glaze.bindings = vec![
        binding("loaves", BindingRole::Input),
        binding("sauce", BindingRole::Input),
    ];
    let mut dip = operation("dip", "coat", "finish", &["glaze"]);
    dip.bindings = vec![
        binding("bread", BindingRole::Input),
        binding("custard", BindingRole::Input),
    ];
    let mut brush = operation("brush_loaves", "coat", "finish", &["dip"]);
    brush.bindings = vec![
        binding("loaves", BindingRole::Input),
        binding("sauce", BindingRole::Input),
        binding("pastry_brush", BindingRole::Tool),
    ];
    let content = extract(&recipe(
        vec![
            resource("loaves", ResourceKind::Material, "loaves"),
            resource("sauce", ResourceKind::Material, "sauce"),
            resource("bread", ResourceKind::Ingredient, "bread"),
            resource("custard", ResourceKind::Material, "custard"),
            resource("pastry_brush", ResourceKind::Equipment, "pastry brush"),
        ],
        vec![glaze, dip, brush],
    ));
    let texts: Vec<&str> = all_steps(&content)
        .iter()
        .map(|step| step.text.as_str())
        .collect();
    assert_eq!(
        texts,
        vec![
            "Glaze the loaves with the sauce.",
            "Dip the bread in the custard.",
            "Brush the loaves with the sauce using the pastry brush.",
        ]
    );
}

#[test]
fn equipment_gets_a_natural_preposition() {
    assert_eq!(equipment_phrase("frying pan"), " in the frying pan");
    assert_eq!(equipment_phrase("baking sheet"), " on the baking sheet");
    assert_eq!(equipment_phrase("sheet pan"), " on the sheet pan");
    assert_eq!(equipment_phrase("broiler"), " under the broiler");
    assert_eq!(equipment_phrase("rubber tongs"), " using the rubber tongs");
}

#[test]
fn internal_temp_doneness_formats_degrees() {
    let mut roast = operation("roast", "heat", "main", &[]);
    roast.bindings = vec![binding("chicken", BindingRole::Input)];
    roast.doneness = vec![culinator_core::DonenessCue {
        kind: DonenessKind::InternalTemp,
        value: quantity(68.0, "celsius"),
    }];
    let content = extract(&recipe(
        vec![resource("chicken", ResourceKind::Ingredient, "chicken")],
        vec![roast],
    ));
    assert_eq!(
        all_steps(&content)[0].text,
        "Roast the chicken, until it reaches 68 °C internal."
    );
}

#[test]
fn optional_previous_step_suppresses_meanwhile() {
    let mut hold = operation("hold", "rest", "main", &[]);
    hold.labor = Some(LaborMode::Passive);
    hold.optional = true;
    let preheat = operation("preheat", "heat", "main", &[]);
    let content = extract(&recipe(vec![], vec![hold, preheat]));
    assert!(all_steps(&content)[1].text.starts_with("Preheat"));
}

#[test]
fn meta_skips_state_already_in_product_name() {
    let mut caramelize = operation("caramelize", "heat", "onions", &[]);
    caramelize.bindings = vec![binding("caramelized_onions", BindingRole::Output)];
    let mut onions = resource("caramelized_onions", ResourceKind::Material, "onions");
    onions
        .properties
        .insert("state".into(), Value::Symbol("caramelized".into()));
    let content = extract(&recipe(vec![onions], vec![caramelize]));
    assert_eq!(
        content.sections[0].steps[0].meta,
        vec!["makes caramelized onions"]
    );
}

#[test]
fn summary_counts_ingredients_steps_and_time() {
    let mut mash = operation("mash", "Mix", "main", &[]);
    mash.duration_min_seconds = Some(180);
    let mut rest = operation("rest_dip", "rest", "main", &["mash"]);
    rest.duration_min_seconds = Some(7200);
    let content = extract(&recipe(
        vec![resource("avocados", ResourceKind::Ingredient, "avocados")],
        vec![mash, rest],
    ));
    assert_eq!(content.summary, "1 ingredient · 2 steps · ~2 h 3 min total");
}

#[test]
fn number_style_switches_between_fractions_and_decimals() {
    let mut resource = resource("milk", ResourceKind::Ingredient, "milk");
    resource.properties.insert(
        "quantity".into(),
        Value::Quantity(Quantity {
            value: 0.5,
            unit: "cup".into(),
            dimension: culinator_core::Dimension::Volume,
        }),
    );
    let recipe = recipe(vec![resource], vec![]);

    let fractions = extract_with(&recipe, NumberStyle::Fractions);
    assert_eq!(fractions.ingredients, vec!["1/2 cup milk"]);

    let decimals = extract_with(&recipe, NumberStyle::Decimals);
    assert_eq!(decimals.ingredients, vec!["0.5 cup milk"]);

    // The default is unchanged, and the guard restored the thread's style.
    assert_eq!(extract(&recipe).ingredients, vec!["1/2 cup milk"]);
}

#[test]
fn fractions_cover_kitchen_denominators_and_fall_back_to_decimals() {
    assert_eq!(format_fraction(0.5), "1/2");
    assert_eq!(format_fraction(0.25), "1/4");
    assert_eq!(format_fraction(1.5), "1 1/2");
    // Thirds are common in recipes and used to render as 0.33333.
    assert_eq!(format_fraction(1.0 / 3.0), "1/3");
    assert_eq!(format_fraction(2.0 / 3.0), "2/3");
    assert_eq!(format_fraction(0.125), "1/8");
    assert_eq!(format_fraction(3.0), "3");
    // A converted metric amount has no sensible fraction.
    assert_eq!(format_fraction(236.59), "236.59");
}
