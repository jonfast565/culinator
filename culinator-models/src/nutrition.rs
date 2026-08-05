use culinator_core::{
    Dimension, IngredientDensity, Quantity, Resource, ResourceKind, Serving, Value,
};
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
    /// Prefer Foundation/SR/Survey only (default). Set false to include branded UPC rows.
    #[serde(default = "default_exclude_branded")]
    pub exclude_branded: bool,
}

fn default_fuzzy_limit() -> usize {
    5
}

fn default_exclude_branded() -> bool {
    true
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

/// Build a matching query from a resource's display name / symbol plus size and
/// state cues. Variant group labels are omitted — they are alternative-set tags,
/// not food descriptors.
pub fn ingredient_match_query(resource: &Resource) -> String {
    let mut parts = Vec::new();
    if let Some(name) = resource_display_name(resource) {
        parts.push(name);
    } else {
        parts.push(resource.symbol.replace('_', " "));
    }
    if let Some(size) = resource.size.as_deref().filter(|value| !value.is_empty()) {
        parts.push(size.to_owned());
    }
    if let Some(state) = resource_state(resource) {
        parts.push(state);
    }
    parts.join(" ")
}

fn resource_display_name(resource: &Resource) -> Option<String> {
    resource
        .properties
        .get("name")
        .and_then(|value| match value {
            Value::Text(text) | Value::Symbol(text) => Some(text.clone()),
            _ => None,
        })
}

fn resource_state(resource: &Resource) -> Option<String> {
    resource
        .properties
        .get("state")
        .and_then(|value| match value {
            Value::Text(text) | Value::Symbol(text) => Some(text.clone()),
            _ => None,
        })
}

/// Resolve an ingredient quantity to grams: mass units pass through; volume uses
/// built-in densities; common count units use typical piece weights.
pub fn resource_mass_grams(resource: &Resource) -> Option<f64> {
    for key in ["mass", "quantity"] {
        if let Some(value) = resource.properties.get(key)
            && let Some(grams) = value_mass_grams_for_resource(value, resource)
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

fn value_mass_grams_for_resource(value: &Value, resource: &Resource) -> Option<f64> {
    let Value::Quantity(quantity) = value else {
        return None;
    };
    if let Some(grams) = quantity.as_grams() {
        return Some(grams);
    }
    let hint = resource_display_name(resource)
        .unwrap_or_else(|| resource.symbol.replace('_', " "));
    match quantity.dimension {
        Dimension::Volume => volume_to_grams(quantity, &hint),
        Dimension::Count => count_to_grams(quantity, resource, &hint),
        _ => None,
    }
}

fn volume_to_grams(quantity: &Quantity, ingredient_hint: &str) -> Option<f64> {
    let density = density_for_ingredient(ingredient_hint)?;
    quantity
        .to_mass(density)
        .ok()
        .and_then(|mass| mass.as_grams())
}

fn density_for_ingredient(hint: &str) -> Option<f64> {
    let registry = IngredientDensity::new();
    let normalized = normalize_ingredient_name(hint);
    if let Some(density) = registry.density_g_per_ml(&normalized) {
        return Some(density);
    }
    // Try progressively shorter suffixes / aliases.
    for alias in synonym_expansions(&normalized) {
        if let Some(density) = registry.density_g_per_ml(&alias) {
            return Some(density);
        }
    }
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    for start in 0..tokens.len() {
        let slice = tokens[start..].join(" ");
        if let Some(density) = registry.density_g_per_ml(&slice) {
            return Some(density);
        }
        for alias in synonym_expansions(&slice) {
            if let Some(density) = registry.density_g_per_ml(&alias) {
                return Some(density);
            }
        }
    }
    None
}

fn count_to_grams(quantity: &Quantity, resource: &Resource, ingredient_hint: &str) -> Option<f64> {
    let unit = quantity.unit.to_ascii_lowercase();
    let count = match unit.as_str() {
        "dozen" | "dozens" => quantity.value * 12.0,
        _ => quantity.value,
    };
    if count <= 0.0 {
        return None;
    }
    let piece = piece_weight_grams(ingredient_hint, resource.size.as_deref(), &unit)?;
    Some(count * piece)
}

fn piece_weight_grams(hint: &str, size: Option<&str>, unit: &str) -> Option<f64> {
    let normalized = normalize_ingredient_name(hint);
    let size = size.map(|value| value.to_ascii_lowercase());

    if matches!(unit, "clove" | "cloves")
        || normalized.contains("garlic") && matches!(unit, "clove" | "cloves" | "count" | "each" | "piece" | "pieces")
    {
        return Some(3.0);
    }

    let key = primary_ingredient_key(&normalized);
    let base = match key.as_str() {
        "egg" | "eggs" => match size.as_deref() {
            Some("small") => 38.0,
            Some("medium") => 44.0,
            Some("jumbo") | Some("extra large") | Some("xl") => 63.0,
            _ => 50.0, // large default
        },
        "banana" | "bananas" => match size.as_deref() {
            Some("small") => 100.0,
            Some("large") => 136.0,
            _ => 118.0,
        },
        "onion" | "onions" | "white onion" | "yellow onion" | "red onion" => match size.as_deref()
        {
            Some("small") => 70.0,
            Some("large") => 150.0,
            _ => 110.0,
        },
        "green onion" | "green onions" | "scallion" | "scallions" | "spring onion" => 15.0,
        "tomato" | "tomatoes" => match size.as_deref() {
            Some("small") => 90.0,
            Some("large") => 180.0,
            _ => 123.0,
        },
        "potato" | "potatoes" => match size.as_deref() {
            Some("small") => 170.0,
            Some("large") => 300.0,
            _ => 213.0,
        },
        "carrot" | "carrots" => 61.0,
        "lemon" | "lemons" => 84.0,
        "lime" | "limes" => 67.0,
        "avocado" | "avocados" => 136.0,
        "jalapeno" | "jalapeño" | "jalapenos" | "chili" | "chilies" | "chilli" | "pepper"
        | "peppers" | "chili pepper" => 14.0,
        "garlic" => 3.0,
        "shallot" | "shallots" => 25.0,
        "egg yolk" | "yolk" | "yolks" => 18.0,
        "egg white" | "whites" => 33.0,
        _ => return None,
    };
    Some(base)
}

fn primary_ingredient_key(normalized: &str) -> String {
    // Prefer longer synonym keys first so "green onion" wins over "onion".
    let mut aliases = synonym_expansions(normalized);
    aliases.insert(0, normalized.to_owned());
    for candidate in &aliases {
        if matches!(
            candidate.as_str(),
            "egg"
                | "eggs"
                | "banana"
                | "bananas"
                | "onion"
                | "onions"
                | "white onion"
                | "yellow onion"
                | "red onion"
                | "green onion"
                | "green onions"
                | "scallion"
                | "scallions"
                | "tomato"
                | "tomatoes"
                | "potato"
                | "potatoes"
                | "carrot"
                | "carrots"
                | "lemon"
                | "lemons"
                | "lime"
                | "limes"
                | "avocado"
                | "avocados"
                | "jalapeno"
                | "garlic"
                | "shallot"
                | "shallots"
        ) {
            return candidate.clone();
        }
    }
    // Fall back to last content token (e.g. "fresh tomatoes" → "tomatoes").
    normalized
        .split_whitespace()
        .next_back()
        .unwrap_or(normalized)
        .to_owned()
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
    // Keep "whole" — USDA uses it as a real distinguisher (whole milk, whole egg).
    "cooked", "hass", "ground", "grated", "sliced", "peeled", "seeded", "boneless",
    "skinless", "unsalted", "salted", "extra", "virgin", "finely", "roughly", "about", "optional",
    "warm", "crushed", "for", "the", "pan", "plus", "more", "serving", "to", "taste", "divided",
    "room", "temperature", "softened", "melted", "cold", "hot", "dried", "canned", "packed",
    "firmly", "loosely", "thinly", "thickly", "into", "and", "or", "with", "from", "a", "an",
];

/// Cooking-name → USDA-leaning aliases used to expand FTS / similarity queries.
fn synonym_expansions(normalized: &str) -> Vec<String> {
    let mut expansions = Vec::new();
    let push = |expansions: &mut Vec<String>, alias: &str| {
        let alias = alias.to_owned();
        if alias != normalized && !expansions.contains(&alias) {
            expansions.push(alias);
        }
    };

    // Phrase-level aliases (check longer keys first via contains / equality).
    let pairs = [
        ("all purpose flour", "wheat flour"),
        ("all-purpose flour", "wheat flour"),
        ("ap flour", "wheat flour"),
        ("bread flour", "wheat flour"),
        ("green onion", "scallion"),
        ("green onions", "scallions"),
        ("scallion", "green onion"),
        ("scallions", "green onions"),
        ("spring onion", "green onion"),
        ("cilantro", "coriander leaves"),
        ("fresh cilantro", "coriander leaves"),
        ("coriander", "coriander leaves"),
        ("baking powder", "leavening agents baking powder"),
        ("instant yeast", "yeast bakers"),
        ("active dry yeast", "yeast bakers"),
        ("dry yeast", "yeast bakers"),
        ("yeast", "yeast bakers"),
        ("olive oil", "oil olive"),
        ("extra virgin olive oil", "oil olive"),
        ("vegetable oil", "oil vegetable"),
        ("canola oil", "oil canola"),
        ("sesame oil", "oil sesame"),
        ("heavy cream", "cream fluid heavy whipping"),
        ("whipping cream", "cream fluid heavy whipping"),
        ("whole milk", "milk whole"),
        ("warm milk", "milk whole"),
        ("butter for the pan", "butter"),
        ("butter or oil for the pan", "butter"),
        ("kosher salt", "salt table"),
        ("sea salt", "salt table"),
        ("table salt", "salt table"),
        ("salt", "salt table"),
        ("bell pepper", "peppers sweet"),
        ("red pepper", "peppers sweet red"),
        ("chili pepper", "peppers hot chili"),
        ("jalapeno", "peppers jalapeno"),
        ("jalapeño", "peppers jalapeno"),
        ("ground cumin", "spices cumin seed"),
        ("ground ginger", "spices ginger"),
        ("ground cinnamon", "spices cinnamon"),
        ("brown sugar", "sugars brown"),
        ("powdered sugar", "sugars powdered"),
        ("confectioners sugar", "sugars powdered"),
        ("soy sauce", "sauce soy"),
        ("tomato paste", "tomato products paste"),
        ("canned tomatoes", "tomatoes red ripe canned"),
        ("greek yogurt", "yogurt greek"),
        ("parmesan", "cheese parmesan"),
        ("mozzarella", "cheese mozzarella"),
        ("cheddar", "cheese cheddar"),
        ("feta", "cheese feta"),
        ("arborio rice", "rice white"),
        ("red lentils", "lentils"),
        ("bulgur", "bulgur"),
        ("chickpeas", "chickpeas garbanzo beans"),
        ("garbanzo beans", "chickpeas"),
        ("beef broth", "soup beef broth"),
        ("chicken broth", "soup chicken broth"),
        ("vegetable broth", "soup vegetable broth"),
    ];

    for (from, to) in pairs {
        if normalized == from || normalized.ends_with(&format!(" {from}")) || normalized.contains(from)
        {
            push(&mut expansions, to);
            push(&mut expansions, from);
        }
    }

    // Token-level aliases.
    for token in normalized.split_whitespace() {
        match token {
            "scallion" | "scallions" => push(&mut expansions, "green onion"),
            "cilantro" => {
                push(&mut expansions, "coriander");
                push(&mut expansions, "coriander leaves");
            }
            "flour" => push(&mut expansions, "wheat flour"),
            "oil" => push(&mut expansions, "oil vegetable"),
            "butter" => push(&mut expansions, "butter salted"),
            "milk" => push(&mut expansions, "milk whole"),
            "egg" | "eggs" => push(&mut expansions, "egg whole"),
            "banana" | "bananas" => push(&mut expansions, "bananas raw"),
            "tomato" | "tomatoes" => push(&mut expansions, "tomatoes red ripe"),
            "onion" | "onions" => push(&mut expansions, "onions raw"),
            "garlic" => push(&mut expansions, "garlic raw"),
            "potato" | "potatoes" => push(&mut expansions, "potatoes flesh and skin"),
            "avocado" | "avocados" => push(&mut expansions, "avocados raw"),
            "pepper" | "peppers" => push(&mut expansions, "peppers sweet"),
            "chili" | "chilies" | "chilli" => push(&mut expansions, "peppers hot chili"),
            "yeast" => push(&mut expansions, "yeast bakers"),
            "salt" => push(&mut expansions, "salt table"),
            "sugar" => push(&mut expansions, "sugars granulated"),
            "cumin" => push(&mut expansions, "spices cumin seed"),
            "cinnamon" => push(&mut expansions, "spices cinnamon"),
            "nutmeg" => push(&mut expansions, "spices nutmeg"),
            "ginger" => push(&mut expansions, "spices ginger"),
            "paprika" => push(&mut expansions, "spices paprika"),
            "rice" => push(&mut expansions, "rice white"),
            "lentils" => push(&mut expansions, "lentils raw"),
            "oats" => push(&mut expansions, "oats regular"),
            "yogurt" => push(&mut expansions, "yogurt plain"),
            "mayonnaise" | "mayo" | "aioli" => push(&mut expansions, "salad dressing mayonnaise"),
            _ => {}
        }
    }

    expansions
}

pub fn normalize_ingredient_name(name: &str) -> String {
    name.to_lowercase()
        .split(|character: char| !character.is_alphanumeric())
        .filter(|word| !word.is_empty() && !INGREDIENT_STOP_WORDS.contains(word))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Content tokens for an ingredient name, with synonym expansions appended.
pub fn ingredient_query_tokens(name: &str) -> Vec<String> {
    let normalized = normalize_ingredient_name(name);
    let mut tokens: Vec<String> = normalized
        .split_whitespace()
        .filter(|word| word.len() > 1)
        .map(|word| word.to_owned())
        .collect();
    if tokens.is_empty() {
        tokens = name
            .split(|character: char| !character.is_alphanumeric())
            .filter(|word| !word.is_empty())
            .map(|word| word.to_ascii_lowercase())
            .collect();
    }
    for expansion in synonym_expansions(&normalized) {
        for token in expansion.split_whitespace() {
            let token = token.to_owned();
            if token.len() > 1 && !tokens.iter().any(|existing| existing == &token) {
                tokens.push(token);
            }
        }
    }
    tokens
}

/// Primary FTS query using AND of core tokens (synonym-aware).
pub fn fts_query_from_ingredient(name: &str) -> String {
    fts_queries_from_ingredient(name)
        .into_iter()
        .next()
        .unwrap_or_default()
}

/// Ordered FTS strategies: AND of primary tokens, then OR fallback, then synonym OR.
pub fn fts_queries_from_ingredient(name: &str) -> Vec<String> {
    let normalized = normalize_ingredient_name(name);
    let primary: Vec<&str> = normalized
        .split_whitespace()
        .filter(|word| word.len() > 2 && !INGREDIENT_STOP_WORDS.contains(word))
        .collect();
    let mut queries = Vec::new();
    if !primary.is_empty() {
        queries.push(primary.join(" AND "));
        if primary.len() > 1 {
            queries.push(primary.join(" OR "));
        }
    }
    let expanded = ingredient_query_tokens(name);
    if !expanded.is_empty() {
        let or_expanded = expanded.join(" OR ");
        if !queries.iter().any(|query| query == &or_expanded) {
            queries.push(or_expanded);
        }
    }
    if queries.is_empty() {
        let fallback = name
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .collect::<Vec<_>>()
            .join(" OR ");
        if !fallback.is_empty() {
            queries.push(fallback);
        }
    }
    queries
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
    if left_tokens.is_empty() || right_tokens.is_empty() {
        return 0.0;
    }
    let matches = left_tokens
        .iter()
        .filter(|token| right_tokens.iter().any(|other| tokens_match(token, other)))
        .count() as f64;
    // F1 over tokens: requiring both recall (query covered) and precision
    // (description not full of unrelated product words). Asymmetric recall-only
    // scoring made "butter" look perfect against "butter flavored popcorn".
    let recall = matches / left_tokens.len() as f64;
    let precision = matches / right_tokens.len() as f64;
    if recall + precision == 0.0 {
        0.0
    } else {
        2.0 * recall * precision / (recall + precision)
    }
}

pub fn string_similarity(left: &str, right: &str) -> f64 {
    let left_norm = normalize_ingredient_name(left);
    let right_norm = normalize_ingredient_name(right);
    if left_norm == right_norm {
        return 1.0;
    }
    // Also compare against synonym-expanded forms.
    let left_variants = {
        let mut variants = synonym_expansions(&left_norm);
        variants.insert(0, left_norm.clone());
        variants
    };
    let mut best = 0.0_f64;
    for left_variant in &left_variants {
        if left_variant == &right_norm {
            return 1.0;
        }
        // Prefix near-match only when the description is a short USDA-style
        // qualifier tail ("milk 3 25 milkfat"), not a long product title that
        // merely begins with the cooking name ("butter flavored gourmet…").
        if !left_variant.is_empty() && right_norm.starts_with(left_variant) {
            let rest = right_norm[left_variant.len()..].trim();
            let rest_tokens = rest.split_whitespace().count();
            if rest.is_empty() {
                best = best.max(0.95);
            } else if rest_tokens <= 2 {
                // e.g. "avocados raw" after light qualifier noise — not
                // "butter flavored gourmet popcorn butter".
                best = best.max(0.88);
            }
        } else if !right_norm.is_empty()
            && left_variant.starts_with(&right_norm)
            && right_norm.len() >= 3
        {
            best = best.max(0.85);
        }
        let left_trigrams = trigrams(left_variant);
        let right_trigrams = trigrams(&right_norm);
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
        let overlap = token_overlap_score(left_variant, &right_norm);
        best = best.max(trigram_score.max(overlap));
    }
    best
}

/// True for USDA Branded Foods rows (retail UPC products).
pub fn is_branded_food(data_type: &str) -> bool {
    let lowered = data_type.to_ascii_lowercase().replace([' ', '-'], "_");
    matches!(lowered.as_str(), "branded_food" | "branded")
}

fn data_type_bonus(data_type: &str) -> f64 {
    let lowered = data_type.to_ascii_lowercase().replace([' ', '-'], "_");
    match lowered.as_str() {
        "foundation_food" | "foundation" => 0.22,
        "sr_legacy_food" | "sr_legacy" => 0.18,
        "survey_fndds_food" | "survey_fndds" | "fndds" => 0.08,
        "branded_food" | "branded" => -0.45,
        _ => 0.0,
    }
}

fn match_quality_bonus(query: &str, result: &NutritionSearchResult) -> f64 {
    let query_norm = normalize_ingredient_name(query);
    let desc_norm = normalize_ingredient_name(&result.description);
    let mut bonus = data_type_bonus(&result.data_type);

    if !query_norm.is_empty() && desc_norm == query_norm {
        bonus += 0.25;
    } else if !query_norm.is_empty() && desc_norm.starts_with(&query_norm) {
        let rest_tokens = desc_norm[query_norm.len()..].trim().split_whitespace().count();
        // Short USDA tails only — not "butter flavored gourmet popcorn…".
        if rest_tokens <= 2 {
            bonus += 0.2;
        }
    } else if !query_norm.is_empty()
        && query_norm.split_whitespace().count() >= 2
        && desc_norm.contains(&query_norm)
    {
        // Multi-word phrase containment only (avoid "egg" ⊂ "egg beaters…").
        bonus += 0.08;
    }

    let query_tokens: Vec<&str> = query_norm.split_whitespace().collect();
    let desc_tokens: Vec<&str> = desc_norm.split_whitespace().collect();
    if !query_tokens.is_empty()
        && query_tokens
            .iter()
            .all(|token| desc_tokens.iter().any(|other| tokens_match(token, other)))
    {
        bonus += 0.1;
        // Extra unmatched description tokens → manufactured / composite product.
        let matched = query_tokens
            .iter()
            .filter(|token| desc_tokens.iter().any(|other| tokens_match(token, other)))
            .count();
        let extra = desc_tokens.len().saturating_sub(matched.saturating_add(1));
        if extra > 0 {
            bonus -= 0.07 * extra as f64;
        }
    }

    if result.brand_owner.as_deref().is_some_and(|brand| !brand.is_empty()) {
        bonus -= 0.12;
    }

    // Long multi-ingredient branded descriptions are usually poor generic matches.
    let comma_count = result.description.matches(',').count();
    if comma_count >= 3 && is_branded_food(&result.data_type) {
        bonus -= 0.08;
    }

    bonus
}

pub fn rank_fuzzy_matches(query: &str, results: &[NutritionSearchResult]) -> Vec<FuzzyFoodMatch> {
    let mut ranked: Vec<FuzzyFoodMatch> = results
        .iter()
        .map(|result| {
            let base = string_similarity(query, &result.description);
            // Allow quality bonuses to exceed 1.0 so Foundation/SR Legacy can
            // outrank branded rows with equally perfect token overlap.
            let score = (base + match_quality_bonus(query, result)).max(0.0);
            FuzzyFoodMatch {
                score,
                result: result.clone(),
            }
        })
        .collect();
    ranked.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                data_type_bonus(&right.result.data_type)
                    .partial_cmp(&data_type_bonus(&left.result.data_type))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| left.result.fdc_id.cmp(&right.result.fdc_id))
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
