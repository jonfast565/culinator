use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HaccpPlanStatus {
    Draft,
    Active,
    Archived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardType {
    Biological,
    Chemical,
    Physical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardLikelihood {
    Unlikely,
    Possible,
    Likely,
    Certain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HaccpPlanSummary {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: HaccpPlanStatus,
    pub hazard_count: i64,
    pub ccp_count: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewHaccpPlan {
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HaccpHazard {
    pub id: Uuid,
    pub position: i64,
    pub hazard_type: HazardType,
    pub description: String,
    pub severity: HazardSeverity,
    pub likelihood: HazardLikelihood,
    pub preventive_measures: Option<String>,
    pub is_ccp: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HaccpCcp {
    pub id: Uuid,
    pub hazard_id: Option<Uuid>,
    pub position: i64,
    pub name: String,
    pub operation_symbol: Option<String>,
    pub critical_limit: String,
    pub monitoring_procedure: String,
    pub monitoring_frequency: Option<String>,
    pub corrective_action: String,
    pub verification_procedure: Option<String>,
    pub responsible_party: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HaccpMonitoringRecord {
    pub id: Uuid,
    pub ccp_id: Uuid,
    pub recorded_at: String,
    pub measured_value: String,
    pub within_limit: bool,
    pub corrective_action_taken: Option<String>,
    pub recorded_by: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HaccpPlanDocument {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: HaccpPlanStatus,
    pub hazards: Vec<HaccpHazard>,
    pub ccps: Vec<HaccpCcp>,
    pub monitoring_records: Vec<HaccpMonitoringRecord>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveHaccpPlanRequest {
    pub title: String,
    pub description: Option<String>,
    pub status: HaccpPlanStatus,
    pub hazards: Vec<HaccpHazard>,
    pub ccps: Vec<HaccpCcp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewHaccpMonitoringRecord {
    pub measured_value: String,
    pub within_limit: bool,
    pub corrective_action_taken: Option<String>,
    pub recorded_by: Option<String>,
    pub notes: Option<String>,
}

#[cfg(test)]
mod test;
