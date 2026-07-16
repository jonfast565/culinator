use crate::{
    ApplicationError, ExecutionRepository, NewRecipeTry, NewTryObservation, RecipeRepository,
    RecipeTryDocument, RecipeTrySummary, ScheduleOptions, ScheduleService, UpdateRecipeTry,
    UpdateTryOperation,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct KitchenService {
    executions: Arc<dyn ExecutionRepository>,
    recipes: Arc<dyn RecipeRepository>,
    schedules: ScheduleService,
}

impl KitchenService {
    pub fn new(
        executions: Arc<dyn ExecutionRepository>,
        recipes: Arc<dyn RecipeRepository>,
        schedules: ScheduleService,
    ) -> Self {
        Self {
            executions,
            recipes,
            schedules,
        }
    }

    pub fn list_tries(&self, recipe_id: Uuid) -> Result<Vec<RecipeTrySummary>, ApplicationError> {
        self.executions.list_tries_for_recipe(recipe_id)
    }

    pub fn get(&self, try_id: Uuid) -> Result<RecipeTryDocument, ApplicationError> {
        self.executions
            .get_try(try_id)?
            .ok_or_else(|| ApplicationError::not_found("recipe try"))
    }

    pub fn start(
        &self,
        recipe_id: Uuid,
        input: NewRecipeTry,
    ) -> Result<RecipeTryDocument, ApplicationError> {
        let recipe = self
            .recipes
            .get_recipe(recipe_id)?
            .ok_or_else(|| ApplicationError::not_found("recipe"))?;
        let schedule = self
            .schedules
            .schedule_source(&recipe.source_text, &ScheduleOptions::default())?;
        if schedule.operations.is_empty() {
            return Err(ApplicationError::InvalidInput(
                "recipe has no operations to execute".to_owned(),
            ));
        }
        self.executions
            .start_try(recipe_id, &recipe.source_text, &schedule, input)
    }

    pub fn update(
        &self,
        try_id: Uuid,
        input: UpdateRecipeTry,
    ) -> Result<RecipeTryDocument, ApplicationError> {
        self.executions.update_try(try_id, input)
    }

    pub fn update_operation(
        &self,
        try_id: Uuid,
        operation_id: Uuid,
        input: UpdateTryOperation,
    ) -> Result<RecipeTryDocument, ApplicationError> {
        self.executions
            .update_try_operation(try_id, operation_id, input)
    }

    pub fn observe(
        &self,
        try_id: Uuid,
        input: NewTryObservation,
    ) -> Result<RecipeTryDocument, ApplicationError> {
        if input.property_path.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "observation property path cannot be empty".to_owned(),
            ));
        }
        self.executions.add_observation(try_id, input)
    }

    pub fn delete(&self, try_id: Uuid) -> Result<(), ApplicationError> {
        if self.executions.delete_try(try_id)? {
            Ok(())
        } else {
            Err(ApplicationError::not_found("recipe try"))
        }
    }
}

#[cfg(test)]
#[path = "kitchen_service/test.rs"]
mod test;
