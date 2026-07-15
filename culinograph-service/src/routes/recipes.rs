use axum::{extract::{Path, State}, http::StatusCode, Json};
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::{
        CreateRecipeRequest, Diagnostic, RecipeDocument, RecipeOutline, RecipeSummary,
        SaveRecipeRequest, ValidateRequest, ValidationResult,
    },
    state::ServiceState,
};

pub async fn list(State(state): State<ServiceState>) -> Result<Json<Vec<RecipeSummary>>, ApiError> {
    Ok(Json(state.recipes().list()?.into_iter().map(RecipeSummary::from).collect()))
}

pub async fn get(
    Path(id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<Json<RecipeDocument>, ApiError> {
    Ok(Json(RecipeDocument::from(state.recipes().get(parse_id(&id)?)?)))
}

pub async fn create(
    State(state): State<ServiceState>,
    Json(request): Json<CreateRecipeRequest>,
) -> Result<(StatusCode, Json<RecipeDocument>), ApiError> {
    let book_id = request.book_id.as_deref().map(parse_id).transpose()?;
    let document = state.recipes().create(book_id)?;
    Ok((StatusCode::CREATED, Json(RecipeDocument::from(document))))
}

pub async fn save(
    Path(id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<SaveRecipeRequest>,
) -> Result<Json<RecipeDocument>, ApiError> {
    let document = state.recipes().save(parse_id(&id)?, &request.source_text)?;
    Ok(Json(RecipeDocument::from(document)))
}

pub async fn delete(
    Path(id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<StatusCode, ApiError> {
    state.recipes().delete(parse_id(&id)?)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn validate(
    State(state): State<ServiceState>,
    Json(request): Json<ValidateRequest>,
) -> Json<ValidationResult> {
    let report = state.recipes().validate_source(&request.source_text);
    Json(ValidationResult {
        valid: report.valid,
        diagnostics: report.diagnostics.into_iter().map(Diagnostic::from).collect(),
        outline: report.outline.map(RecipeOutline::from),
    })
}

fn parse_id(value: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(value).map_err(|_| ApiError::bad_request("Invalid UUID"))
}

#[cfg(test)]
mod test;
