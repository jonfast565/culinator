use crate::{ApplicationError, RecipeSchedule, ScheduleOptions};
use culinograph_core::Recipe;

pub trait RecipeScheduler: Send + Sync {
    fn schedule(
        &self,
        recipe: &Recipe,
        options: &ScheduleOptions,
    ) -> Result<RecipeSchedule, ApplicationError>;
}

#[cfg(test)]
mod test;
