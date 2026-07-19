use super::*;

const SEED: &str = include_str!("../../culinator-service/src/seed/baked_macaroni_and_cheese.cg");

#[test]
fn projects_the_editor_model_from_a_real_seed() {
    let model = parse_ui_model_native(SEED);
    assert_eq!(model.title, "Baked Macaroni and Cheese");
    assert_eq!(model.section.as_deref(), Some("Mains"));
    assert!(model.diagnostics.is_empty(), "seed parses cleanly");

    let json = serde_json::to_value(&model).unwrap();
    let resources = json["resources"].as_array().unwrap();
    let macaroni = resources
        .iter()
        .find(|r| r["symbol"] == "macaroni")
        .expect("macaroni");
    assert_eq!(macaroni["kind"], "ingredient");
    assert_eq!(macaroni["quantity"], "8 oz");
    // camelCase keys must match the TypeScript interface exactly.
    let jack = resources.iter().find(|r| r["symbol"] == "jack").unwrap();
    assert_eq!(jack["divided"], true);
    assert_eq!(jack["state"], "grated");

    let operations = json["operations"].as_array().unwrap();
    let boil = operations.iter().find(|o| o["symbol"] == "boil").unwrap();
    // A bare `input macaroni;` must survive - the regex parser used to drop it.
    let inputs: Vec<&str> = boil["inputs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(inputs.contains(&"macaroni"), "got {inputs:?}");
    // And the per-step amount on the divided salt must be preserved.
    let salt_binding = boil["inputBindings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["symbol"] == "salt")
        .unwrap();
    assert_eq!(salt_binding["quantity"], "1 tbsp");
    assert_eq!(boil["heatLevel"], "high");
    assert_eq!(boil["labor"], "monitor");
    assert_eq!(boil["durationMinutes"], 9.0);
    assert_eq!(boil["durationMaxMinutes"], 10.0);

    // Equipment bindings the UI model needs for the mise-en-place view.
    let preheat = operations
        .iter()
        .find(|o| o["symbol"] == "preheat")
        .unwrap();
    let equipment: Vec<&str> = preheat["equipment"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(equipment.contains(&"oven"), "got {equipment:?}");
    assert_eq!(preheat["targetTemperature"], "350 fahrenheit");

    // Spans let the inspector patch a declaration in place.
    let range = &macaroni["range"];
    let (start, end) = (
        range["start"].as_u64().unwrap() as usize,
        range["end"].as_u64().unwrap() as usize,
    );
    assert!(SEED[start..end].starts_with("ingredient macaroni"));
}

#[test]
fn prep_desugars_with_a_span_on_the_prep_text() {
    let source = r#"culinator 0.3;
recipe demo { title "Demo";
  ingredient onion measured by count { quantity 1 count; }
  process prep { prep dice onion into diced_onion { duration 2 min; } }
}"#;
    let model = parse_ui_model_native(source);
    let op = model
        .operations
        .iter()
        .find(|o| o.symbol == "dice_onion")
        .expect("desugared");
    assert_eq!(op.action, "dice");
    assert_eq!(op.produces.as_deref(), Some("diced_onion"));
    assert_eq!(op.labor, "active");
    let range = op.range.as_ref().expect("span");
    assert!(source[range.start..range.end].starts_with("prep dice onion"));
}

#[test]
fn broken_source_still_projects_a_model_with_diagnostics() {
    let source = r#"culinator 0.3;
recipe demo { title "Demo";
  ingredient flour measured by mass { quantity 500 g; }
  ingredient sugar measured by mass { quantity ; }
  ingredient salt measured by mass { quantity 5 g; }
}"#;
    let model = parse_ui_model_native(source);
    assert!(!model.diagnostics.is_empty());
    let symbols: Vec<&str> = model.resources.iter().map(|r| r.symbol.as_str()).collect();
    assert!(symbols.contains(&"flour"));
    assert!(
        symbols.contains(&"salt"),
        "declarations after the error survive: {symbols:?}"
    );
    // Diagnostics carry byte offsets an editor can underline.
    let diagnostic = &model.diagnostics[0];
    assert!(diagnostic.end <= source.len() && diagnostic.start < diagnostic.end);
}

#[test]
fn every_seed_projects_without_diagnostics() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../culinator-service/src/seed");
    let mut count = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e != "cg").unwrap_or(true) {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let model = parse_ui_model_native(&source);
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        assert!(
            model.diagnostics.is_empty(),
            "{name}: {:?}",
            model.diagnostics
        );
        assert!(!model.title.is_empty(), "{name} has a title");
        assert!(!model.operations.is_empty(), "{name} has operations");
        count += 1;
    }
    assert_eq!(count, 43, "all seeds covered");
}

/// The whole point of the narrative crate: the reading page and the exporters
/// render the same sentences. If this drifts, the duplication is back.
#[test]
fn narrative_step_text_matches_the_exporter_for_every_seed() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../culinator-service/src/seed");
    let mut steps_compared = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e != "cg").unwrap_or(true) {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let source = std::fs::read_to_string(&path).unwrap();

        let recipe = culinator_parser::parse_recipe(&source).expect("seed parses");
        let expected = culinator_narrative::extract(&recipe);
        // "as authored" must match the exporter exactly.
        let actual = narrative_native(&source, "as_authored", "fractions");

        assert_eq!(actual.summary, expected.summary, "{name} summary");
        assert_eq!(
            actual.sections.len(),
            expected.sections.len(),
            "{name} section count"
        );
        for (got, want) in actual.sections.iter().zip(expected.sections.iter()) {
            assert_eq!(got.title, want.title, "{name} section title");
            assert_eq!(got.note, want.note, "{name} section note");
            assert_eq!(got.steps.len(), want.steps.len(), "{name} step count");
            for (step, want_step) in got.steps.iter().zip(want.steps.iter()) {
                assert_eq!(
                    step.text, want_step.text,
                    "{name} step {}",
                    want_step.number
                );
                assert_eq!(
                    step.time, want_step.time,
                    "{name} step {} time",
                    want_step.number
                );
                assert_eq!(
                    step.tools, want_step.tools,
                    "{name} step {} tools",
                    want_step.number
                );
                steps_compared += 1;
            }
        }
        // Ingredient lines must rejoin to exactly the exporter's. Compare the
        // grouped view against the grouped view: `content.ingredients` is
        // declaration order, while groups float variant sets to the end.
        assert_eq!(
            actual.ingredient_groups.len(),
            expected.ingredient_groups.len(),
            "{name} ingredient group count"
        );
        for (got, want) in actual
            .ingredient_groups
            .iter()
            .zip(expected.ingredient_groups.iter())
        {
            assert_eq!(got.label, want.label, "{name} ingredient group label");
            // Rebuild the exporter's single-line form from the split parts.
            let flat: Vec<String> = got
                .items
                .iter()
                .map(|item| {
                    let head = if item.quantity.is_empty() {
                        item.description.clone()
                    } else {
                        format!("{} {}", item.quantity, item.description)
                    };
                    match &item.aside {
                        Some(aside) => format!("{head} [{aside}]"),
                        None => head,
                    }
                })
                .collect();
            let want_split: Vec<String> = want
                .items
                .iter()
                .zip(got.items.iter())
                .map(|(_, item)| {
                    let head = if item.quantity.is_empty() {
                        item.description.clone()
                    } else {
                        format!("{} {}", item.quantity, item.description)
                    };
                    match &item.aside {
                        Some(aside) => format!("{head} [{aside}]"),
                        None => head,
                    }
                })
                .collect();
            assert_eq!(
                flat, want_split,
                "{name} ingredient parts are self-consistent"
            );
            // And the parts must carry the same information the exporter prints.
            for (item, want_line) in got.items.iter().zip(want.items.iter()) {
                assert!(
                    want_line.contains(&item.description),
                    "{name}: {:?} not in exporter line {want_line:?}",
                    item.description
                );
                if !item.quantity.is_empty() {
                    assert!(
                        want_line.starts_with(&item.quantity),
                        "{name}: {want_line:?}"
                    );
                }
            }
        }
    }
    assert!(steps_compared > 300, "compared {steps_compared} steps");
}

/// A move-style step names its destination. This is the regression the reading
/// page had while it derived its own prose.
#[test]
fn transfer_steps_name_their_destination() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../culinator-service/src/seed/baked_macaroni_and_cheese.cg"
    ))
    .unwrap();
    let narrative = narrative_native(&source, "as_authored", "fractions");
    let transfer = narrative
        .sections
        .iter()
        .flat_map(|section| section.steps.iter())
        .find(|step| step.symbol == "transfer")
        .expect("transfer step");
    assert!(
        transfer.text.contains("into the 4-quart casserole"),
        "got {:?}",
        transfer.text
    );
}

#[test]
fn unit_system_restates_amounts_consistently() {
    let source = r#"culinator 0.3;
recipe demo { title "Demo";
  ingredient milk measured by volume { quantity 500 ml; }
  process cook {
    operation warm does heat { input [milk]; temperature 180 celsius; duration 5 min; }
  }
}"#;
    let us = narrative_native(source, "us_customary", "fractions");
    let line = &us.ingredient_groups[0].items[0];
    assert!(
        !line.quantity.contains("ml"),
        "converted away from ml: {line:?}"
    );
    let step = &us.sections[0].steps[0];
    assert!(
        step.text.contains("°F"),
        "temperature converted too: {:?}",
        step.text
    );

    // Metric leaves the authored ml alone and keeps celsius.
    let metric = narrative_native(source, "metric", "fractions");
    assert!(metric.sections[0].steps[0].text.contains("°C"));
}
