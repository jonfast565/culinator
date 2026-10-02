use super::*;

fn request(source: &str, mass: f64) -> BatchRequest {
    BatchRequest {
        source_text: source.into(),
        formula_symbol: "dough".into(),
        formula: None,
        constraint: FormulaConstraint::TargetMass { grams: mass },
        rounding_increment_grams: None,
        apply_minimums: false,
        piece_count: None,
    }
}

#[test]
fn pizza_batch_scales_step_input_and_keeps_advanced_fields() {
    let source = include_str!("../../../culinator-service/src/seed/pizza_dough.cg");
    let source = source.replace(
        "stage final;",
        "stage final;\n            reference_group main;\n            min_mass 0.1 g;",
    );
    let preview = plan_batch(&request(&source, 1256.0));
    assert!(preview.blockers.is_empty(), "{:?}", preview.blockers);
    assert!(preview.proposed_source.contains("quantity 800 g;"));
    assert!(preview.proposed_source.contains("input flour 800 g;"));
    assert!(preview.source_diff.contains("-        quantity 400 g;"));
    assert!(preview.source_diff.contains("+        quantity 800 g;"));
    assert!(preview.proposed_source.contains("reference_group main;"));
    assert!(preview.proposed_source.contains("min_mass 0.1 g;"));
    assert!(preview.proposed_source.contains("piece_mass 628 g;"));
    assert!(preview.warnings.iter().any(|w| w.contains("1 tbsp")));
    parse_recipe(&preview.proposed_source).unwrap();
    assert!(apply_batch(&request(&source, 1256.0), "stale").is_err());
    assert_eq!(
        apply_batch(&request(&source, 1256.0), &preview.source_fingerprint)
            .unwrap()
            .proposed_source,
        preview.proposed_source
    );
}

#[test]
fn volume_is_kept_and_missing_ingredient_blocks() {
    let source = "culinator 0.3; recipe bread {\n ingredient flour measured by mass { quantity 100 g; }\n ingredient water measured by volume { quantity 100 ml; }\n formula dough relative to flour { target 200 g; ingredient flour { percentage 100%; reference true; flour true; } ingredient water { percentage 100%; water_fraction 1; } }\n process method { operation mix does mix { input flour 100 g; input water 50 ml; } }\n }";
    let preview = plan_batch(&request(source, 400.0));
    assert!(preview.blockers.is_empty(), "{:?}", preview.blockers);
    assert!(preview.proposed_source.contains("quantity 200 ml;"));
    assert!(preview.proposed_source.contains("input water 100 ml;"));
    let broken = source.replace(
        "ingredient water measured by volume",
        "ingredient missing measured by volume",
    );
    let preview = plan_batch(&request(&broken, 400.0));
    assert!(
        preview
            .blockers
            .iter()
            .any(|b| b.contains("No recipe ingredient"))
    );
    assert_eq!(preview.proposed_source, broken);
}

#[test]
fn divided_step_amounts_cannot_exceed_the_saved_ingredient_total() {
    let source = include_str!("../../../culinator-service/src/seed/pizza_dough.cg")
        .replace("input flour 400 g;", "input flour 450 g;");
    let preview = plan_batch(&request(&source, 1256.0));
    assert!(
        preview
            .blockers
            .iter()
            .any(|blocker| blocker.contains("Explicit step amounts for `flour` exceed"))
    );
    assert_eq!(preview.proposed_source, source);
}

#[test]
fn numeric_prose_is_warned_about_but_preserved() {
    let source = include_str!("../../../culinator-service/src/seed/pizza_dough.cg").replace(
        "title \"Pizza Dough\";",
        "title \"Pizza Dough\";\n    description \"Makes 2 balls of dough\";",
    );
    let preview = plan_batch(&request(&source, 1256.0));
    assert!(preview.blockers.is_empty(), "{:?}", preview.blockers);
    assert!(
        preview
            .warnings
            .iter()
            .any(|warning| warning.contains("Makes 2 balls"))
    );
    assert!(
        preview
            .proposed_source
            .contains("description \"Makes 2 balls of dough\";")
    );

    let percentage_note =
        include_str!("../../../culinator-service/src/seed/pizza_dough_62_percent.cg");
    let preview = plan_batch(&request(percentage_note, 834.0));
    assert!(
        preview
            .warnings
            .iter()
            .any(|warning| warning.contains("62%"))
    );
}

#[test]
fn formula_target_keeps_its_authored_unit_and_untouched_source() {
    let source = "culinator 0.3; recipe bread { ingredient flour measured by mass { quantity 1 lb; } formula dough relative to flour { target 1 lb; ingredient flour { percentage 100%; reference true; flour true; } } process method { operation mix does mix { input [flour]; } } }";
    let unchanged = plan_batch(&request(source, 453.59237));
    assert!(unchanged.blockers.is_empty(), "{:?}", unchanged.blockers);
    assert_eq!(unchanged.proposed_source, source);
    assert!(unchanged.source_diff.is_empty());

    let doubled = plan_batch(&request(source, 907.18474));
    assert!(doubled.blockers.is_empty(), "{:?}", doubled.blockers);
    assert!(doubled.proposed_source.contains("target 2 lb;"));
    assert!(doubled.proposed_source.contains("quantity 2 lb;"));
}

#[test]
fn fixed_mass_rows_and_source_comments_survive_scaling() {
    let source = "culinator 0.3; recipe bread {\n ingredient flour measured by mass { quantity 100 g; }\n ingredient water measured by mass { quantity 100 g; }\n ingredient salt measured by mass { quantity 5 g; }\n formula dough relative to flour {\n target 205 g;\n ingredient flour { percentage 100%; reference true; flour true; }\n ingredient water { percentage 100%; water_fraction 1; }\n // Keep this fixed for every batch.\n ingredient salt { mass 5 g; basis absolute; custom_field keep; }\n }\n process method { operation mix does mix { input [flour, water, salt]; } }\n }";
    let preview = plan_batch(&request(source, 305.0));
    assert!(preview.blockers.is_empty(), "{:?}", preview.blockers);
    assert!(preview.proposed_source.contains("quantity 150 g;"));
    assert!(preview.proposed_source.contains("quantity 5 g;"));
    assert!(
        preview
            .proposed_source
            .contains("mass 5 g; basis absolute; custom_field keep;")
    );
    assert!(
        preview
            .proposed_source
            .contains("// Keep this fixed for every batch.")
    );
}

#[test]
fn added_formula_row_is_inserted_without_reprinting_existing_rows() {
    let source = "culinator 0.3; recipe bread {\n ingredient flour measured by mass { quantity 100 g; }\n ingredient water measured by mass { quantity 100 g; }\n ingredient salt measured by mass { quantity 1 g; }\n formula dough relative to flour { target 200 g; ingredient flour { percentage 100%; reference true; flour true; custom_field keep; } ingredient water { percentage 100%; water_fraction 1; } }\n process method { operation mix does mix { input [flour, water, salt]; } }\n }";
    let mut formula = parse_recipe(source).unwrap().formulas.remove(0);
    let mut salt = formula.ingredients[1].clone();
    salt.symbol = "salt".into();
    salt.name = "salt".into();
    salt.percentage = Some(1.0);
    salt.water_fraction = 0.0;
    salt.properties.clear();
    formula.ingredients.push(salt);
    let mut req = request(source, 201.0);
    req.formula = Some(formula);
    let preview = plan_batch(&req);
    assert!(preview.blockers.is_empty(), "{:?}", preview.blockers);
    assert!(preview.proposed_source.contains("custom_field keep;"));
    assert!(preview.proposed_source.contains("ingredient salt {"));
    assert_eq!(
        parse_recipe(&preview.proposed_source).unwrap().formulas[0]
            .ingredients
            .len(),
        3
    );
}

#[test]
fn clearing_a_formula_role_removes_only_that_statement() {
    let source = include_str!("../../../culinator-service/src/seed/pizza_dough.cg");
    let mut formula = parse_recipe(source).unwrap().formulas.remove(0);
    let oil = formula
        .ingredients
        .iter_mut()
        .find(|item| item.symbol == "olive_oil")
        .unwrap();
    oil.properties.remove("role");
    let mut req = request(source, 628.0);
    req.formula = Some(formula);
    let preview = plan_batch(&req);
    assert!(preview.blockers.is_empty(), "{:?}", preview.blockers);
    assert!(!preview.proposed_source.contains("role fat;"));
    assert!(
        preview
            .proposed_source
            .contains("ingredient olive_oil measured by mass")
    );
}

#[test]
fn changing_flour_role_updates_the_standard_type() {
    let source = "culinator 0.3; recipe bread { ingredient flour measured by mass { quantity 100 g; } formula dough relative to flour { target 100 g; ingredient flour as Flour<BakersPercent> { percentage 100%; reference true; } } process method { operation mix does mix { input [flour]; } } }";
    let mut formula = parse_recipe(source).unwrap().formulas.remove(0);
    formula.ingredients[0].is_flour = false;
    let mut req = request(source, 100.0);
    req.formula = Some(formula);
    let preview = plan_batch(&req);
    assert!(preview.blockers.is_empty(), "{:?}", preview.blockers);
    assert!(
        preview
            .proposed_source
            .contains("Ingredient<BakersPercent>")
    );
    assert!(!parse_recipe(&preview.proposed_source).unwrap().formulas[0].ingredients[0].is_flour);
}

#[test]
fn ambiguous_count_yield_blocks() {
    let source = include_str!("../../../culinator-service/src/seed/pizza_dough.cg");
    let source = source.replace(
        "    yield bases",
        "    yield spare measured by count { amount 2 count; }\n    yield bases",
    );
    let mut req = request(&source, 1256.0);
    req.piece_count = Some(4.0);
    let preview = plan_batch(&req);
    assert!(
        preview
            .blockers
            .iter()
            .any(|b| b.contains("exactly one count yield"))
    );
}

#[test]
fn piece_and_serving_counts_update_structured_outputs() {
    let source = include_str!("../../../culinator-service/src/seed/pizza_dough.cg");
    let mut req = request(source, 1256.0);
    req.piece_count = Some(4.0);
    let preview = plan_batch(&req);
    assert!(preview.blockers.is_empty(), "{:?}", preview.blockers);
    assert!(preview.proposed_source.contains("pieces 4 count;"));
    assert!(preview.proposed_source.contains("piece_mass 314 g;"));
    assert!(preview.proposed_source.contains("amount 4 count;"));

    let source = "culinator 0.3; recipe bread { ingredient flour measured by mass { quantity 100 g; } formula dough relative to flour { target 100 g; ingredient flour { percentage 100%; reference true; flour true; } } serving portions measured by count { amount 2 count; } process method { operation mix does mix { input [flour]; } } }";
    let mut req = request(source, 400.0);
    req.constraint = FormulaConstraint::Servings {
        count: 4.0,
        grams_per_serving: 100.0,
    };
    let preview = plan_batch(&req);
    assert!(preview.blockers.is_empty(), "{:?}", preview.blockers);
    assert!(preview.proposed_source.contains("amount 4 count;"));
}

#[test]
fn saved_formula_drift_is_reported_without_source_mutation() {
    let source = include_str!("../../../culinator-service/src/seed/pizza_dough.cg")
        .replace("quantity 400 g;", "quantity 410 g;");
    let preview = plan_batch(&request(&source, 628.0));
    assert!(
        preview
            .out_of_sync
            .iter()
            .any(|item| item.contains("flour"))
    );
    assert!(source.contains("quantity 410 g;"));
}

#[test]
fn creates_a_formula_for_a_recipe_that_has_none() {
    let source = include_str!("../../../culinator-service/src/seed/pizza_dough.cg");
    let parsed = parse_recipe(source).unwrap();
    let outline = Outline::parse(source).unwrap();
    let node = node_for(&outline, "formula", "dough").unwrap();
    let bare = apply_text_edits(source, &[TextEdit::delete(node.range)]).unwrap();
    let mut req = request(&bare, 628.0);
    req.formula = Some(parsed.formulas[0].clone());
    let preview = plan_batch(&req);
    assert!(preview.blockers.is_empty(), "{:?}", preview.blockers);
    assert_eq!(
        parse_recipe(&preview.proposed_source)
            .unwrap()
            .formulas
            .len(),
        1
    );
    assert_eq!(preview.source_fingerprint, fingerprint(&bare));
}

#[test]
fn every_seed_is_registered_and_formula_batches_match_declared_amounts() {
    let root = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../culinator-service/src/seed"
    ));
    let registry = include_str!("../../../culinator-service/src/state.rs");
    let mut formula_count = 0;
    let entries = [root.to_path_buf(), root.join("crowded_kitchen")]
        .into_iter()
        .flat_map(|dir| std::fs::read_dir(dir).unwrap())
        .map(Result::unwrap);
    for entry in entries {
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "cg") {
            continue;
        }
        let relative = path.strip_prefix(root).unwrap().to_string_lossy();
        assert!(
            registry.contains(&format!("include_str!(\"seed/{relative}\")")),
            "{relative} is absent from seed registry"
        );
        let source = std::fs::read_to_string(&path).unwrap();
        let recipe = parse_recipe(&source).unwrap_or_else(|error| panic!("{relative}: {error}"));
        for formula in &recipe.formulas {
            formula_count += 1;
            let Some(Value::Quantity(target)) = formula.properties.get("target") else {
                panic!("{relative}: formula missing target");
            };
            let grams = target.as_grams().unwrap();
            let preview = plan_batch(&BatchRequest {
                formula_symbol: formula.symbol.clone(),
                ..request(&source, grams)
            });
            assert!(
                preview.blockers.is_empty(),
                "{relative}: {:?}",
                preview.blockers
            );
            assert!(
                preview.out_of_sync.is_empty(),
                "{relative}: {:?}",
                preview.out_of_sync
            );
            assert!(
                !preview
                    .changes
                    .iter()
                    .any(|change| change.path.starts_with("ingredient.")
                        || change.path.starts_with("operation.")),
                "{relative}: {:?}",
                preview.changes
            );
        }
    }
    assert!(formula_count >= 2);
}
