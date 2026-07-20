use super::*;

const SEED_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../culinator-service/src/seed");

fn seeds() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(SEED_DIR).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e != "cg").unwrap_or(true) {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        out.push((name, std::fs::read_to_string(&path).unwrap()));
    }
    out.sort();
    out
}

/// Walk every scope and assert nodes are contiguous, non-overlapping, and start
/// where the scope starts. This is the invariant the whole design rests on: if
/// a byte belongs to no node, a structural edit can silently drop it.
fn assert_tiles(source: &str, nodes: &[OutlineNode], scope: TextRange, label: &str) {
    let mut cursor = scope.start;
    for node in nodes {
        assert_eq!(
            node.range.start, cursor,
            "{label}: gap or overlap before {:?}",
            node.keyword
        );
        assert!(
            node.code_range.start >= node.range.start
                && node.code_range.end == node.range.end
                && node.code_range.start <= node.code_range.end,
            "{label}: code_range not inside range for {:?}",
            node.keyword
        );
        // Leading trivia really is only trivia.
        let lead = &source[node.range.start..node.code_range.start];
        assert!(
            lead.trim().is_empty() || lead.trim_start().starts_with("//") || lead.contains("/*"),
            "{label}: non-trivia leading text {lead:?} before {:?}",
            node.keyword
        );
        if let Some(inner) = node.block_inner_range {
            assert_eq!(node.form, OutlineForm::Declaration);
            assert_tiles(source, &node.children, inner, label);
            // Whatever follows the last child inside the block is trivia.
            let tail_start = node.children.last().map_or(inner.start, |c| c.range.end);
            assert!(
                source[tail_start..inner.end].trim().is_empty(),
                "{label}: non-trivia tail in block of {:?}",
                node.keyword
            );
        }
        cursor = node.range.end;
    }
    assert!(
        cursor <= scope.end && source[cursor..scope.end].trim().is_empty(),
        "{label}: non-trivia tail {:?}",
        &source[cursor..scope.end]
    );
}

#[test]
fn every_seed_tiles_completely() {
    let all = seeds();
    assert_eq!(all.len(), 43, "all seeds covered");
    for (name, source) in &all {
        let outline = Outline::parse(source).expect("seed parses");
        assert_tiles(
            source,
            &outline.nodes,
            TextRange::new(0, source.len()),
            name,
        );
        // And the concatenation really is the source, trailing trivia aside.
        let joined: String = outline
            .nodes
            .iter()
            .map(|node| &source[node.range.as_range()])
            .collect();
        let end = outline.nodes.last().map_or(0, |node| node.range.end);
        assert_eq!(joined, source[..end], "{name} round-trips");
    }
}

#[test]
fn reads_declarations_statements_and_symbols() {
    let source = std::fs::read_to_string(format!("{SEED_DIR}/pizza_dough.cg")).unwrap();
    let outline = Outline::parse(&source).unwrap();

    assert_eq!(outline.nodes[0].keyword, "culinator");
    assert_eq!(outline.nodes[0].form, OutlineForm::Statement);

    let recipe = outline.recipe().expect("recipe declaration");
    assert_eq!(recipe.symbol.as_deref(), Some("pizza_dough"));
    assert_eq!(recipe.form, OutlineForm::Declaration);

    let title = recipe.child("title").expect("title");
    assert_eq!(
        &source[title.code_range.as_range()],
        r#"title "Pizza Dough";"#
    );
    assert_eq!(
        &source[title.value_range.expect("value").as_range()],
        r#""Pizza Dough""#
    );

    let flour = recipe
        .children
        .iter()
        .find(|node| node.symbol.as_deref() == Some("flour"))
        .expect("flour");
    assert_eq!(flour.keyword, "ingredient");
    assert_eq!(flour.form, OutlineForm::Declaration);
    assert_eq!(flour.indent, "    ");
    assert_eq!(
        &source[flour.header_range.expect("header").as_range()],
        "ingredient flour measured by mass "
    );
    assert_eq!(
        &source[flour.child("quantity").unwrap().code_range.as_range()],
        "quantity 400 g;"
    );
    // Nested: process -> operation -> its own fields.
    let mixing = recipe
        .children
        .iter()
        .find(|node| node.symbol.as_deref() == Some("mixing"))
        .expect("mixing process");
    let knead = mixing.child("operation").expect("operation");
    assert_eq!(knead.symbol.as_deref(), Some("knead"));
    assert_eq!(knead.indent, "        ");
    assert_eq!(knead.children_named("input").count(), 2);
}

/// The reason `range` includes leading trivia: deleting a declaration has to
/// take the comment that explains it, not orphan it above the next one.
#[test]
fn leading_trivia_attaches_to_the_following_node() {
    let source =
        std::fs::read_to_string(format!("{SEED_DIR}/baked_macaroni_and_cheese.cg")).unwrap();
    let outline = Outline::parse(&source).unwrap();
    let recipe = outline.recipe().unwrap();
    let salt = recipe
        .children
        .iter()
        .find(|node| node.symbol.as_deref() == Some("salt"))
        .expect("salt");
    let full = &source[salt.range.as_range()];
    assert!(
        full.contains("// \"1 tablespoon plus 1/2 teaspoon kosher salt, divided\""),
        "comment travels with the declaration: {full:?}"
    );
    assert!(&source[salt.code_range.as_range()].starts_with("ingredient salt"));
}

/// A property the semantic model drops entirely still has a span, which is the
/// whole point: it survives an edit to its sibling because nothing touches it.
#[test]
fn unmodelled_properties_are_visible() {
    let source =
        std::fs::read_to_string(format!("{SEED_DIR}/baked_macaroni_and_cheese.cg")).unwrap();
    let outline = Outline::parse(&source).unwrap();
    let butter = outline
        .recipe()
        .unwrap()
        .children
        .iter()
        .find(|node| node.symbol.as_deref() == Some("butter"))
        .expect("butter");
    let allergen = butter.child("allergen").expect("allergen statement");
    assert_eq!(&source[allergen.code_range.as_range()], "allergen milk;");
    assert_eq!(&source[allergen.value_range.unwrap().as_range()], "milk");
}

#[test]
fn splicing_one_statement_leaves_every_other_byte_alone() {
    let source =
        std::fs::read_to_string(format!("{SEED_DIR}/baked_macaroni_and_cheese.cg")).unwrap();
    let outline = Outline::parse(&source).unwrap();
    let macaroni = outline
        .recipe()
        .unwrap()
        .children
        .iter()
        .find(|node| node.symbol.as_deref() == Some("macaroni"))
        .expect("macaroni");
    let quantity = macaroni.child("quantity").expect("quantity");
    let edited = format!(
        "{}{}{}",
        &source[..quantity.code_range.start],
        "quantity 12 oz;",
        &source[quantity.code_range.end..]
    );
    assert!(edited.contains("quantity 12 oz;"));
    assert!(
        edited.contains("allergen milk;"),
        "unmodelled property kept"
    );
    assert!(
        edited.contains("// Butter is also divided"),
        "comments kept"
    );
    assert_eq!(
        edited.lines().count(),
        source.lines().count(),
        "no lines added or lost"
    );
}

/// Empty blocks and inline blocks both appear in the seeds.
#[test]
fn handles_inline_and_empty_blocks() {
    let source = "culinator 0.3;\nrecipe demo {\n    title \"D\";\n    material dough measured by mass { }\n    ingredient oil measured by volume { quantity 1 tbsp; }\n}\n";
    let outline = Outline::parse(source).unwrap();
    let recipe = outline.recipe().unwrap();
    let dough = recipe
        .children
        .iter()
        .find(|n| n.symbol.as_deref() == Some("dough"))
        .unwrap();
    assert!(dough.children.is_empty());
    assert_eq!(dough.form, OutlineForm::Declaration);

    let oil = recipe
        .children
        .iter()
        .find(|n| n.symbol.as_deref() == Some("oil"))
        .unwrap();
    let quantity = oil.child("quantity").unwrap();
    assert_eq!(&source[quantity.code_range.as_range()], "quantity 1 tbsp;");
    // An inline block gives its member no line indentation of its own.
    assert_eq!(quantity.indent, "");
    assert_tiles(
        source,
        &outline.nodes,
        TextRange::new(0, source.len()),
        "inline",
    );
}

/// A block-less `prep` is a statement, not a declaration — the outline reports
/// syntax and lets the caller decide what it means.
#[test]
fn block_less_prep_is_a_statement() {
    let source = "culinator 0.3;\nrecipe demo {\n    title \"D\";\n    process p { prep dice onion into diced; }\n}\n";
    let outline = Outline::parse(source).unwrap();
    let prep = outline
        .recipe()
        .unwrap()
        .child("process")
        .unwrap()
        .child("prep")
        .expect("prep");
    assert_eq!(prep.form, OutlineForm::Statement);
    assert_eq!(prep.symbol.as_deref(), Some("dice"));
}

#[test]
fn unbalanced_source_recovers_to_an_empty_outline() {
    let broken = "culinator 0.3;\nrecipe demo {\n    title \"D\";\n";
    assert!(Outline::parse(broken).is_err());
    let outline = Outline::parse_recovering(broken);
    assert!(outline.nodes.is_empty());
    assert_eq!(outline.source_len, broken.len());
}

/// Half-typed input still tiles — the bytes have to be accounted for even when
/// the statement has no terminator yet.
#[test]
fn unterminated_statement_still_tiles() {
    let source = "culinator 0.3;\nrecipe demo {\n    title \"D\";\n    quantity 5\n}\n";
    let outline = Outline::parse(source).unwrap();
    assert_tiles(
        source,
        &outline.nodes,
        TextRange::new(0, source.len()),
        "partial",
    );
    let last = outline.recipe().unwrap().children.last().unwrap();
    assert_eq!(last.keyword, "quantity");
    assert_eq!(last.form, OutlineForm::Statement);
}
