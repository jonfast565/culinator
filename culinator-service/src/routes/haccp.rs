use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use culinator_models::{NewHaccpMonitoringRecord, NewHaccpPlan, SaveHaccpPlanRequest};
use uuid::Uuid;

use crate::{error::ApiError, state::ServiceState};

pub async fn list_for_recipe(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<Json<Vec<culinator_models::HaccpPlanSummary>>, ApiError> {
    Ok(Json(state.haccp().list_for_recipe(parse_id(&recipe_id)?)?))
}

pub async fn create(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<NewHaccpPlan>,
) -> Result<(StatusCode, Json<culinator_models::HaccpPlanDocument>), ApiError> {
    let created = state.haccp().create(parse_id(&recipe_id)?, request)?;
    Ok((StatusCode::CREATED, Json(created)))
}

pub async fn get(
    Path(plan_id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<Json<culinator_models::HaccpPlanDocument>, ApiError> {
    Ok(Json(state.haccp().get(parse_id(&plan_id)?)?))
}

pub async fn save(
    Path(plan_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<SaveHaccpPlanRequest>,
) -> Result<Json<culinator_models::HaccpPlanDocument>, ApiError> {
    Ok(Json(state.haccp().save(parse_id(&plan_id)?, request)?))
}

pub async fn delete(
    Path(plan_id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<StatusCode, ApiError> {
    state.haccp().delete(parse_id(&plan_id)?)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_monitoring_record(
    Path(ccp_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<NewHaccpMonitoringRecord>,
) -> Result<(StatusCode, Json<culinator_models::HaccpMonitoringRecord>), ApiError> {
    let created = state
        .haccp()
        .record_monitoring(parse_id(&ccp_id)?, request)?;
    Ok((StatusCode::CREATED, Json(created)))
}

fn parse_id(value: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(value).map_err(|_| ApiError::bad_request("Invalid UUID"))
}

#[cfg(test)]
mod test;
