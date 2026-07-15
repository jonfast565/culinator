use super::*;

const RECIPE: &str = r#"culinograph 0.3; recipe bread { title "Bread"; ingredient flour measured by mass { quantity 500 g; } process mix { operation combine does mix { input [flour]; duration 1 min; } } yield loaf measured by mass { mass 500 g; } }"#;
const BOOK: &str =
    r#"culinograph 0.3; book favorites { title "Favorites"; recipe bread { title "Bread"; } }"#;

#[test]
fn parses_recipe_document() {
    let recipe = parse_recipe(RECIPE).expect("recipe parses");
    assert_eq!(recipe.title, "Bread");
    assert_eq!(recipe.resources.len(), 1);
}

#[test]
fn parses_recipe_book_document() {
    let book = parse_recipe_book(BOOK).expect("book parses");
    assert_eq!(book.title, "Favorites");
    assert_eq!(book.recipes.len(), 1);
}

#[test]
fn rejects_wrong_document_kind() {
    assert!(parse_recipe(BOOK).is_err());
}

#[test]
fn produces_registers_implicit_intermediate() {
    use culinograph_core::{BindingRole, ResourceKind};
    let source = r#"culinograph 0.3; recipe stew { title "Stew";
        ingredient onion measured by count { quantity 1 count; }
        process cook {
            operation saute does heat { input [onion]; produces base; duration 5 min; }
            operation simmer does heat { input [base]; after saute; duration 30 min; }
        }
    }"#;
    let recipe = parse_recipe(source).expect("recipe parses");
    // `base` is produced but never declared, so it becomes an implicit intermediate.
    let base = recipe
        .resources
        .iter()
        .find(|r| r.symbol == "base")
        .expect("intermediate registered");
    assert_eq!(base.kind, ResourceKind::Intermediate);
    // The producing operation gains an Output binding for it.
    let saute = recipe
        .operations
        .iter()
        .find(|o| o.symbol == "saute")
        .unwrap();
    assert!(
        saute
            .bindings
            .iter()
            .any(|b| b.resource == "base" && b.role == BindingRole::Output)
    );
    // Only one intermediate resource is added even though `base` is referenced twice.
    assert_eq!(
        recipe
            .resources
            .iter()
            .filter(|r| r.symbol == "base")
            .count(),
        1
    );
}

#[test]
fn produces_reuses_declared_material() {
    use culinograph_core::ResourceKind;
    let source = r#"culinograph 0.3; recipe stew { title "Stew";
        ingredient onion measured by count { quantity 1 count; }
        material base measured by mass { }
        process cook {
            operation saute does heat { input [onion]; produces base; duration 5 min; }
        }
    }"#;
    let recipe = parse_recipe(source).expect("recipe parses");
    let matches: Vec<_> = recipe
        .resources
        .iter()
        .filter(|r| r.symbol == "base")
        .collect();
    // The declared material is used as-is; no duplicate intermediate is created.
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].kind, ResourceKind::Material);
}

#[test]
fn lossless_parse_keeps_exact_source() {
    let source = "culinograph 0.3;\n// before\nrecipe bread { title \"Bread\"; /* after */ }\n";
    let parsed = parse_lossless(source).expect("both layers parse");
    assert_eq!(parsed.syntax.round_trip(), source);
}

#[test]
fn lossless_edit_reprojects_semantics_without_formatting_rest() {
    let source = "culinograph 0.3;\nrecipe bread {\n  title \"Bread\"; // retained\n}\n";
    let parsed = parse_lossless(source).unwrap();
    let start = source.find("Bread\"").unwrap();
    let edited = parsed
        .edit(&[TextEdit::replace(
            TextRange::new(start, start + 5),
            "Baguette",
        )])
        .unwrap();
    let reparsed = parse_recipe(edited.syntax.source()).unwrap();
    assert_eq!(reparsed.title, "Baguette");
    assert!(edited.syntax.source().contains("// retained"));
    assert!(edited.syntax.source().contains("title \"Baguette\""));
}
