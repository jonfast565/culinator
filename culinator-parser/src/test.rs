use super::*;

const RECIPE: &str = r#"culinator 0.3; recipe bread { title "Bread"; ingredient flour measured by mass { quantity 500 g; } process mix { operation combine does mix { input [flour]; duration 1 min; } } yield loaf measured by mass { mass 500 g; } }"#;
const BOOK: &str =
    r#"culinator 0.3; book favorites { title "Favorites"; recipe bread { title "Bread"; } }"#;

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
fn declarations_carry_spans_that_slice_their_own_source() {
    let source = r#"culinator 0.3;
recipe demo {
    title "Demo";
    ingredient flour measured by mass {
        quantity 500 g;
    }
    process mixing {
        prep chop onion into chopped_onion { duration 2 min; }
        operation combine does mix {
            input [flour, chopped_onion];
            duration 1 min;
        }
    }
}"#;
    let recipe = parse_recipe(source).expect("recipe parses");

    let flour = recipe
        .resources
        .iter()
        .find(|resource| resource.symbol == "flour")
        .expect("flour resource");
    let span = flour
        .span
        .as_ref()
        .expect("declared resources carry a span");
    let text = &source[span.start..span.end];
    assert!(text.starts_with("ingredient flour"), "got {text:?}");
    assert!(text.ends_with('}'), "span covers the whole block: {text:?}");

    let combine = recipe
        .operations
        .iter()
        .find(|operation| operation.symbol == "combine")
        .expect("combine operation");
    let span = combine.span.as_ref().expect("operations carry a span");
    let text = &source[span.start..span.end];
    assert!(text.starts_with("operation combine"), "got {text:?}");
    assert!(text.ends_with('}'), "got {text:?}");

    // `prep` desugars to an operation but must still point at the `prep` text,
    // so an editor patching by span rewrites what the author actually wrote.
    let chop = recipe
        .operations
        .iter()
        .find(|operation| operation.symbol == "chop_onion")
        .expect("prep-desugared operation");
    let span = chop.span.as_ref().expect("prep operations carry a span");
    assert!(source[span.start..span.end].starts_with("prep chop onion"));

    // Intermediates are synthesized from operation outputs, not written down,
    // so they have no source to point at.
    let intermediate = recipe
        .resources
        .iter()
        .find(|resource| resource.kind == culinator_core::ResourceKind::Intermediate)
        .expect("chopped_onion is registered as an intermediate");
    assert!(
        intermediate.span.is_none(),
        "synthesized resources have no span"
    );
}

/// The editor case: a recipe with one declaration still being typed. Recovery
/// must cost only the broken declaration.
#[test]
fn recovers_from_a_half_typed_declaration() {
    let source = r#"culinator 0.3;
recipe demo {
    title "Demo";
    ingredient flour measured by mass { quantity 500 g; }
    ingredient sugar measured by mass { quantity ; }
    ingredient salt measured by mass { quantity 5 g; }
    process mixing {
        operation combine does mix { input [flour]; duration 1 min; }
    }
}"#;
    let outcome = parse_recipe_recovering(source);
    let recipe = outcome.value.expect("a partial recipe is still produced");

    assert!(
        !outcome.diagnostics.is_empty(),
        "the broken quantity is reported"
    );
    // The declarations on both sides of the break survive.
    let symbols: Vec<&str> = recipe
        .resources
        .iter()
        .map(|resource| resource.symbol.as_str())
        .collect();
    assert!(symbols.contains(&"flour"), "got {symbols:?}");
    assert!(symbols.contains(&"salt"), "got {symbols:?}");
    // And so does everything after it, which is the whole point.
    assert_eq!(recipe.title, "Demo");
    assert!(
        recipe.operations.iter().any(|op| op.symbol == "combine"),
        "operations after the error still parse"
    );

    // The diagnostic points at real source, not a token index.
    let diagnostic = &outcome.diagnostics[0];
    assert!(diagnostic.span.end <= source.len());
    assert!(diagnostic.span.start < diagnostic.span.end);
}

/// A quote opened but not yet closed is what a live editor sees constantly.
#[test]
fn recovers_from_an_unterminated_string() {
    let source = r#"culinator 0.3;
recipe demo {
    title "Dem"#;
    let outcome = parse_recipe_recovering(source);
    assert!(
        outcome
            .diagnostics
            .iter()
            .any(|d| d.message.contains("unterminated string")),
        "got {:?}",
        outcome.diagnostics
    );
    // The partial title still projects, so the preview keeps its heading.
    let recipe = outcome.value.expect("partial recipe");
    assert_eq!(recipe.title, "Dem");
}

/// Recovery must never loop forever on input it cannot make progress on.
#[test]
fn recovery_terminates_on_garbage() {
    for source in [
        "culinator 0.3; recipe demo { ;;;;; }",
        "culinator 0.3; recipe demo { } } } }",
        "culinator 0.3; recipe demo { ingredient { { { ",
        "culinator 0.3; recipe demo {",
        "culinator 0.3; recipe demo { @@@ ### }",
    ] {
        let outcome = parse_recipe_recovering(source);
        assert!(
            !outcome.diagnostics.is_empty() || outcome.value.is_some(),
            "{source:?} produced neither a value nor a diagnostic"
        );
    }
}

/// Strict parsing must stay exactly as strict as before: anything that yields a
/// diagnostic is still an error for validation/scheduling/export.
#[test]
fn strict_parse_still_rejects_what_recovery_tolerates() {
    let source = r#"culinator 0.3;
recipe demo {
    title "Demo";
    ingredient sugar measured by mass { quantity ; }
}"#;
    assert!(parse_recipe(source).is_err(), "strict parse rejects");
    assert!(
        parse_recipe_recovering(source).value.is_some(),
        "recovering parse still yields a model"
    );
}

#[test]
fn produces_registers_implicit_intermediate() {
    use culinator_core::{BindingRole, ResourceKind};
    let source = r#"culinator 0.3; recipe stew { title "Stew";
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
    use culinator_core::ResourceKind;
    let source = r#"culinator 0.3; recipe stew { title "Stew";
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
    let source = "culinator 0.3;\n// before\nrecipe bread { title \"Bread\"; /* after */ }\n";
    let parsed = parse_lossless(source).expect("both layers parse");
    assert_eq!(parsed.syntax.round_trip(), source);
}

#[test]
fn lossless_edit_reprojects_semantics_without_formatting_rest() {
    let source = "culinator 0.3;\nrecipe bread {\n  title \"Bread\"; // retained\n}\n";
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
