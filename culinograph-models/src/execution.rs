use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeTryStatus {
    Active,
    Paused,
    Completed,
    Abandoned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TryOperationStatus {
    Pending,
    Active,
    Completed,
    Skipped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeTrySummary {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub title: Option<String>,
    pub status: RecipeTryStatus,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub operation_count: i64,
    pub observation_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewRecipeTry {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub scale_factor: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TryOperation {
    pub operation_id: Uuid,
    pub operation_symbol: String,
    pub status: TryOperationStatus,
    pub scheduled_start: Option<String>,
    pub scheduled_end: Option<String>,
    pub actual_start: Option<String>,
    pub actual_end: Option<String>,
    pub duration_seconds: u64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TryObservation {
    pub id: Uuid,
    pub operation_id: Option<Uuid>,
    pub operation_symbol: Option<String>,
    pub observed_at: String,
    pub property_path: String,
    pub value: Value,
    pub unit: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeTryDocument {
    pub id: Uuid,
    pub recipe_id: Uuid,
    pub recipe_revision_id: Option<Uuid>,
    pub title: Option<String>,
    pub status: RecipeTryStatus,
    pub scale_factor: f64,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub notes: Option<String>,
    pub findings: Option<String>,
    pub operations: Vec<TryOperation>,
    pub observations: Vec<TryObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRecipeTry {
    pub status: Option<RecipeTryStatus>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub findings: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTryOperation {
    pub status: Option<TryOperationStatus>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTryObservation {
    pub operation_symbol: Option<String>,
    pub property_path: String,
    pub value: Value,
    pub unit: Option<String>,
    pub notes: Option<String>,
}

#[cfg(test)]
mod test;
