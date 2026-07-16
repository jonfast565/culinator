use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use culinator_models::{NewRecipeTry, NewTryObservation, UpdateRecipeTry, UpdateTryOperation};
use uuid::Uuid;

use crate::{error::ApiError, state::ServiceState};

pub async fn list_for_recipe(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<Json<Vec<culinator_models::RecipeTrySummary>>, ApiError> {
    Ok(Json(state.kitchen().list_tries(parse_id(&recipe_id)?)?))
}

pub async fn start(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<NewRecipeTry>,
) -> Result<(StatusCode, Json<culinator_models::RecipeTryDocument>), ApiError> {
    let created = state.kitchen().start(parse_id(&recipe_id)?, request)?;
    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn get(
    Path(try_id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<Json<culinator_models::RecipeTryDocument>, ApiError> {
    Ok(Json(state.kitchen().get(parse_id(&try_id)?)?))
}

pub async fn update(
    Path(try_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<UpdateRecipeTry>,
) -> Result<Json<culinator_models::RecipeTryDocument>, ApiError> {
    Ok(Json(state.kitchen().update(parse_id(&try_id)?, request)?))
}

pub async fn update_operation(
    Path((try_id, operation_id)): Path<(String, String)>,
    State(state): State<ServiceState>,
    Json(request): Json<UpdateTryOperation>,
) -> Result<Json<culinator_models::RecipeTryDocument>, ApiError> {
    Ok(Json(state.kitchen().update_operation(
        parse_id(&try_id)?,
        parse_id(&operation_id)?,
        request,
    )?))
}

pub async fn add_observation(
    Path(try_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<NewTryObservation>,
) -> Result<Json<culinator_models::RecipeTryDocument>, ApiError> {
    Ok(Json(state.kitchen().observe(parse_id(&try_id)?, request)?))
}

pub async fn delete(
    Path(try_id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<StatusCode, ApiError> {
    state.kitchen().delete(parse_id(&try_id)?)?;
    Ok(StatusCode::NO_CONTENT)
}

fn parse_id(value: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(value).map_err(|_| ApiError::bad_request("Invalid UUID"))
}

#[cfg(test)]
#[path = "kitchen/test.rs"]
mod test;
