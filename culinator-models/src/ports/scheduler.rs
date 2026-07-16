use crate::{ApplicationError, RecipeSchedule, ScheduleOptions};
use culinator_core::Recipe;

pub trait RecipeScheduler: Send + Sync {
    fn schedule(
        &self,
        recipe: &Recipe,
        options: &ScheduleOptions,
    ) -> Result<RecipeSchedule, ApplicationError>;
}

#[cfg(test)]
mod test;
