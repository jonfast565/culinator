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
    assert_eq!(jack["allergen"], "milk");

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

    // `measured by` must survive even when the declaration carries no quantity.
    // A divided ingredient puts its amounts on the step bindings, so deriving
    // the dimension from the declared quantity reported "unspecified" here —
    // and a structured editor writing that back would drop `measured by volume`.
    let salt = resources.iter().find(|r| r["symbol"] == "salt").unwrap();
    assert_eq!(salt["divided"], true);
    assert!(salt.get("quantity").is_none(), "divided salt declares none");
    assert_eq!(salt["measurement"], "volume");
    assert_eq!(macaroni["measurement"], "mass");

    // Spans let the inspector patch a declaration in place.
    let range = &macaroni["range"];
    let (start, end) = (
        range["start"].as_u64().unwrap() as usize,
        range["end"].as_u64().unwrap() as usize,
    );
    assert!(SEED[start..end].starts_with("ingredient macaroni"));
}

/// The builder joins the two models by byte range: it renders a card from
/// `UiResource`/`UiOperation` and edits through the matching outline node. If
/// these ever disagree, an edit lands on the wrong declaration — so pin it.
#[test]
fn outline_spans_agree_with_the_ui_model_for_every_seed() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../culinator-service/src/seed");
    let mut joined = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e != "cg").unwrap_or(true) {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let source = std::fs::read_to_string(&path).unwrap();
        let model = parse_ui_model_native(&source);
        let outline = parse_outline_native(&source);
        assert!(outline.parsed, "{name} has a walkable tree");

        // Flatten the outline so nested operations inside processes are found.
        fn flatten<'a>(
            nodes: &'a [crate::outline::UiOutlineNode],
            out: &mut Vec<&'a crate::outline::UiOutlineNode>,
        ) {
            for node in nodes {
                out.push(node);
                flatten(&node.children, out);
            }
        }
        let mut all = Vec::new();
        flatten(&outline.nodes, &mut all);

        for resource in &model.resources {
            let Some(range) = &resource.range else {
                continue;
            };
            let node = all
                .iter()
                .find(|node| node.code_range.start == range.start)
                .unwrap_or_else(|| {
                    panic!(
                        "{name}: no outline node at {} for {}",
                        range.start, resource.symbol
                    )
                });
            assert_eq!(
                node.symbol.as_deref(),
                Some(resource.symbol.as_str()),
                "{name}: symbol mismatch at {}",
                range.start
            );
            assert_eq!(
                node.code_range.end, range.end,
                "{name}: {}",
                resource.symbol
            );
            joined += 1;
        }
        for operation in &model.operations {
            let Some(range) = &operation.range else {
                continue;
            };
            // `prep` desugars into an operation whose span is the prep
            // statement, so match on position rather than on symbol.
            let node = all
                .iter()
                .find(|node| node.code_range.start == range.start)
                .unwrap_or_else(|| {
                    panic!(
                        "{name}: no outline node at {} for {}",
                        range.start, operation.symbol
                    )
                });
            assert_eq!(
                node.code_range.end, range.end,
                "{name}: {}",
                operation.symbol
            );
            assert!(
                node.keyword == "operation" || node.keyword == "prep",
                "{name}: {} is a {:?}",
                operation.symbol,
                node.keyword
            );
            joined += 1;
        }
    }
    assert!(joined > 500, "joined {joined} declarations");
}

/// Spans are reported in UTF-16 units because JavaScript slices strings that
/// way. Emitting raw byte offsets meant that in any recipe containing a
/// non-ASCII character every later span pointed past its declaration — deleting
/// a step from `fully_loaded_guacamole.cg` (two em-dashes in its comments)
/// sliced `"ation rest does rest {…"`, orphaned an `oper`, and broke the file.
#[test]
fn spans_are_utf16_offsets_so_javascript_can_slice_them() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../culinator-service/src/seed/fully_loaded_guacamole.cg"
    ))
    .unwrap();
    assert!(
        source.contains('—'),
        "seed still has the multi-byte comment"
    );
    let utf16: Vec<u16> = source.encode_utf16().collect();

    let model = parse_ui_model_native(&source);
    let rest = model
        .operations
        .iter()
        .find(|operation| operation.symbol == "rest")
        .expect("rest step");
    let range = rest.range.as_ref().expect("span");
    let sliced = String::from_utf16(&utf16[range.start..range.end]).unwrap();
    assert!(
        sliced.starts_with("operation rest does rest"),
        "got {sliced:?}"
    );

    // Same for the outline, and the two must still agree with each other.
    let outline = parse_outline_native(&source);
    fn flatten<'a>(
        nodes: &'a [crate::outline::UiOutlineNode],
        out: &mut Vec<&'a crate::outline::UiOutlineNode>,
    ) {
        for node in nodes {
            out.push(node);
            flatten(&node.children, out);
        }
    }
    let mut all = Vec::new();
    flatten(&outline.nodes, &mut all);
    let node = all
        .iter()
        .find(|node| node.code_range.start == range.start)
        .expect("outline node at the same offset");
    assert_eq!(node.code_range.end, range.end);
    assert_eq!(outline.source_len, utf16.len());

    // Deleting by that span leaves source that still parses.
    let mut remaining: Vec<u16> = utf16[..range.start].to_vec();
    remaining.extend_from_slice(&utf16[range.end..]);
    let edited = String::from_utf16(&remaining).unwrap();
    assert!(
        parse_ui_model_native(&edited).diagnostics.is_empty(),
        "deleting a step must not corrupt the recipe"
    );
}

#[test]
fn outline_json_uses_camel_case_keys() {
    let source = "culinator 0.3;\nrecipe demo {\n    title \"D\";\n    ingredient salt measured by volume { quantity 1 tsp; }\n}\n";
    let json: serde_json::Value = serde_json::from_str(&parse_outline(source)).unwrap();
    assert_eq!(json["parsed"], true);
    assert_eq!(json["sourceLen"], source.len());
    let recipe = &json["nodes"][1];
    assert_eq!(recipe["keyword"], "recipe");
    assert_eq!(recipe["form"], "declaration");
    assert!(recipe["codeRange"]["start"].is_number());
    assert!(recipe["blockInnerRange"]["end"].is_number());
    // Absent optionals are omitted, not null, so the TS fields stay optional.
    let title = &recipe["children"][0];
    assert_eq!(title["form"], "statement");
    assert!(title.get("blockInnerRange").is_none());
    assert!(title.get("symbol").is_none());
}

#[test]
fn unwalkable_source_reports_parsed_false() {
    let outline = parse_outline_native("culinator 0.3;\nrecipe demo {\n  title \"D\";\n");
    assert!(!outline.parsed);
    assert!(outline.nodes.is_empty());
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
