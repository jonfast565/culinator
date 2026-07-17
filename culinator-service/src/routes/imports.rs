use crate::{
    ApiError, ServiceState,
    models::{StructuredImportRequest, TranslateRecipeImagesRequest, UpdateImportSettingsRequest},
};
use axum::{Json, extract::State};
use culinator_models::{ImportDraft, RecipeImportRequest, RecipeImportResult, StructuredInput};

pub async fn get_settings(
    State(state): State<ServiceState>,
) -> Result<Json<culinator_models::PublicImportSettings>, ApiError> {
    Ok(Json(state.imports().public_settings()?))
}
pub async fn update_settings(
    State(state): State<ServiceState>,
    Json(request): Json<UpdateImportSettingsRequest>,
) -> Result<Json<culinator_models::PublicImportSettings>, ApiError> {
    let current = state.imports().settings()?;
    let value = request.merge(current);
    state.imports().save_settings(&value)?;
    Ok(Json(state.imports().public_settings()?))
}
pub async fn translate(
    State(state): State<ServiceState>,
    Json(request): Json<TranslateRecipeImagesRequest>,
) -> Result<Json<RecipeImportResult>, ApiError> {
    let result = state
        .imports()
        .translate(RecipeImportRequest {
            images: request.images,
            target_language: request.target_language,
            recipe_book_title: request.recipe_book_title,
        })
        .await?;
    Ok(Json(result))
}

pub async fn import_structured(
    State(state): State<ServiceState>,
    Json(request): Json<StructuredImportRequest>,
) -> Result<Json<ImportDraft>, ApiError> {
    let draft = state.imports().import_structured(StructuredInput {
        format: request.format,
        content: request.content,
    })?;
    Ok(Json(draft))
}
#[cfg(test)]
mod test;
