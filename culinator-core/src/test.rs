use super::*;
use std::collections::BTreeMap;

fn ingredient(
    symbol: &str,
    pct: f64,
    reference: bool,
    flour: bool,
    water_fraction: f64,
) -> FormulaIngredient {
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
        water_fraction,
        scalable: true,
        properties: BTreeMap::new(),
    }
}

fn ingredient_with_props(
    symbol: &str,
    pct: f64,
    reference: bool,
    flour: bool,
    water_fraction: f64,
    stage: &str,
    properties: BTreeMap<Symbol, Value>,
) -> FormulaIngredient {
    FormulaIngredient {
        id: Uuid::new_v4(),
        symbol: symbol.into(),
        name: symbol.into(),
        stage: stage.into(),
        basis: FormulaBasis::ReferencePercent,
        percentage: Some(pct),
        mass_grams: None,
        is_reference: reference,
        is_flour: flour,
        water_fraction,
        scalable: true,
        properties,
    }
}

#[test]
fn named_type_has_no_arguments() {
    assert!(TypeRef::named("Mass").arguments.is_empty());
}

#[test]
fn empty_recipe_book_has_expected_type() {
    let book = RecipeBook::empty("bread", "Bread", "0.3");
    assert_eq!(book.declared_type.name, "RecipeBook");
    assert!(book.recipes.is_empty());
}

#[test]
fn solves_reference_formula() {
    let formula = Formula {
        id: Uuid::new_v4(),
        recipe_id: None,
        symbol: "bread".into(),
        name: "Bread".into(),
        basis: FormulaBasis::ReferencePercent,
        ingredients: vec![
            ingredient("flour", 100.0, true, true, 0.0),
            ingredient("water", 70.0, false, false, 1.0),
            ingredient("salt", 2.0, false, false, 0.0),
        ],
        properties: BTreeMap::new(),
    };
    let result = formula
        .solve_for_target_mass(1720.0)
        .expect("formula solves");
    assert!((result.reference_mass_grams - 1000.0).abs() < 0.001);
    assert!((result.hydration_percent - 70.0).abs() < 0.001);
}

#[test]
fn converts_weights_to_total_percentages() {
    let mut a = ingredient("a", 100.0, true, false, 0.0);
    a.mass_grams = Some(600.0);
    let mut b = ingredient("b", 50.0, false, false, 0.0);
    b.mass_grams = Some(400.0);
    let formula = Formula {
        id: Uuid::new_v4(),
        recipe_id: None,
        symbol: "x".into(),
        name: "X".into(),
        basis: FormulaBasis::ReferencePercent,
        ingredients: vec![a, b],
        properties: BTreeMap::new(),
    };
    let result = formula
        .weights_to_percentages(PercentageView::Total)
        .expect("conversion succeeds");
    assert!((result.lines[0].percentage.expect("percentage") - 60.0).abs() < 0.001);
}

#[test]
fn poolish_build_stage_emits_staged_ingredients() {
    let preferment = Preferment {
        kind: PrefermentKind::Poolish,
        flour_pct: 20.0,
        hydration: 100.0,
        inoculation: 0.1,
        stage: "poolish".into(),
    };
    let stage = preferment.build_stage();
    assert_eq!(stage.len(), 3);
    assert_eq!(stage[0].stage, "poolish");
    assert_eq!(stage[0].symbol, "poolish_flour");
    assert!(stage[0].is_flour);
    assert!(!stage[0].is_reference);
    assert!((stage[0].percentage.expect("pct") - 20.0).abs() < 0.001);
    assert_eq!(stage[1].symbol, "poolish_water");
    assert!((stage[1].percentage.expect("pct") - 20.0).abs() < 0.001);
    assert_eq!(stage[2].symbol, "poolish_culture");
    assert!((stage[2].percentage.expect("pct") - 0.02).abs() < 0.001);
}

#[test]
fn soaker_build_stage_omits_culture() {
    let preferment = Preferment {
        kind: PrefermentKind::Soaker,
        flour_pct: 15.0,
        hydration: 80.0,
        inoculation: 5.0,
        stage: "soaker".into(),
    };
    let stage = preferment.build_stage();
    assert_eq!(stage.len(), 2);
    assert!(!stage.iter().any(|item| item.symbol.ends_with("_culture")));
}

#[test]
fn preferment_kind_parse_is_case_insensitive() {
    assert!(PrefermentKind::parse("poolish").is_ok());
    assert_eq!(
        PrefermentKind::parse("old_dough").expect("parse"),
        PrefermentKind::OldDough
    );
    assert_eq!(
        PrefermentKind::parse("scald").expect("parse"),
        PrefermentKind::Scald
    );
    assert!(PrefermentKind::parse("unknown_kind").is_err());
}

#[test]
fn solves_formula_with_preferment_stage_and_role_metrics() {
    let mut salt_props = BTreeMap::new();
    salt_props.insert("is_salt".into(), Value::Boolean(true));
    let mut fat_props = BTreeMap::new();
    fat_props.insert("role".into(), Value::Text("fat".into()));
    let poolish = Preferment {
        kind: PrefermentKind::Poolish,
        flour_pct: 20.0,
        hydration: 100.0,
        inoculation: 0.0,
        stage: "poolish".into(),
    };
    let mut poolish_stage = poolish.build_stage();
    poolish_stage[0].is_reference = true;
    let formula = Formula {
        id: Uuid::new_v4(),
        recipe_id: None,
        symbol: "bread".into(),
        name: "Bread".into(),
        basis: FormulaBasis::ReferencePercent,
        ingredients: [
            poolish_stage,
            vec![
                ingredient("final_flour", 80.0, true, true, 0.0),
                ingredient("water", 50.0, false, false, 1.0),
                ingredient_with_props("salt", 2.0, false, false, 0.0, "final", salt_props),
                ingredient_with_props("butter", 6.0, false, false, 0.0, "final", fat_props),
            ],
        ]
        .concat(),
        properties: BTreeMap::new(),
    };
    let result = formula
        .solve_for_target_mass(1580.0)
        .expect("formula solves");
    assert!((result.prefermented_flour_percent - 20.0).abs() < 0.1);
    assert!((result.hydration_percent - 70.0).abs() < 0.1);
    assert!((result.salt_percent - 2.0).abs() < 0.1);
    assert!((result.fat_percent - 6.0).abs() < 0.1);
    assert!(result.effective_hydration_percent >= result.hydration_percent);
}

#[test]
fn desired_dough_temperature_without_preferment() {
    let water = desired_dough_temperature(78.0, 24.0, 70.0, 74.0, None);
    assert!((water - 66.0).abs() < 0.001);
}

#[test]
fn solves_for_piece_count_using_piece_mass() {
    let mut props = BTreeMap::new();
    props.insert(
        "piece_mass".into(),
        Value::Quantity(Quantity {
            value: 310.0,
            unit: "g".into(),
            dimension: Dimension::Mass,
        }),
    );
    let formula = Formula {
        id: Uuid::new_v4(),
        recipe_id: None,
        symbol: "dough".into(),
        name: "Dough".into(),
        basis: FormulaBasis::ReferencePercent,
        ingredients: vec![
            ingredient("flour", 100.0, true, true, 0.0),
            ingredient("water", 50.0, false, false, 1.0),
        ],
        properties: props,
    };
    let result = formula.solve_for_pieces(2.0).expect("solves");
    assert!((result.target_mass_grams - 620.0).abs() < 0.001);
    // 100% flour + 50% water → flour is 2/3 of the batch.
    assert!((result.reference_mass_grams - (620.0 * 100.0 / 150.0)).abs() < 0.1);
    assert!((result.hydration_percent - 50.0).abs() < 0.1);
}

#[test]
fn solves_for_round_pan_volume() {
    let formula = Formula {
        id: Uuid::new_v4(),
        recipe_id: None,
        symbol: "dough".into(),
        name: "Dough".into(),
        basis: FormulaBasis::ReferencePercent,
        ingredients: vec![
            ingredient("flour", 100.0, true, true, 0.0),
            ingredient("water", 60.0, false, false, 1.0),
        ],
        properties: BTreeMap::new(),
    };
    // 30 cm pizza, 0.4 cm thick, density 1.1 → π·15²·0.4·1.1 ≈ 311 g
    let result = formula.solve_for_round_pan(30.0, 0.4).expect("solves");
    assert!((result.target_mass_grams - 311.0).abs() < 2.0);
}

#[test]
fn desired_dough_temperature_with_preferment() {
    let water = desired_dough_temperature(78.0, 24.0, 70.0, 74.0, Some(65.0));
    assert!((water - 79.0).abs() < 0.001);
}
