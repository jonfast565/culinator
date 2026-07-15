use axum::{
    Json,
    extract::{Path, State},
};
use culinograph_core::{Formula, FormulaResult, PercentageConversion};
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::{FormulaCalculationRequest, FormulaRunRequest, PercentageRequest},
    state::ServiceState,
};

pub async fn calculate(
    State(state): State<ServiceState>,
    Json(request): Json<FormulaCalculationRequest>,
) -> Result<Json<FormulaResult>, ApiError> {
    Ok(Json(
        state
            .formulas()
            .calculate(&request.formula, request.target_mass_grams)?,
    ))
}

pub async fn percentages(
    State(state): State<ServiceState>,
    Json(request): Json<PercentageRequest>,
) -> Result<Json<PercentageConversion>, ApiError> {
    Ok(Json(
        state
            .formulas()
            .percentages(&request.formula, request.view)?,
    ))
}

pub async fn save(
    State(state): State<ServiceState>,
    Json(formula): Json<Formula>,
) -> Result<Json<Formula>, ApiError> {
    state.formulas().save(&formula)?;
    Ok(Json(formula))
}

pub async fn list_for_recipe(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<Json<Vec<Formula>>, ApiError> {
    Ok(Json(
        state.formulas().list_for_recipe(parse_id(&recipe_id)?)?,
    ))
}

pub async fn get(
    Path(formula_id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<Json<Formula>, ApiError> {
    Ok(Json(state.formulas().get(parse_id(&formula_id)?)?))
}

pub async fn calculate_and_record(
    Path(formula_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<FormulaRunRequest>,
) -> Result<Json<FormulaResult>, ApiError> {
    Ok(Json(state.formulas().calculate_and_record(
        parse_id(&formula_id)?,
        request.target_mass_grams,
    )?))
}

fn parse_id(value: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(value).map_err(|_| ApiError::bad_request("Invalid UUID"))
}

#[cfg(test)]
mod test;
