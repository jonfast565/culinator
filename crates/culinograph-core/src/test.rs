use super::*;
use std::collections::BTreeMap;

fn ingredient(symbol: &str, pct: f64, reference: bool, flour: bool, water_fraction: f64) -> FormulaIngredient {
    FormulaIngredient { id: Uuid::new_v4(), symbol: symbol.into(), name: symbol.into(), stage: "final".into(), basis: FormulaBasis::ReferencePercent, percentage: Some(pct), mass_grams: None, is_reference: reference, is_flour: flour, water_fraction, scalable: true, properties: BTreeMap::new() }
}

#[test]
fn named_type_has_no_arguments() { assert!(TypeRef::named("Mass").arguments.is_empty()); }

#[test]
fn empty_recipe_book_has_expected_type() {
    let book = RecipeBook::empty("bread", "Bread", "0.3");
    assert_eq!(book.declared_type.name, "RecipeBook");
    assert!(book.recipes.is_empty());
}

#[test]
fn solves_reference_formula() {
    let formula = Formula { id: Uuid::new_v4(), recipe_id: None, symbol: "bread".into(), name: "Bread".into(), basis: FormulaBasis::ReferencePercent, ingredients: vec![ingredient("flour",100.0,true,true,0.0), ingredient("water",70.0,false,false,1.0), ingredient("salt",2.0,false,false,0.0)], properties: BTreeMap::new() };
    let result = formula.solve_for_target_mass(1720.0).expect("formula solves");
    assert!((result.reference_mass_grams - 1000.0).abs() < 0.001);
    assert!((result.hydration_percent - 70.0).abs() < 0.001);
}

#[test]
fn converts_weights_to_total_percentages() {
    let mut a=ingredient("a",100.0,true,false,0.0); a.mass_grams=Some(600.0);
    let mut b=ingredient("b",50.0,false,false,0.0); b.mass_grams=Some(400.0);
    let formula=Formula{id:Uuid::new_v4(),recipe_id:None,symbol:"x".into(),name:"X".into(),basis:FormulaBasis::ReferencePercent,ingredients:vec![a,b],properties:BTreeMap::new()};
    let result=formula.weights_to_percentages(PercentageView::Total).expect("conversion succeeds");
    assert!((result.lines[0].percentage.expect("percentage")-60.0).abs()<0.001);
}
