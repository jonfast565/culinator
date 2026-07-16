use crate::{
    ApplicationError, HaccpMonitoringRecord, HaccpPlanDocument, HaccpPlanSummary, NewHaccpPlan,
    NewHaccpMonitoringRecord, SaveHaccpPlanRequest,
};
use uuid::Uuid;

pub trait HaccpRepository: Send + Sync {
    fn list_plans_for_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<HaccpPlanSummary>, ApplicationError>;
    fn get_plan(&self, plan_id: Uuid) -> Result<Option<HaccpPlanDocument>, ApplicationError>;
    fn create_plan(
        &self,
        recipe_id: Uuid,
        input: NewHaccpPlan,
    ) -> Result<HaccpPlanDocument, ApplicationError>;
    fn save_plan(
        &self,
        plan_id: Uuid,
        input: SaveHaccpPlanRequest,
    ) -> Result<HaccpPlanDocument, ApplicationError>;
    fn delete_plan(&self, plan_id: Uuid) -> Result<bool, ApplicationError>;
    fn add_monitoring_record(
        &self,
        ccp_id: Uuid,
        input: NewHaccpMonitoringRecord,
    ) -> Result<HaccpMonitoringRecord, ApplicationError>;
}

#[cfg(test)]
#[path = "haccp/test.rs"]
mod test;
