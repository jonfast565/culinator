use crate::{
    ApiError, ServiceState,
    models::{TranslateRecipeImagesRequest, UpdateImportSettingsRequest},
};
use axum::{Json, extract::State};
use culinator_models::{PublicImportSettings, RecipeImportRequest, RecipeImportResult};

pub async fn get_settings(
    State(state): State<ServiceState>,
) -> Result<Json<PublicImportSettings>, ApiError> {
    let value = state.imports().settings()?;
    Ok(Json(PublicImportSettings::from(&value)))
}
pub async fn update_settings(
    State(state): State<ServiceState>,
    Json(request): Json<UpdateImportSettingsRequest>,
) -> Result<Json<PublicImportSettings>, ApiError> {
    let current = state.imports().settings()?;
    let value = request.merge(current);
    state.imports().save_settings(&value)?;
    Ok(Json(PublicImportSettings::from(&value)))
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
#[cfg(test)]
mod test;
