use super::*;
use culinator_core::{
    BindingRole, Dimension, DependencyKind, DonenessKind, HeatLevel, LaborMode, ResourceKind, Value,
};

fn op<'a>(recipe: &'a culinator_core::Recipe, symbol: &str) -> &'a culinator_core::Operation {
    recipe
        .operations
        .iter()
        .find(|o| o.symbol == symbol)
        .unwrap_or_else(|| panic!("operation `{symbol}` not found"))
}

#[test]
fn semantic_parser_ignores_block_comments() {
    let source = "culinator 0.3; /* comment */ recipe bread { title \"Bread\"; }";
    let recipe = parse_semantic_recipe(source).expect("semantic recipe");
    assert_eq!(recipe.title, "Bread");
}

#[test]
fn prep_sugar_desugars_to_operation_with_bindings() {
    let source = r#"culinator 0.3;
recipe guac {
    title "Guac";
    ingredient jalapeno measured by count { name "jalapeno"; }
    process prep {
        prep mince jalapeno into minced_jalapeno { duration 2 min; }
    }
}"#;
    let recipe = parse_semantic_recipe(source).expect("semantic recipe");
    let op = recipe
        .operations
        .iter()
        .find(|op| op.symbol == "mince_jalapeno")
        .expect("prep desugared to mince_jalapeno operation");
    assert_eq!(op.declared_type.name, "Mince");
    assert_eq!(op.process, "prep");
    assert_eq!(op.labor, Some(LaborMode::Active));
    assert_eq!(op.duration_max_seconds, Some(120));
    assert!(op.bindings.iter().any(|b| b.role == BindingRole::Input
        && b.resource == "jalapeno"));
    assert!(op.bindings.iter().any(|b| b.role == BindingRole::Output
        && b.resource == "minced_jalapeno"));
    // The produced material becomes an implicit intermediate resource.
    let material = recipe
        .resources
        .iter()
        .find(|r| r.symbol == "minced_jalapeno")
        .expect("output registered as intermediate");
    assert_eq!(material.kind, ResourceKind::Intermediate);
    // The process records the desugared operation.
    let process = recipe.processes.iter().find(|p| p.symbol == "prep").unwrap();
    assert!(process.operations.contains(&"mince_jalapeno".to_string()));
}

#[test]
fn prep_sugar_defaults_output_symbol_and_labor() {
    let source = r#"culinator 0.3;
recipe guac {
    title "Guac";
    ingredient onion measured by count { name "onion"; }
    prep dice onion;
}"#;
    let recipe = parse_semantic_recipe(source).expect("semantic recipe");
    let op = recipe
        .operations
        .iter()
        .find(|op| op.symbol == "dice_onion")
        .expect("prep desugared at recipe root");
    assert_eq!(op.labor, Some(LaborMode::Active));
    assert!(op.bindings.iter().any(|b| b.role == BindingRole::Output
        && b.resource == "onion_dice"));
}

fn parse_op_body(body: &str) -> culinator_core::Recipe {
    let source = format!(
        "culinator 0.3;\nrecipe r {{\n  process p {{\n    operation step does heat {{\n{body}\n    }}\n  }}\n}}"
    );
    parse_semantic_recipe(&source).expect("semantic recipe")
}

#[test]
fn operation_temperature_and_heat_level() {
    let recipe = parse_op_body("      temperature 350 f;\n      heat medium_high;");
    let step = op(&recipe, "step");
    let temp = step.target_temperature.as_ref().expect("temperature");
    assert_eq!(temp.value, 350.0);
    assert_eq!(temp.unit, "f");
    assert_eq!(temp.dimension, Dimension::Temperature);
    assert_eq!(step.heat_level, Some(HeatLevel::MediumHigh));
}

#[test]
fn operation_typed_doneness_cues() {
    let recipe = parse_op_body(
        "      until internal_temp 165 f;\n      until visual \"golden brown\";\n      until rise \"doubled in size\";",
    );
    let step = op(&recipe, "step");
    assert_eq!(step.doneness.len(), 3);
    assert_eq!(step.doneness[0].kind, DonenessKind::InternalTemp);
    assert!(matches!(step.doneness[0].value, Value::Quantity(ref q) if q.value == 165.0));
    assert_eq!(step.doneness[1].kind, DonenessKind::Visual);
    assert!(matches!(step.doneness[1].value, Value::Text(ref s) if s == "golden brown"));
    assert_eq!(step.doneness[2].kind, DonenessKind::Rise);
}

#[test]
fn operation_duration_range_up_to_and_estimated() {
    let range = parse_op_body("      duration 25 min to 30 min;");
    let step = op(&range, "step");
    assert_eq!(step.duration_min_seconds, Some(1500));
    assert_eq!(step.duration_max_seconds, Some(1800));
    assert!(!step.duration_estimated);

    let up_to = parse_op_body("      duration up to 8 h;");
    let step = op(&up_to, "step");
    assert_eq!(step.duration_min_seconds, Some(0));
    assert_eq!(step.duration_max_seconds, Some(28800));

    let est = parse_op_body("      duration estimated 30 min;");
    let step = op(&est, "step");
    assert!(step.duration_estimated);
    assert_eq!(step.duration_min_seconds, Some(1800));
    assert_eq!(step.duration_max_seconds, Some(1800));
}

#[test]
fn operation_dependency_modifiers() {
    let source = r#"culinator 0.3;
recipe r {
  process p {
    operation a does mix { duration 1 min; produces x; }
    operation b does mix {
      input x;
      after a lag 30 min optional;
    }
    operation c does mix { input x; after a start_start; }
  }
}"#;
    let recipe = parse_semantic_recipe(source).expect("semantic recipe");
    let b = op(&recipe, "b");
    let dep = &b.dependencies[0];
    assert_eq!(dep.predecessor, "a");
    assert_eq!(dep.minimum_lag_seconds, Some(1800));
    assert!(dep.optional);
    let c = op(&recipe, "c");
    assert_eq!(c.dependencies[0].kind, DependencyKind::StartStart);
}

#[test]
fn operation_binding_partial_quantity() {
    let recipe = parse_op_body("      input butter 6 tbsp;\n      input [sugar, eggs];");
    let step = op(&recipe, "step");
    let butter = step
        .bindings
        .iter()
        .find(|b| b.resource == "butter")
        .expect("butter binding");
    let qty = butter.quantity.as_ref().expect("partial quantity");
    assert_eq!(qty.value, 6.0);
    assert_eq!(qty.unit, "tbsp");
    // List form has no per-item quantity.
    assert!(step
        .bindings
        .iter()
        .find(|b| b.resource == "sugar")
        .unwrap()
        .quantity
        .is_none());
}

#[test]
fn resource_optional_divided_substitutes_and_ranged_quantity() {
    let source = r#"culinator 0.3;
recipe r {
  ingredient garlic measured by count {
    quantity 2 to 3 clove;
    optional true;
    substitutes [garlic_powder];
  }
  ingredient butter measured by mass {
    quantity 2 stick;
    divided true;
  }
}"#;
    let recipe = parse_semantic_recipe(source).expect("semantic recipe");
    let garlic = recipe.resources.iter().find(|r| r.symbol == "garlic").unwrap();
    assert!(garlic.optional);
    assert_eq!(garlic.substitutes.len(), 1);
    match garlic.properties.get("quantity") {
        Some(Value::Range { min, max }) => {
            assert!(matches!(**min, Value::Number(n) if n == 2.0));
            assert!(matches!(**max, Value::Quantity(ref q) if q.value == 3.0 && q.unit == "clove"));
        }
        other => panic!("expected ranged quantity, got {other:?}"),
    }
    // `clove` and `stick` now resolve to the Count dimension, not Ratio.
    let butter = recipe.resources.iter().find(|r| r.symbol == "butter").unwrap();
    assert!(butter.divided);
}

#[test]
fn oz_unit_resolves_to_mass_dimension() {
    let source = r#"culinator 0.3;
recipe r {
  ingredient cheese measured by mass { quantity 8 oz; }
}"#;
    let recipe = parse_semantic_recipe(source).expect("semantic recipe");
    let cheese = recipe.resources.iter().find(|r| r.symbol == "cheese").unwrap();
    match cheese.properties.get("quantity") {
        Some(Value::Quantity(q)) => assert_eq!(q.dimension, Dimension::Mass),
        other => panic!("expected mass quantity, got {other:?}"),
    }
}

#[test]
fn expanded_cooking_units_classify_by_dimension() {
    use Dimension::*;
    let cases: &[(&str, Dimension)] = &[
        ("gram", Mass),
        ("pounds", Mass),
        ("oz", Mass),
        ("tablespoon", Volume),
        ("tsp", Volume),
        ("pint", Volume),
        ("gallon", Volume),
        ("dash", Volume),
        ("fahrenheit", Temperature),
        ("celsius", Temperature),
        ("minutes", Time),
        ("hours", Time),
        ("days", Time),
        ("inch", Length),
        ("cm", Length),
        ("clove", Count),
        ("bunch", Count),
        ("head", Count),
        ("dozen", Count),
        ("sprig", Count),
    ];
    for (unit, expected) in cases {
        assert_eq!(
            Dimension::from_unit(unit),
            *expected,
            "unit `{unit}` should be {expected:?}"
        );
    }
    // Case-insensitive and trailing-dot tolerant.
    assert_eq!(Dimension::from_unit("OZ"), Mass);
    assert_eq!(Dimension::from_unit("tbsp."), Volume);
}

#[test]
fn mass_units_convert_to_grams() {
    use culinator_core::Quantity;
    let q = |value: f64, unit: &str| Quantity {
        value,
        unit: unit.to_owned(),
        dimension: Dimension::from_unit(unit),
    };
    assert_eq!(q(1.0, "kg").as_grams(), Some(1000.0));
    assert_eq!(q(1.0, "pound").as_grams(), Some(453.592_37));
    assert_eq!(q(16.0, "oz").as_grams(), Some(16.0 * 28.349_523_125));
    // Non-mass units have no gram equivalent.
    assert_eq!(q(2.0, "cup").as_grams(), None);
}

#[test]
fn duration_accepts_long_form_time_units() {
    let recipe = parse_op_body("      duration 2 hours to 3 hours;");
    let step = op(&recipe, "step");
    assert_eq!(step.duration_min_seconds, Some(7200));
    assert_eq!(step.duration_max_seconds, Some(10800));

    let days = parse_op_body("      duration up to 2 days;");
    let step = op(&days, "step");
    assert_eq!(step.duration_max_seconds, Some(172_800));
}
