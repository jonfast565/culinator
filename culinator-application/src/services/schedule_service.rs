use culinator_models::{
    ApplicationError, DocumentParser, RecipeSchedule, RecipeScheduler, ScheduleOptions,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ScheduleService {
    parser: Arc<dyn DocumentParser>,
    scheduler: Arc<dyn RecipeScheduler>,
}
impl ScheduleService {
    pub fn new(parser: Arc<dyn DocumentParser>, scheduler: Arc<dyn RecipeScheduler>) -> Self {
        Self { parser, scheduler }
    }
    pub fn schedule_source(
        &self,
        source: &str,
        options: &ScheduleOptions,
    ) -> Result<RecipeSchedule, ApplicationError> {
        let document = self.parser.parse_document(source)?;
        let recipe = match document {
            culinator_core::Document::Recipe { recipe } => recipe,
            culinator_core::Document::RecipeBook { .. } => {
                return Err(ApplicationError::InvalidInput(
                    "Schedule a recipe, not an entire recipe book".into(),
                ));
            }
        };
        self.scheduler.schedule(&recipe, options)
    }
}
#[cfg(test)]
mod test;
