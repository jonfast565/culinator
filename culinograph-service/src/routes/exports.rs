use axum::{extract::{Path, State}, Json};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use uuid::Uuid;
use crate::{models::{ExportRecipeRequest, ExportRecipeResponse}, ApiError, ServiceState};

pub async fn export_recipe(Path(id): Path<String>, State(state): State<ServiceState>, Json(request): Json<ExportRecipeRequest>) -> Result<Json<ExportRecipeResponse>, ApiError> {
    let id=Uuid::parse_str(&id).map_err(|e| ApiError::bad_request(e.to_string()))?;
    let bundle=state.exports().export_recipe(id,&request.options)?;
    Ok(Json(ExportRecipeResponse { file_name:bundle.file_name, media_type:"application/zip", archive_base64:STANDARD.encode(bundle.archive), files:bundle.files.into_iter().map(|f|f.path).collect() }))
}
#[cfg(test)] mod test;
