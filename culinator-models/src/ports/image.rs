use crate::{ApplicationError, RecipeImageAsset, RecipeImageData, UploadRecipeImageRequest};
use uuid::Uuid;

/// Persistence for recipe images (recipe hero + per-step photos). Bytes live in
/// a side table keyed by `(recipe_id, handle)`, separate from the `.cg` source
/// which only carries the handle.
pub trait RecipeImageRepository: Send + Sync {
    fn list_recipe_images(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<RecipeImageAsset>, ApplicationError>;

    fn get_recipe_image(
        &self,
        recipe_id: Uuid,
        handle: &str,
    ) -> Result<Option<RecipeImageData>, ApplicationError>;

    fn upload_recipe_image(
        &self,
        recipe_id: Uuid,
        input: UploadRecipeImageRequest,
    ) -> Result<RecipeImageAsset, ApplicationError>;

    fn delete_recipe_image(&self, recipe_id: Uuid, handle: &str) -> Result<bool, ApplicationError>;
}
