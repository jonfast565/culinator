use crate::{
    ApiError, ServiceState,
    models::{ExportBookRequest, ExportRecipeRequest, ExportRecipeResponse},
};
use axum::{
    Json,
    extract::{Path, State},
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use uuid::Uuid;

pub async fn export_recipe(
    Path(id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<ExportRecipeRequest>,
) -> Result<Json<ExportRecipeResponse>, ApiError> {
    let id = Uuid::parse_str(&id).map_err(|e| ApiError::bad_request(e.to_string()))?;
    let bundle = state.exports().export_recipe(id, &request.options)?;
    Ok(Json(export_response(bundle)))
}

pub async fn export_book(
    Path(id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<ExportBookRequest>,
) -> Result<Json<ExportRecipeResponse>, ApiError> {
    let id = Uuid::parse_str(&id).map_err(|e| ApiError::bad_request(e.to_string()))?;
    let bundle = state.exports().export_book(id, &request.options)?;
    Ok(Json(export_response(bundle)))
}

fn export_response(bundle: culinator_models::RecipeExportBundle) -> ExportRecipeResponse {
    ExportRecipeResponse {
        file_name: bundle.file_name,
        media_type: bundle.media_type,
        archive_base64: STANDARD.encode(bundle.archive),
        files: bundle.files.into_iter().map(|f| f.path).collect(),
    }
}
#[cfg(test)]
mod test;
