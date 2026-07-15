use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use culinograph_application::NewRecipeBook;
use uuid::Uuid;

use crate::{
    error::ApiError,
    models::{MoveRecipeRequest, RecipeBookSummary, SaveRecipeBookRequest},
    state::ServiceState,
};

pub async fn list(
    State(state): State<ServiceState>,
) -> Result<Json<Vec<RecipeBookSummary>>, ApiError> {
    Ok(Json(
        state
            .books()
            .list()?
            .into_iter()
            .map(RecipeBookSummary::from)
            .collect(),
    ))
}

pub async fn create(
    State(state): State<ServiceState>,
    Json(request): Json<SaveRecipeBookRequest>,
) -> Result<(StatusCode, Json<RecipeBookSummary>), ApiError> {
    let created = state.books().create(request.into())?;
    Ok((StatusCode::CREATED, Json(created.into())))
}

pub async fn update(
    Path(id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<SaveRecipeBookRequest>,
) -> Result<Json<RecipeBookSummary>, ApiError> {
    Ok(Json(
        state.books().update(parse_id(&id)?, request.into())?.into(),
    ))
}

pub async fn delete(
    Path(id): Path<String>,
    State(state): State<ServiceState>,
) -> Result<StatusCode, ApiError> {
    state.books().delete(parse_id(&id)?)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn move_recipe(
    Path(recipe_id): Path<String>,
    State(state): State<ServiceState>,
    Json(request): Json<MoveRecipeRequest>,
) -> Result<StatusCode, ApiError> {
    let book_id = request.book_id.as_deref().map(parse_id).transpose()?;
    state
        .recipes()
        .move_to_book(parse_id(&recipe_id)?, book_id, request.position)?;
    Ok(StatusCode::NO_CONTENT)
}

impl From<SaveRecipeBookRequest> for NewRecipeBook {
    fn from(value: SaveRecipeBookRequest) -> Self {
        Self {
            title: value.title,
            symbol: value.symbol,
            description: value.description,
        }
    }
}

fn parse_id(value: &str) -> Result<Uuid, ApiError> {
    Uuid::parse_str(value).map_err(|_| ApiError::bad_request("Invalid UUID"))
}

#[cfg(test)]
mod test;
