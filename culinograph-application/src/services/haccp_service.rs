use crate::{
    ApplicationError, HaccpMonitoringRecord, HaccpPlanDocument, HaccpPlanSummary,
    HaccpRepository, NewHaccpMonitoringRecord, NewHaccpPlan, SaveHaccpPlanRequest,
};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct HaccpService {
    repository: Arc<dyn HaccpRepository>,
}

impl HaccpService {
    pub fn new(repository: Arc<dyn HaccpRepository>) -> Self {
        Self { repository }
    }

    pub fn list_for_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<HaccpPlanSummary>, ApplicationError> {
        self.repository.list_plans_for_recipe(recipe_id)
    }

    pub fn get(&self, plan_id: Uuid) -> Result<HaccpPlanDocument, ApplicationError> {
        self.repository
            .get_plan(plan_id)?
            .ok_or_else(|| ApplicationError::not_found("HACCP plan"))
    }

    pub fn create(
        &self,
        recipe_id: Uuid,
        input: NewHaccpPlan,
    ) -> Result<HaccpPlanDocument, ApplicationError> {
        if input.title.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "HACCP plan title cannot be empty".to_owned(),
            ));
        }
        self.repository.create_plan(recipe_id, input)
    }

    pub fn save(
        &self,
        plan_id: Uuid,
        input: SaveHaccpPlanRequest,
    ) -> Result<HaccpPlanDocument, ApplicationError> {
        if input.title.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "HACCP plan title cannot be empty".to_owned(),
            ));
        }
        for hazard in &input.hazards {
            if hazard.description.trim().is_empty() {
                return Err(ApplicationError::InvalidInput(
                    "hazard description cannot be empty".to_owned(),
                ));
            }
        }
        for ccp in &input.ccps {
            if ccp.name.trim().is_empty() || ccp.critical_limit.trim().is_empty() {
                return Err(ApplicationError::InvalidInput(
                    "CCP name and critical limit are required".to_owned(),
                ));
            }
        }
        self.repository.save_plan(plan_id, input)
    }

    pub fn delete(&self, plan_id: Uuid) -> Result<(), ApplicationError> {
        if self.repository.delete_plan(plan_id)? {
            Ok(())
        } else {
            Err(ApplicationError::not_found("HACCP plan"))
        }
    }

    pub fn record_monitoring(
        &self,
        ccp_id: Uuid,
        input: NewHaccpMonitoringRecord,
    ) -> Result<HaccpMonitoringRecord, ApplicationError> {
        if input.measured_value.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "measured value cannot be empty".to_owned(),
            ));
        }
        self.repository.add_monitoring_record(ccp_id, input)
    }
}

#[cfg(test)]
mod test;
