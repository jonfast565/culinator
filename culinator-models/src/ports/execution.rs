use crate::{
    ApplicationError, NewRecipeTry, NewTryObservation, RecipeTryDocument, RecipeTrySummary,
    UpdateRecipeTry, UpdateTryOperation,
};
use uuid::Uuid;

pub trait ExecutionRepository: Send + Sync {
    fn list_tries_for_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<RecipeTrySummary>, ApplicationError>;
    fn get_try(&self, try_id: Uuid) -> Result<Option<RecipeTryDocument>, ApplicationError>;
    fn start_try(
        &self,
        recipe_id: Uuid,
        source_text: &str,
        schedule: &crate::RecipeSchedule,
        input: NewRecipeTry,
    ) -> Result<RecipeTryDocument, ApplicationError>;
    fn update_try(
        &self,
        try_id: Uuid,
        input: UpdateRecipeTry,
    ) -> Result<RecipeTryDocument, ApplicationError>;
    fn update_try_operation(
        &self,
        try_id: Uuid,
        operation_id: Uuid,
        input: UpdateTryOperation,
    ) -> Result<RecipeTryDocument, ApplicationError>;
    fn add_observation(
        &self,
        try_id: Uuid,
        input: NewTryObservation,
    ) -> Result<RecipeTryDocument, ApplicationError>;
    fn delete_try(&self, try_id: Uuid) -> Result<bool, ApplicationError>;
}

#[cfg(test)]
#[path = "execution/test.rs"]
mod test;
