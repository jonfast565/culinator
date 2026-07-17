use super::StructuredRecipeParser;
use culinator_models::{StructuredInputFormat, StructuredRecipeImporter};
use culinator_parser::parse_recipe;

const JSON_LD: &str = r#"{
  "@context": "https://schema.org",
  "@type": "Recipe",
  "name": "Easy Crepes",
  "description": "Simple breakfast crepes.",
  "recipeIngredient": ["2 large eggs", "1 cup milk", "1 cup flour"],
  "recipeInstructions": [
    {"@type": "HowToStep", "text": "Whisk eggs and milk."},
    {"@type": "HowToStep", "text": "Add flour and rest 30 minutes."}
  ]
}"#;

#[test]
fn jsonld_import_parses_as_valid_cg() {
    let draft = StructuredRecipeParser
        .import(culinator_models::StructuredInput {
            format: StructuredInputFormat::JsonLd,
            content: JSON_LD.to_owned(),
        })
        .unwrap();
    let recipe = parse_recipe(&draft.source_text).unwrap();
    assert_eq!(recipe.title, "Easy Crepes");
    assert_eq!(recipe.resources.len(), 3);
    assert_eq!(recipe.operations.len(), 2);
    assert!(draft.source_text.contains("recipe easy_crepes"));
}

#[test]
fn generic_json_import_round_trips() {
    let content = r#"{
      "title": "Tea",
      "ingredients": ["1 bag black tea", "8 oz water"],
      "instructions": ["Steep 3 minutes"]
    }"#;
    let draft = StructuredRecipeParser
        .import(culinator_models::StructuredInput {
            format: StructuredInputFormat::Json,
            content: content.to_owned(),
        })
        .unwrap();
    let recipe = parse_recipe(&draft.source_text).unwrap();
    assert_eq!(recipe.title, "Tea");
    assert_eq!(recipe.resources.len(), 2);
    assert_eq!(recipe.operations.len(), 1);
}

#[test]
fn yaml_import_round_trips() {
    let content = r#"
title: Pancakes
ingredients:
  - 1 cup flour
  - 1 egg
instructions:
  - Mix batter
  - Cook on griddle
"#;
    let draft = StructuredRecipeParser
        .import(culinator_models::StructuredInput {
            format: StructuredInputFormat::Yaml,
            content: content.to_owned(),
        })
        .unwrap();
    let recipe = parse_recipe(&draft.source_text).unwrap();
    assert_eq!(recipe.title, "Pancakes");
    assert_eq!(recipe.resources.len(), 2);
}
