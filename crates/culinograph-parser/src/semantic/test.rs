use super::*;

#[test]
fn semantic_parser_ignores_block_comments() {
    let source = "culinograph 0.3; /* comment */ recipe bread { title \"Bread\"; }";
    let recipe = parse_semantic_recipe(source).expect("semantic recipe");
    assert_eq!(recipe.title, "Bread");
}
