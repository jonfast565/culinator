use super::*;
use culinograph_core::*;
use std::collections::BTreeMap;
#[tokio::test] async fn calculate_rejects_empty_formula() { let formula=Formula{id:uuid::Uuid::new_v4(),recipe_id:None,symbol:"x".into(),name:"X".into(),basis:FormulaBasis::ReferencePercent,ingredients:vec![],properties:BTreeMap::new()}; assert!(calculate(Json(FormulaCalculationRequest{formula,target_mass_grams:100.0})).await.is_err()); }
