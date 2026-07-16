use culinograph_models::{
    ApplicationError, ExecutionRepository, NewRecipeTry, NewTryObservation, RecipeSchedule,
    RecipeTryDocument, RecipeTryStatus, RecipeTrySummary, TryObservation, TryOperation,
    TryOperationStatus, UpdateRecipeTry, UpdateTryOperation,
};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::repository::{SqliteCatalogRepository, parse_uuid};

impl ExecutionRepository for SqliteCatalogRepository {
    fn list_tries_for_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<RecipeTrySummary>, ApplicationError> {
        self.with_connection(|connection| list_recipe_tries(connection, &recipe_id.to_string()))
    }

    fn get_try(&self, try_id: Uuid) -> Result<Option<RecipeTryDocument>, ApplicationError> {
        self.with_connection(|connection| get_recipe_try(connection, &try_id.to_string()))
    }

    fn start_try(
        &self,
        recipe_id: Uuid,
        source_text: &str,
        schedule: &RecipeSchedule,
        input: NewRecipeTry,
    ) -> Result<RecipeTryDocument, ApplicationError> {
        let try_id = Uuid::new_v4();
        let revision_id = Uuid::new_v4();
        self.with_connection(|connection| {
            start_recipe_try(
                connection,
                &try_id.to_string(),
                &recipe_id.to_string(),
                &revision_id.to_string(),
                source_text,
                schedule,
                &input,
            )
        })?;
        self.get_try(try_id)?.ok_or_else(|| {
            ApplicationError::Internal("created recipe try could not be read".to_owned())
        })
    }

    fn update_try(
        &self,
        try_id: Uuid,
        input: UpdateRecipeTry,
    ) -> Result<RecipeTryDocument, ApplicationError> {
        self.with_connection(|connection| {
            update_recipe_try(connection, &try_id.to_string(), &input)
        })?;
        self.get_try(try_id)?
            .ok_or_else(|| ApplicationError::not_found("recipe try"))
    }

    fn update_try_operation(
        &self,
        try_id: Uuid,
        operation_id: Uuid,
        input: UpdateTryOperation,
    ) -> Result<RecipeTryDocument, ApplicationError> {
        self.with_connection(|connection| {
            update_try_operation_row(
                connection,
                &try_id.to_string(),
                &operation_id.to_string(),
                &input,
            )
        })?;
        self.get_try(try_id)?
            .ok_or_else(|| ApplicationError::not_found("recipe try"))
    }

    fn add_observation(
        &self,
        try_id: Uuid,
        input: NewTryObservation,
    ) -> Result<RecipeTryDocument, ApplicationError> {
        self.with_connection(|connection| {
            add_try_observation(connection, &try_id.to_string(), &input)
        })?;
        self.get_try(try_id)?
            .ok_or_else(|| ApplicationError::not_found("recipe try"))
    }

    fn delete_try(&self, try_id: Uuid) -> Result<bool, ApplicationError> {
        self.with_connection(|connection| {
            Ok(connection.execute("DELETE FROM executions WHERE id=?1", [try_id.to_string()])? > 0)
        })
    }
}

pub fn list_recipe_tries(
    connection: &Connection,
    recipe_id: &str,
) -> Result<Vec<RecipeTrySummary>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT e.id, e.recipe_id, e.title, e.status, e.started_at, e.completed_at,
                COUNT(DISTINCT eo.operation_id), COUNT(DISTINCT obs.id)
         FROM executions e
         LEFT JOIN execution_operations eo ON eo.execution_id = e.id
         LEFT JOIN execution_observations obs ON obs.execution_id = e.id
         WHERE e.recipe_id = ?1
         GROUP BY e.id
         ORDER BY COALESCE(e.started_at, e.completed_at) DESC",
    )?;
    statement
        .query_map([recipe_id], |row| {
            Ok(RecipeTrySummary {
                id: parse_uuid(row.get::<_, String>(0)?)?,
                recipe_id: parse_uuid(row.get::<_, String>(1)?)?,
                title: row.get(2)?,
                status: parse_try_status(row.get::<_, String>(3)?),
                started_at: row.get(4)?,
                completed_at: row.get(5)?,
                operation_count: row.get(6)?,
                observation_count: row.get(7)?,
            })
        })?
        .collect()
}

pub fn get_recipe_try(
    connection: &Connection,
    try_id: &str,
) -> Result<Option<RecipeTryDocument>, rusqlite::Error> {
    let header = connection
        .query_row(
            "SELECT id, recipe_id, recipe_revision_id, title, status, scale_factor,
                    started_at, completed_at, notes, findings
             FROM executions WHERE id=?1",
            [try_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, f64>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                ))
            },
        )
        .optional()?;
    let Some((
        id,
        recipe_id,
        recipe_revision_id,
        title,
        status,
        scale_factor,
        started_at,
        completed_at,
        notes,
        findings,
    )) = header
    else {
        return Ok(None);
    };
    Ok(Some(RecipeTryDocument {
        id: parse_uuid(id)?,
        recipe_id: parse_uuid(recipe_id)?,
        recipe_revision_id: recipe_revision_id.map(parse_uuid).transpose()?,
        title,
        status: parse_try_status(status),
        scale_factor,
        started_at,
        completed_at,
        notes,
        findings,
        operations: load_try_operations(connection, try_id)?,
        observations: load_try_observations(connection, try_id)?,
    }))
}

fn start_recipe_try(
    connection: &mut Connection,
    try_id: &str,
    recipe_id: &str,
    revision_id: &str,
    source_text: &str,
    schedule: &RecipeSchedule,
    input: &NewRecipeTry,
) -> Result<(), rusqlite::Error> {
    let tx = connection.transaction()?;
    let source_hash = format!("{:x}", simple_hash(source_text));
    tx.execute(
        "INSERT INTO recipe_revisions (id, recipe_id, version_label, source_text, source_hash)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            revision_id,
            recipe_id,
            "kitchen_try",
            source_text,
            source_hash
        ],
    )?;
    tx.execute(
        "INSERT INTO executions
         (id, recipe_id, recipe_revision_id, scale_factor, status, started_at, notes, title)
         VALUES (?1, ?2, ?3, ?4, 'active', CURRENT_TIMESTAMP, ?5, ?6)",
        params![
            try_id,
            recipe_id,
            revision_id,
            input.scale_factor.unwrap_or(1.0),
            input.notes,
            input.title
        ],
    )?;

    let operation_ids = load_operation_ids(&tx, recipe_id)?;
    for scheduled in &schedule.operations {
        let Some(operation_id) = operation_ids.get(&scheduled.symbol) else {
            continue;
        };
        let scheduled_start = format_iso_offset(scheduled.start_seconds);
        let scheduled_end = format_iso_offset(scheduled.end_seconds);
        tx.execute(
            "INSERT INTO execution_operations
             (execution_id, operation_id, operation_symbol, status,
              scheduled_start, scheduled_end)
             VALUES (?1, ?2, ?3, 'pending', ?4, ?5)",
            params![
                try_id,
                operation_id,
                scheduled.symbol,
                scheduled_start,
                scheduled_end
            ],
        )?;
    }

    tx.commit()
}

fn update_recipe_try(
    connection: &mut Connection,
    try_id: &str,
    input: &UpdateRecipeTry,
) -> Result<(), rusqlite::Error> {
    let current_status = connection.query_row(
        "SELECT status FROM executions WHERE id=?1",
        [try_id],
        |row| row.get::<_, String>(0),
    )?;
    let status = input
        .status
        .map(try_status_text)
        .unwrap_or(current_status.as_str());
    let completed_at = if status == "completed" || status == "abandoned" {
        Some("CURRENT_TIMESTAMP")
    } else {
        None
    };
    let changed = if completed_at.is_some() {
        connection.execute(
            "UPDATE executions
             SET status=?2, title=COALESCE(?3, title), notes=COALESCE(?4, notes),
                 findings=COALESCE(?5, findings), completed_at=CURRENT_TIMESTAMP
             WHERE id=?1",
            params![try_id, status, input.title, input.notes, input.findings],
        )?
    } else {
        connection.execute(
            "UPDATE executions
             SET status=?2, title=COALESCE(?3, title), notes=COALESCE(?4, notes),
                 findings=COALESCE(?5, findings)
             WHERE id=?1",
            params![try_id, status, input.title, input.notes, input.findings],
        )?
    };
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(())
}

fn update_try_operation_row(
    connection: &mut Connection,
    try_id: &str,
    operation_id: &str,
    input: &UpdateTryOperation,
) -> Result<(), rusqlite::Error> {
    let current_status = connection.query_row(
        "SELECT status FROM execution_operations WHERE execution_id=?1 AND operation_id=?2",
        params![try_id, operation_id],
        |row| row.get::<_, String>(0),
    )?;
    let status = input
        .status
        .map(try_operation_status_text)
        .unwrap_or(current_status.as_str());
    let set_start = status == "active";
    let set_end = matches!(status, "completed" | "skipped");
    let sql = if set_start && set_end {
        "UPDATE execution_operations
         SET status=?3, notes=COALESCE(?4, notes),
             actual_start=COALESCE(actual_start, CURRENT_TIMESTAMP),
             actual_end=CURRENT_TIMESTAMP
         WHERE execution_id=?1 AND operation_id=?2"
    } else if set_start {
        "UPDATE execution_operations
         SET status=?3, notes=COALESCE(?4, notes),
             actual_start=COALESCE(actual_start, CURRENT_TIMESTAMP)
         WHERE execution_id=?1 AND operation_id=?2"
    } else if set_end {
        "UPDATE execution_operations
         SET status=?3, notes=COALESCE(?4, notes),
             actual_end=CURRENT_TIMESTAMP
         WHERE execution_id=?1 AND operation_id=?2"
    } else {
        "UPDATE execution_operations
         SET status=?3, notes=COALESCE(?4, notes)
         WHERE execution_id=?1 AND operation_id=?2"
    };
    let changed = connection.execute(sql, params![try_id, operation_id, status, input.notes])?;
    if changed == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(())
}

fn add_try_observation(
    connection: &mut Connection,
    try_id: &str,
    input: &NewTryObservation,
) -> Result<(), rusqlite::Error> {
    let operation_id = input.operation_symbol.as_deref().and_then(|symbol| {
        connection
            .query_row(
                "SELECT eo.operation_id FROM execution_operations eo
                     WHERE eo.execution_id=?1 AND eo.operation_symbol=?2",
                params![try_id, symbol],
                |row| row.get::<_, String>(0),
            )
            .ok()
    });
    connection.execute(
        "INSERT INTO execution_observations
         (id, execution_id, operation_id, property_path, value_json, unit, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            Uuid::new_v4().to_string(),
            try_id,
            operation_id,
            input.property_path,
            serde_json::to_string(&input.value).unwrap_or_else(|_| "\"\"".to_owned()),
            input.unit,
            input.notes
        ],
    )?;
    Ok(())
}

fn load_try_operations(
    connection: &Connection,
    try_id: &str,
) -> Result<Vec<TryOperation>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT eo.operation_id, eo.operation_symbol, eo.status, eo.scheduled_start,
                eo.scheduled_end, eo.actual_start, eo.actual_end, eo.notes,
                o.duration_min_seconds, o.duration_max_seconds
         FROM execution_operations eo
         LEFT JOIN operations o ON o.id = eo.operation_id
         WHERE eo.execution_id = ?1
         ORDER BY eo.scheduled_start, eo.operation_symbol",
    )?;
    statement
        .query_map([try_id], |row| {
            let min_seconds = row.get::<_, Option<i64>>(8)?.unwrap_or(60);
            let max_seconds = row
                .get::<_, Option<i64>>(9)?
                .unwrap_or(min_seconds)
                .max(min_seconds);
            Ok(TryOperation {
                operation_id: parse_uuid(row.get::<_, String>(0)?)?,
                operation_symbol: row
                    .get::<_, Option<String>>(1)?
                    .unwrap_or_else(|| "unknown".to_owned()),
                status: parse_try_operation_status(row.get::<_, String>(2)?),
                scheduled_start: row.get(3)?,
                scheduled_end: row.get(4)?,
                actual_start: row.get(5)?,
                actual_end: row.get(6)?,
                notes: row.get(7)?,
                duration_seconds: max_seconds as u64,
            })
        })?
        .collect()
}

fn load_try_observations(
    connection: &Connection,
    try_id: &str,
) -> Result<Vec<TryObservation>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT obs.id, obs.operation_id, eo.operation_symbol, obs.observed_at,
                obs.property_path, obs.value_json, obs.unit, obs.notes
         FROM execution_observations obs
         LEFT JOIN execution_operations eo
           ON eo.execution_id = obs.execution_id AND eo.operation_id = obs.operation_id
         WHERE obs.execution_id = ?1
         ORDER BY obs.observed_at DESC",
    )?;
    statement
        .query_map([try_id], |row| {
            let value_json: String = row.get(5)?;
            Ok(TryObservation {
                id: parse_uuid(row.get::<_, String>(0)?)?,
                operation_id: row
                    .get::<_, Option<String>>(1)?
                    .map(parse_uuid)
                    .transpose()?,
                operation_symbol: row.get(2)?,
                observed_at: row.get(3)?,
                property_path: row.get(4)?,
                value: serde_json::from_str(&value_json).unwrap_or(Value::Null),
                unit: row.get(6)?,
                notes: row.get(7)?,
            })
        })?
        .collect()
}

fn load_operation_ids(
    connection: &Connection,
    recipe_id: &str,
) -> Result<HashMap<String, String>, rusqlite::Error> {
    let mut statement =
        connection.prepare("SELECT id, symbol FROM operations WHERE recipe_id=?1")?;
    let rows = statement.query_map([recipe_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut out = HashMap::new();
    for row in rows {
        let (id, symbol) = row?;
        out.insert(symbol, id);
    }
    Ok(out)
}

fn simple_hash(value: &str) -> u64 {
    value.bytes().fold(0_u64, |hash, byte| {
        hash.wrapping_mul(16777619).wrapping_add(byte as u64)
    })
}

fn format_iso_offset(seconds: u64) -> String {
    format!("+{seconds} seconds")
}

fn try_status_text(status: RecipeTryStatus) -> &'static str {
    match status {
        RecipeTryStatus::Active => "active",
        RecipeTryStatus::Paused => "paused",
        RecipeTryStatus::Completed => "completed",
        RecipeTryStatus::Abandoned => "abandoned",
    }
}

fn parse_try_status(value: String) -> RecipeTryStatus {
    match value.as_str() {
        "paused" => RecipeTryStatus::Paused,
        "completed" => RecipeTryStatus::Completed,
        "abandoned" => RecipeTryStatus::Abandoned,
        _ => RecipeTryStatus::Active,
    }
}

fn try_operation_status_text(status: TryOperationStatus) -> &'static str {
    match status {
        TryOperationStatus::Pending => "pending",
        TryOperationStatus::Active => "active",
        TryOperationStatus::Completed => "completed",
        TryOperationStatus::Skipped => "skipped",
    }
}

fn parse_try_operation_status(value: String) -> TryOperationStatus {
    match value.as_str() {
        "active" => TryOperationStatus::Active,
        "completed" => TryOperationStatus::Completed,
        "skipped" => TryOperationStatus::Skipped,
        _ => TryOperationStatus::Pending,
    }
}

#[cfg(test)]
#[path = "execution/test.rs"]
mod test;
