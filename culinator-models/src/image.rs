use serde::{Deserialize, Serialize};

/// Metadata for a persisted recipe image (no bytes), used in list responses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeImageAsset {
    pub id: uuid::Uuid,
    pub recipe_id: uuid::Uuid,
    /// Stable token referenced from the `.cg` source (`image`/`photo`).
    pub handle: String,
    /// `"cover"` for the recipe hero, `"step"` for a per-operation photo.
    pub role: String,
    pub operation_symbol: Option<String>,
    pub media_type: String,
    pub file_name: Option<String>,
    pub byte_size: i64,
    pub created_at: String,
}

/// A recipe image with its base64-encoded bytes, used for single fetches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeImageData {
    pub asset: RecipeImageAsset,
    pub data_base64: String,
}

/// Request to upload (or replace) a recipe image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadRecipeImageRequest {
    /// Omit to have the server generate a stable handle.
    pub handle: Option<String>,
    pub role: String,
    pub operation_symbol: Option<String>,
    pub media_type: String,
    pub file_name: Option<String>,
    pub data_base64: String,
}
