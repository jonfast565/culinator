use culinograph_models::{
    ApplicationError, HaccpCcp, HaccpHazard, HaccpMonitoringRecord, HaccpPlanDocument,
    HaccpPlanStatus, HaccpPlanSummary, HaccpRepository, HazardLikelihood, HazardSeverity,
    HazardType, NewHaccpMonitoringRecord, NewHaccpPlan, SaveHaccpPlanRequest,
};
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::repository::{SqliteCatalogRepository, parse_uuid};

impl HaccpRepository for SqliteCatalogRepository {
    fn list_plans_for_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<HaccpPlanSummary>, ApplicationError> {
        self.with_connection(|connection| list_haccp_plans(connection, &recipe_id.to_string()))
    }

    fn get_plan(&self, plan_id: Uuid) -> Result<Option<HaccpPlanDocument>, ApplicationError> {
        self.with_connection(|connection| get_haccp_plan(connection, &plan_id.to_string()))
    }

    fn create_plan(
        &self,
        recipe_id: Uuid,
        input: NewHaccpPlan,
    ) -> Result<HaccpPlanDocument, ApplicationError> {
        let id = Uuid::new_v4();
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO haccp_plans (id, recipe_id, title, description, status)
                 VALUES (?1, ?2, ?3, ?4, 'draft')",
                params![
                    id.to_string(),
                    recipe_id.to_string(),
                    input.title,
                    input.description
                ],
            )
        })?;
        self.get_plan(id)?.ok_or_else(|| {
            ApplicationError::Internal("created HACCP plan could not be read".to_owned())
        })
    }

    fn save_plan(
        &self,
        plan_id: Uuid,
        input: SaveHaccpPlanRequest,
    ) -> Result<HaccpPlanDocument, ApplicationError> {
        self.with_connection(|connection| {
            save_haccp_plan(connection, &plan_id.to_string(), &input)
        })?;
        self.get_plan(plan_id)?
            .ok_or_else(|| ApplicationError::not_found("HACCP plan"))
    }

    fn delete_plan(&self, plan_id: Uuid) -> Result<bool, ApplicationError> {
        self.with_connection(|connection| {
            Ok(
                connection.execute("DELETE FROM haccp_plans WHERE id=?1", [plan_id.to_string()])?
                    > 0,
            )
        })
    }

    fn add_monitoring_record(
        &self,
        ccp_id: Uuid,
        input: NewHaccpMonitoringRecord,
    ) -> Result<HaccpMonitoringRecord, ApplicationError> {
        let id = Uuid::new_v4();
        self.with_connection(|connection| {
            let exists: i64 = connection.query_row(
                "SELECT COUNT(*) FROM haccp_ccps WHERE id=?1",
                [ccp_id.to_string()],
                |row| row.get(0),
            )?;
            if exists == 0 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }
            connection.execute(
                "INSERT INTO haccp_monitoring_records
                 (id, ccp_id, measured_value, within_limit, corrective_action_taken, recorded_by, notes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id.to_string(),
                    ccp_id.to_string(),
                    input.measured_value,
                    input.within_limit as i64,
                    input.corrective_action_taken,
                    input.recorded_by,
                    input.notes
                ],
            )?;
            Ok(())
        })?;
        self.with_connection(|connection| load_monitoring_record(connection, &id.to_string()))?
            .ok_or_else(|| {
                ApplicationError::Internal("created monitoring record could not be read".to_owned())
            })
    }
}

pub fn list_haccp_plans(
    connection: &Connection,
    recipe_id: &str,
) -> Result<Vec<HaccpPlanSummary>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT p.id, p.recipe_id, p.title, p.description, p.status,
                COUNT(DISTINCT h.id), COUNT(DISTINCT c.id), p.updated_at
         FROM haccp_plans p
         LEFT JOIN haccp_hazards h ON h.plan_id = p.id
         LEFT JOIN haccp_ccps c ON c.plan_id = p.id
         WHERE p.recipe_id = ?1
         GROUP BY p.id
         ORDER BY p.updated_at DESC, p.title",
    )?;
    statement
        .query_map([recipe_id], |row| {
            Ok(HaccpPlanSummary {
                id: parse_uuid(row.get::<_, String>(0)?)?,
                recipe_id: parse_uuid(row.get::<_, String>(1)?)?,
                title: row.get(2)?,
                description: row.get(3)?,
                status: parse_plan_status(row.get::<_, String>(4)?),
                hazard_count: row.get(5)?,
                ccp_count: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?
        .collect()
}

pub fn get_haccp_plan(
    connection: &Connection,
    plan_id: &str,
) -> Result<Option<HaccpPlanDocument>, rusqlite::Error> {
    let header = connection
        .query_row(
            "SELECT id, recipe_id, title, description, status, updated_at
             FROM haccp_plans WHERE id=?1",
            [plan_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .optional()?;
    let Some((id, recipe_id, title, description, status, updated_at)) = header else {
        return Ok(None);
    };
    Ok(Some(HaccpPlanDocument {
        id: parse_uuid(id)?,
        recipe_id: parse_uuid(recipe_id)?,
        title,
        description,
        status: parse_plan_status(status),
        hazards: load_hazards(connection, plan_id)?,
        ccps: load_ccps(connection, plan_id)?,
        monitoring_records: load_monitoring_records_for_plan(connection, plan_id)?,
        updated_at,
    }))
}

fn save_haccp_plan(
    connection: &mut Connection,
    plan_id: &str,
    input: &SaveHaccpPlanRequest,
) -> Result<(), rusqlite::Error> {
    let tx = connection.transaction()?;
    let changed = tx.execute(
        "UPDATE haccp_plans
         SET title=?2, description=?3, status=?4, updated_at=CURRENT_TIMESTAMP
         WHERE id=?1",
        params![
            plan_id,
            input.title,
            input.description,
            plan_status_text(input.status)
        ],
    )?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    tx.execute(
        "DELETE FROM haccp_monitoring_records WHERE ccp_id IN
         (SELECT id FROM haccp_ccps WHERE plan_id=?1)",
        [plan_id],
    )?;
    tx.execute("DELETE FROM haccp_ccps WHERE plan_id=?1", [plan_id])?;
    tx.execute("DELETE FROM haccp_hazards WHERE plan_id=?1", [plan_id])?;

    for hazard in &input.hazards {
        tx.execute(
            "INSERT INTO haccp_hazards
             (id, plan_id, position, hazard_type, description, severity, likelihood,
              preventive_measures, is_ccp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                hazard.id.to_string(),
                plan_id,
                hazard.position,
                hazard_type_text(hazard.hazard_type),
                hazard.description,
                severity_text(hazard.severity),
                likelihood_text(hazard.likelihood),
                hazard.preventive_measures,
                hazard.is_ccp as i64
            ],
        )?;
    }

    for ccp in &input.ccps {
        tx.execute(
            "INSERT INTO haccp_ccps
             (id, plan_id, hazard_id, position, name, operation_symbol, critical_limit,
              monitoring_procedure, monitoring_frequency, corrective_action,
              verification_procedure, responsible_party)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                ccp.id.to_string(),
                plan_id,
                ccp.hazard_id.map(|value| value.to_string()),
                ccp.position,
                ccp.name,
                ccp.operation_symbol,
                ccp.critical_limit,
                ccp.monitoring_procedure,
                ccp.monitoring_frequency,
                ccp.corrective_action,
                ccp.verification_procedure,
                ccp.responsible_party
            ],
        )?;
    }

    tx.commit()
}

fn load_hazards(
    connection: &Connection,
    plan_id: &str,
) -> Result<Vec<HaccpHazard>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT id, position, hazard_type, description, severity, likelihood,
                preventive_measures, is_ccp
         FROM haccp_hazards WHERE plan_id=?1 ORDER BY position, description",
    )?;
    statement
        .query_map([plan_id], |row| {
            Ok(HaccpHazard {
                id: parse_uuid(row.get::<_, String>(0)?)?,
                position: row.get(1)?,
                hazard_type: parse_hazard_type(row.get::<_, String>(2)?),
                description: row.get(3)?,
                severity: parse_severity(row.get::<_, String>(4)?),
                likelihood: parse_likelihood(row.get::<_, String>(5)?),
                preventive_measures: row.get(6)?,
                is_ccp: row.get::<_, i64>(7)? != 0,
            })
        })?
        .collect()
}

fn load_ccps(connection: &Connection, plan_id: &str) -> Result<Vec<HaccpCcp>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT id, hazard_id, position, name, operation_symbol, critical_limit,
                monitoring_procedure, monitoring_frequency, corrective_action,
                verification_procedure, responsible_party
         FROM haccp_ccps WHERE plan_id=?1 ORDER BY position, name",
    )?;
    statement
        .query_map([plan_id], |row| {
            Ok(HaccpCcp {
                id: parse_uuid(row.get::<_, String>(0)?)?,
                hazard_id: row
                    .get::<_, Option<String>>(1)?
                    .map(parse_uuid)
                    .transpose()?,
                position: row.get(2)?,
                name: row.get(3)?,
                operation_symbol: row.get(4)?,
                critical_limit: row.get(5)?,
                monitoring_procedure: row.get(6)?,
                monitoring_frequency: row.get(7)?,
                corrective_action: row.get(8)?,
                verification_procedure: row.get(9)?,
                responsible_party: row.get(10)?,
            })
        })?
        .collect()
}

fn load_monitoring_records_for_plan(
    connection: &Connection,
    plan_id: &str,
) -> Result<Vec<HaccpMonitoringRecord>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT r.id, r.ccp_id, r.recorded_at, r.measured_value, r.within_limit,
                r.corrective_action_taken, r.recorded_by, r.notes
         FROM haccp_monitoring_records r
         JOIN haccp_ccps c ON c.id = r.ccp_id
         WHERE c.plan_id = ?1
         ORDER BY r.recorded_at DESC",
    )?;
    statement
        .query_map([plan_id], map_monitoring_record)?
        .collect()
}

fn load_monitoring_record(
    connection: &Connection,
    record_id: &str,
) -> Result<Option<HaccpMonitoringRecord>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT id, ccp_id, recorded_at, measured_value, within_limit,
                    corrective_action_taken, recorded_by, notes
             FROM haccp_monitoring_records WHERE id=?1",
            [record_id],
            map_monitoring_record,
        )
        .optional()
}

fn map_monitoring_record(
    row: &rusqlite::Row<'_>,
) -> Result<HaccpMonitoringRecord, rusqlite::Error> {
    Ok(HaccpMonitoringRecord {
        id: parse_uuid(row.get::<_, String>(0)?)?,
        ccp_id: parse_uuid(row.get::<_, String>(1)?)?,
        recorded_at: row.get(2)?,
        measured_value: row.get(3)?,
        within_limit: row.get::<_, i64>(4)? != 0,
        corrective_action_taken: row.get(5)?,
        recorded_by: row.get(6)?,
        notes: row.get(7)?,
    })
}

fn plan_status_text(status: HaccpPlanStatus) -> &'static str {
    match status {
        HaccpPlanStatus::Draft => "draft",
        HaccpPlanStatus::Active => "active",
        HaccpPlanStatus::Archived => "archived",
    }
}

fn parse_plan_status(value: String) -> HaccpPlanStatus {
    match value.as_str() {
        "active" => HaccpPlanStatus::Active,
        "archived" => HaccpPlanStatus::Archived,
        _ => HaccpPlanStatus::Draft,
    }
}

fn hazard_type_text(value: HazardType) -> &'static str {
    match value {
        HazardType::Biological => "biological",
        HazardType::Chemical => "chemical",
        HazardType::Physical => "physical",
    }
}

fn parse_hazard_type(value: String) -> HazardType {
    match value.as_str() {
        "chemical" => HazardType::Chemical,
        "physical" => HazardType::Physical,
        _ => HazardType::Biological,
    }
}

fn severity_text(value: HazardSeverity) -> &'static str {
    match value {
        HazardSeverity::Low => "low",
        HazardSeverity::Medium => "medium",
        HazardSeverity::High => "high",
        HazardSeverity::Critical => "critical",
    }
}

fn parse_severity(value: String) -> HazardSeverity {
    match value.as_str() {
        "medium" => HazardSeverity::Medium,
        "high" => HazardSeverity::High,
        "critical" => HazardSeverity::Critical,
        _ => HazardSeverity::Low,
    }
}

fn likelihood_text(value: HazardLikelihood) -> &'static str {
    match value {
        HazardLikelihood::Unlikely => "unlikely",
        HazardLikelihood::Possible => "possible",
        HazardLikelihood::Likely => "likely",
        HazardLikelihood::Certain => "certain",
    }
}

fn parse_likelihood(value: String) -> HazardLikelihood {
    match value.as_str() {
        "possible" => HazardLikelihood::Possible,
        "likely" => HazardLikelihood::Likely,
        "certain" => HazardLikelihood::Certain,
        _ => HazardLikelihood::Unlikely,
    }
}

#[cfg(test)]
#[path = "haccp/test.rs"]
mod test;
