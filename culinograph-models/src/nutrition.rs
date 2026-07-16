use culinograph_core::{Dimension, Resource, ResourceKind, Serving, Value};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::models::{FoodNutrientRecord, NutritionFacts, NutritionSearchResult};

/// USDA FoodData Central nutrient identifiers used for label aggregation.
pub const FDC_ENERGY_KCAL: i64 = 1008;
pub const FDC_PROTEIN: i64 = 1003;
pub const FDC_TOTAL_FAT: i64 = 1004;
pub const FDC_CARBOHYDRATE: i64 = 1005;
pub const FDC_FIBER: i64 = 1079;
pub const FDC_TOTAL_SUGARS: i64 = 2000;
pub const FDC_ADDED_SUGARS: i64 = 1235;
pub const FDC_SODIUM: i64 = 1093;
pub const FDC_CHOLESTEROL: i64 = 1253;
pub const FDC_SATURATED_FAT: i64 = 1258;
pub const FDC_TRANS_FAT: i64 = 1257;
pub const FDC_VITAMIN_D: i64 = 1106;
pub const FDC_CALCIUM: i64 = 1087;
pub const FDC_IRON: i64 = 1089;
pub const FDC_POTASSIUM: i64 = 1092;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceNutritionLink {
    pub recipe_id: uuid::Uuid,
    pub resource_symbol: String,
    pub fdc_id: i64,
    pub food_description: String,
    pub linked_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkResourceNutritionRequest {
    pub resource_symbol: String,
    pub fdc_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeIngredientNutrition {
    pub resource_symbol: String,
    pub resource_name: Option<String>,
    pub mass_grams: Option<f64>,
    pub fdc_id: Option<i64>,
    pub food_description: Option<String>,
    pub linked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculateRecipeNutritionRequest {
    pub servings_per_container: Option<f64>,
    pub serving_size: Option<String>,
    pub serving_size_grams: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeNutritionResult {
    pub facts: NutritionFacts,
    pub total_mass_grams: f64,
    pub linked_ingredient_count: usize,
    pub total_ingredient_count: usize,
    pub ingredients: Vec<RecipeIngredientNutrition>,
    pub warnings: Vec<String>,
}

pub fn resource_mass_grams(resource: &Resource) -> Option<f64> {
    for key in ["mass", "quantity"] {
        if let Some(value) = resource.properties.get(key)
            && let Some(grams) = value_mass_grams(value)
        {
            return Some(grams);
        }
    }
    None
}

pub fn value_mass_grams(value: &Value) -> Option<f64> {
    match value {
        Value::Quantity(quantity) if quantity.dimension == Dimension::Mass => {
            Some(match quantity.unit.as_str() {
                "kg" => quantity.value * 1000.0,
                "mg" => quantity.value / 1000.0,
                _ => quantity.value,
            })
        }
        _ => None,
    }
}

pub fn ingredient_resources(recipe: &culinograph_core::Recipe) -> Vec<&Resource> {
    recipe
        .resources
        .iter()
        .filter(|resource| resource.kind == ResourceKind::Ingredient)
        .collect()
}

pub fn default_serving_context(servings: &[Serving]) -> (f64, String, Option<f64>) {
    let default = servings
        .iter()
        .find(|serving| serving.is_default)
        .or_else(|| servings.first());
    let Some(serving) = default else {
        return (1.0, "1 serving".to_owned(), None);
    };
    let label = match &serving.amount {
        Value::Quantity(quantity) => format!("{} {}", quantity.value, quantity.unit),
        Value::Text(text) => text.clone(),
        Value::Symbol(symbol) => symbol.clone(),
        _ => serving.symbol.clone(),
    };
    (1.0, label, serving.mass_grams)
}

pub fn aggregate_nutrients(ingredients: &[(f64, Vec<FoodNutrientRecord>)]) -> BTreeMap<i64, f64> {
    let mut totals = BTreeMap::new();
    for (mass_grams, nutrients) in ingredients {
        if *mass_grams <= 0.0 {
            continue;
        }
        let factor = mass_grams / 100.0;
        for nutrient in nutrients {
            if let Some(amount) = nutrient.amount {
                *totals.entry(nutrient.nutrient_id).or_insert(0.0) += amount * factor;
            }
        }
    }
    totals
}

pub fn nutrients_to_facts(
    totals: &BTreeMap<i64, f64>,
    total_mass_grams: f64,
    servings_per_container: f64,
    serving_size: &str,
    serving_size_grams: Option<f64>,
) -> NutritionFacts {
    let servings = if servings_per_container > 0.0 {
        servings_per_container
    } else {
        1.0
    };
    let per_serving =
        |nutrient_id: i64| totals.get(&nutrient_id).copied().unwrap_or(0.0) / servings;
    let per_serving_mass = if total_mass_grams > 0.0 {
        total_mass_grams / servings
    } else {
        serving_size_grams.unwrap_or(0.0)
    };

    NutritionFacts {
        servings_per_container: servings,
        serving_size: serving_size.to_owned(),
        serving_size_grams: serving_size_grams.or(if per_serving_mass > 0.0 {
            Some(per_serving_mass)
        } else {
            None
        }),
        calories: per_serving(FDC_ENERGY_KCAL),
        total_fat_grams: per_serving(FDC_TOTAL_FAT),
        saturated_fat_grams: per_serving(FDC_SATURATED_FAT),
        trans_fat_grams: per_serving(FDC_TRANS_FAT),
        cholesterol_milligrams: per_serving(FDC_CHOLESTEROL),
        sodium_milligrams: per_serving(FDC_SODIUM),
        total_carbohydrate_grams: per_serving(FDC_CARBOHYDRATE),
        dietary_fiber_grams: per_serving(FDC_FIBER),
        total_sugars_grams: per_serving(FDC_TOTAL_SUGARS),
        added_sugars_grams: per_serving(FDC_ADDED_SUGARS),
        protein_grams: per_serving(FDC_PROTEIN),
        vitamin_d_micrograms: optional_nutrient(totals, FDC_VITAMIN_D, servings),
        calcium_milligrams: optional_nutrient(totals, FDC_CALCIUM, servings),
        iron_milligrams: optional_nutrient(totals, FDC_IRON, servings),
        potassium_milligrams: optional_nutrient(totals, FDC_POTASSIUM, servings),
    }
}

fn optional_nutrient(totals: &BTreeMap<i64, f64>, nutrient_id: i64, servings: f64) -> Option<f64> {
    totals
        .get(&nutrient_id)
        .copied()
        .map(|value| value / servings)
}

pub fn search_result_label(result: &NutritionSearchResult) -> String {
    if let Some(brand) = result
        .brand_owner
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        format!("{} ({})", result.description, brand)
    } else {
        result.description.clone()
    }
}

#[cfg(test)]
#[path = "nutrition/test.rs"]
mod test;
