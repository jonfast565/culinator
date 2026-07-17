use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use culinator_models::{
    AutoLinkRequest, CalculateRecipeNutritionRequest, FuzzyMatchRequest,
    LinkResourceNutritionRequest, SaveIngredientManualNutritionRequest, SaveRecipeNutritionRequest,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{error::ApiError, state::ServiceState};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

pub async fn search(
    Query(query): Query<SearchQuery>,
    State(state): State<ServiceState>,
) -> Result<Json<Vec<culinator_models::NutritionSearchResult>>, ApiError> {
    Ok(Json(state.nutrition().search_foods(&query.q, query.limit)?))
}

pub async fn fuzzy_match(
    State(state): State<ServiceState>,
    Json(request): Json<FuzzyMatchRequest>,
) -> Result<Json<Vec<culinator_models::FuzzyFoodMatch>>, ApiError> {
    Ok(Json(state.nutrition().fuzzy_match(request)?))
}

pub async fn get_state(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<Json<culinator_models::RecipeNutritionState>, ApiError> {
    Ok(Json(state.nutrition().get_state(parse_id(&recipe_id)?)?))
}

pub async fn save_recipe_nutrition(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<SaveRecipeNutritionRequest>,
) -> Result<Json<culinator_models::RecipeNutritionState>, ApiError> {
    Ok(Json(
        state
            .nutrition()
            .save_recipe_nutrition(parse_id(&recipe_id)?, request)?,
    ))
}

pub async fn save_manual_ingredient(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<SaveIngredientManualNutritionRequest>,
) -> Result<Json<culinator_models::IngredientManualNutrition>, ApiError> {
    Ok(Json(
        state
            .nutrition()
            .save_manual_ingredient(parse_id(&recipe_id)?, request)?,
    ))
}

pub async fn delete_manual_ingredient(
    Path((recipe_id, resource_symbol)): Path<(String, String)>,
    State(state): State<ServiceState>,
) -> Result<StatusCode, ApiError> {
    state
        .nutrition()
        .delete_manual_ingredient(parse_id(&recipe_id)?, &resource_symbol)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn auto_link(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<AutoLinkRequest>,
) -> Result<Json<culinator_models::AutoLinkResult>, ApiError> {
    Ok(Json(
        state
            .nutrition()
            .auto_link(parse_id(&recipe_id)?, request)?,
    ))
}

pub async fn list_links(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<Json<Vec<culinator_models::ResourceNutritionLink>>, ApiError> {
    Ok(Json(state.nutrition().list_links(parse_id(&recipe_id)?)?))
}

pub async fn link_resource(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<LinkResourceNutritionRequest>,
) -> Result<(StatusCode, Json<culinator_models::ResourceNutritionLink>), ApiError> {
    let linked = state
        .nutrition()
        .link_resource(parse_id(&recipe_id)?, request)?;
    Ok((StatusCode::CREATED, Json(linked)))
}

pub async fn unlink_resource(
    Path((recipe_id, resource_symbol)): Path<(String, String)>,
    State(state): State<ServiceState>,
) -> Result<StatusCode, ApiError> {
    state
        .nutrition()
        .unlink_resource(parse_id(&recipe_id)?, &resource_symbol)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn calculate(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<CalculateRecipeNutritionRequest>,
) -> Result<Json<culinator_models::RecipeNutritionResult>, ApiError> {
    Ok(Json(
        state
            .nutrition()
            .calculate(parse_id(&recipe_id)?, request)?,
    ))
}

pub async fn status(
    State(state): State<ServiceState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!({
        "catalogAvailable": state.nutrition().catalog_available(),
    })))
}

fn parse_id(value: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(value).map_err(|_| ApiError::bad_request("Invalid UUID"))
}

#[cfg(test)]
#[path = "nutrition/test.rs"]
mod test;
