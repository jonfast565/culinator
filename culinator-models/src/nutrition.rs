use culinator_core::{Resource, ResourceKind, Serving, Value};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
    pub manual: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngredientManualNutrition {
    pub recipe_id: uuid::Uuid,
    pub resource_symbol: String,
    pub facts_per_100g: NutritionFacts,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRecipeNutritionRequest {
    pub manual_override: bool,
    pub facts: Option<NutritionFacts>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveIngredientManualNutritionRequest {
    pub resource_symbol: String,
    pub facts_per_100g: NutritionFacts,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeNutritionState {
    pub recipe_id: uuid::Uuid,
    pub links: Vec<ResourceNutritionLink>,
    pub manual_ingredients: Vec<IngredientManualNutrition>,
    pub manual_override: bool,
    pub manual_facts: Option<NutritionFacts>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuzzyFoodMatch {
    pub result: NutritionSearchResult,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuzzyMatchRequest {
    pub query: String,
    #[serde(default = "default_fuzzy_limit")]
    pub limit: usize,
}

fn default_fuzzy_limit() -> usize {
    5
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoLinkRequest {
    #[serde(default = "default_min_score")]
    pub min_score: f64,
    #[serde(default)]
    pub dry_run: bool,
}

fn default_min_score() -> f64 {
    0.45
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngredientMatchSuggestion {
    pub resource_symbol: String,
    pub resource_name: Option<String>,
    pub best_match: Option<FuzzyFoodMatch>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoLinkResult {
    pub linked: Vec<ResourceNutritionLink>,
    pub skipped: Vec<String>,
    pub suggestions: Vec<IngredientMatchSuggestion>,
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
    pub manual_override: bool,
    pub calculated: bool,
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
        Value::Quantity(quantity) => quantity.as_grams(),
        _ => None,
    }
}

pub fn ingredient_resources(recipe: &culinator_core::Recipe) -> Vec<&Resource> {
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

const INGREDIENT_STOP_WORDS: &[&str] = &[
    "fresh", "diced", "chopped", "minced", "ripe", "large", "small", "medium", "organic", "raw",
    "cooked", "hass", "whole", "ground", "grated", "sliced", "peeled", "seeded", "boneless",
    "skinless", "unsalted", "salted", "extra", "virgin", "finely", "roughly", "about", "optional",
];

pub fn normalize_ingredient_name(name: &str) -> String {
    name.to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty() && !INGREDIENT_STOP_WORDS.contains(word))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn fts_query_from_ingredient(name: &str) -> String {
    let normalized = name.to_lowercase();
    let words: Vec<&str> = normalized
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| word.len() > 2 && !INGREDIENT_STOP_WORDS.contains(word))
        .collect();
    if words.is_empty() {
        name.split(|character: char| character.is_whitespace())
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>()
            .join(" OR ")
    } else {
        words.join(" OR ")
    }
}

fn trigrams(value: &str) -> BTreeSet<String> {
    let normalized = value.to_lowercase();
    let chars: Vec<char> = format!("  {normalized} ").chars().collect();
    if chars.len() < 3 {
        return BTreeSet::from([normalized]);
    }
    chars
        .windows(3)
        .map(|window| window.iter().collect::<String>())
        .collect()
}

fn tokens_match(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let left_stem = left.trim_end_matches('s');
    let right_stem = right.trim_end_matches('s');
    if left_stem.len() >= 3 && left_stem == right_stem {
        return true;
    }
    let shorter = left.len().min(right.len());
    if shorter >= 4 && (left.starts_with(right) || right.starts_with(left)) {
        return true;
    }
    false
}

fn token_overlap_score(left: &str, right: &str) -> f64 {
    let left_tokens: Vec<&str> = left.split_whitespace().collect();
    let right_tokens: Vec<&str> = right.split_whitespace().collect();
    if left_tokens.is_empty() {
        return 0.0;
    }
    let matches = left_tokens
        .iter()
        .filter(|token| right_tokens.iter().any(|other| tokens_match(token, other)))
        .count();
    matches as f64 / left_tokens.len() as f64
}

pub fn string_similarity(left: &str, right: &str) -> f64 {
    let left = normalize_ingredient_name(left);
    let right = normalize_ingredient_name(right);
    if left == right {
        return 1.0;
    }
    let left_trigrams = trigrams(&left);
    let right_trigrams = trigrams(&right);
    let trigram_score = if left_trigrams.is_empty() && right_trigrams.is_empty() {
        1.0
    } else {
        let intersection = left_trigrams.intersection(&right_trigrams).count();
        let union = left_trigrams.union(&right_trigrams).count();
        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    };
    trigram_score.max(token_overlap_score(&left, &right))
}

pub fn rank_fuzzy_matches(query: &str, results: &[NutritionSearchResult]) -> Vec<FuzzyFoodMatch> {
    let mut ranked: Vec<FuzzyFoodMatch> = results
        .iter()
        .map(|result| FuzzyFoodMatch {
            score: string_similarity(query, &result.description),
            result: result.clone(),
        })
        .collect();
    ranked.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ranked
}

pub fn manual_facts_to_nutrients(facts: &NutritionFacts) -> Vec<FoodNutrientRecord> {
    let pairs = [
        (FDC_ENERGY_KCAL, facts.calories),
        (FDC_PROTEIN, facts.protein_grams),
        (FDC_TOTAL_FAT, facts.total_fat_grams),
        (FDC_SATURATED_FAT, facts.saturated_fat_grams),
        (FDC_TRANS_FAT, facts.trans_fat_grams),
        (FDC_CHOLESTEROL, facts.cholesterol_milligrams),
        (FDC_SODIUM, facts.sodium_milligrams),
        (FDC_CARBOHYDRATE, facts.total_carbohydrate_grams),
        (FDC_FIBER, facts.dietary_fiber_grams),
        (FDC_TOTAL_SUGARS, facts.total_sugars_grams),
        (FDC_ADDED_SUGARS, facts.added_sugars_grams),
        (FDC_VITAMIN_D, facts.vitamin_d_micrograms.unwrap_or(0.0)),
        (FDC_CALCIUM, facts.calcium_milligrams.unwrap_or(0.0)),
        (FDC_IRON, facts.iron_milligrams.unwrap_or(0.0)),
        (FDC_POTASSIUM, facts.potassium_milligrams.unwrap_or(0.0)),
    ];
    pairs
        .into_iter()
        .filter(|(_, amount)| *amount > 0.0)
        .map(|(nutrient_id, amount)| FoodNutrientRecord {
            id: None,
            fdc_id: 0,
            nutrient_id,
            amount: Some(amount),
            data_points: None,
            derivation_id: None,
            min: None,
            max: None,
            median: None,
        })
        .collect()
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
