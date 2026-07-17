use culinator_core::{LaborMode, Recipe, ResourceKind, Value};
use culinator_models::{ApplicationError, RangeF64, RecipeSearch, SearchHit, SearchQuery};
use rusqlite::{Connection, Transaction, params};

use crate::repository::{SqliteCatalogRepository, parse_uuid};

impl RecipeSearch for SqliteCatalogRepository {
    fn query(&self, query: &SearchQuery) -> Result<Vec<SearchHit>, ApplicationError> {
        self.with_connection(|connection| search(connection, query))
    }
}

pub(crate) fn index_recipe(connection: &Transaction<'_>, recipe: &Recipe) -> rusqlite::Result<()> {
    let recipe_id = recipe.id.to_string();
    let book_id = recipe.book_id.map(|id| id.to_string());
    let fields = extract_index_fields(recipe);

    connection.execute(
        "DELETE FROM recipe_search WHERE recipe_id = ?1",
        [&recipe_id],
    )?;
    connection.execute(
        "INSERT INTO recipe_search (title, ingredients, techniques, notes, section, recipe_id, book_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            fields.title,
            fields.ingredients,
            fields.techniques,
            fields.notes,
            fields.section,
            recipe_id,
            book_id,
        ],
    )?;

    let allergens_json =
        serde_json::to_string(&fields.allergens).unwrap_or_else(|_| "[]".to_owned());
    connection.execute(
        "INSERT INTO recipe_search_filters (recipe_id, book_id, allergens_json, max_active_minutes, hydration_percent)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(recipe_id) DO UPDATE SET
           book_id=excluded.book_id,
           allergens_json=excluded.allergens_json,
           max_active_minutes=excluded.max_active_minutes,
           hydration_percent=excluded.hydration_percent",
        params![
            recipe_id,
            book_id,
            allergens_json,
            fields.max_active_minutes,
            fields.hydration_percent,
        ],
    )?;
    Ok(())
}

struct IndexFields {
    title: String,
    ingredients: String,
    techniques: String,
    notes: String,
    section: String,
    allergens: Vec<String>,
    max_active_minutes: Option<f64>,
    hydration_percent: Option<f64>,
}

fn extract_index_fields(recipe: &Recipe) -> IndexFields {
    let ingredients = recipe
        .resources
        .iter()
        .filter(|resource| {
            matches!(
                resource.kind,
                ResourceKind::Ingredient | ResourceKind::Material | ResourceKind::Intermediate
            )
        })
        .map(|resource| resource.symbol.clone())
        .collect::<Vec<_>>()
        .join(" ");

    let techniques = recipe
        .operations
        .iter()
        .filter_map(|operation| {
            operation
                .properties
                .get("does")
                .or_else(|| operation.properties.get("action"))
                .map(value_to_plain)
        })
        .collect::<Vec<_>>()
        .join(" ");

    let section = recipe
        .properties
        .get("section")
        .map(value_to_plain)
        .unwrap_or_default();

    let notes = recipe
        .properties
        .get("notes")
        .or_else(|| recipe.properties.get("note"))
        .map(value_to_plain)
        .unwrap_or_default();

    let mut allergens = Vec::new();
    for resource in &recipe.resources {
        if let Some(value) = resource.properties.get("allergen") {
            allergens.push(value_to_plain(value));
        }
    }
    allergens.sort();
    allergens.dedup();

    let max_active_minutes = {
        let total_seconds: u64 = recipe
            .operations
            .iter()
            .filter(|operation| operation.labor == Some(LaborMode::Active))
            .filter_map(|operation| {
                operation
                    .duration_max_seconds
                    .or(operation.duration_min_seconds)
            })
            .sum();
        if total_seconds > 0 {
            Some(total_seconds as f64 / 60.0)
        } else {
            None
        }
    };

    let hydration_percent = recipe
        .formulas
        .first()
        .and_then(|formula| formula.solve_for_target_mass(1000.0).ok())
        .map(|result| result.hydration_percent);

    IndexFields {
        title: recipe.title.clone(),
        ingredients,
        techniques,
        notes,
        section,
        allergens,
        max_active_minutes,
        hydration_percent,
    }
}

fn value_to_plain(value: &Value) -> String {
    match value {
        Value::Text(text) | Value::Symbol(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Quantity(quantity) => format!("{} {}", quantity.value, quantity.unit),
        Value::Boolean(flag) => flag.to_string(),
        Value::List(items) => items
            .iter()
            .map(value_to_plain)
            .collect::<Vec<_>>()
            .join(" "),
        Value::Object(map) => map
            .values()
            .map(value_to_plain)
            .collect::<Vec<_>>()
            .join(" "),
        Value::Range { min, max } => format!("{} to {}", value_to_plain(min), value_to_plain(max)),
    }
}

pub(crate) fn search(
    connection: &Connection,
    query: &SearchQuery,
) -> Result<Vec<SearchHit>, rusqlite::Error> {
    let limit = query.limit.clamp(1, 200);
    let text = query
        .text
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());

    let mut sql = String::from(
        "SELECT rs.recipe_id, rs.book_id, rs.title,
                snippet(recipe_search, 0, '<mark>', '</mark>', '…', 24) AS snippet,
                bm25(recipe_search) AS score
         FROM recipe_search rs
         INNER JOIN recipe_search_filters f ON f.recipe_id = rs.recipe_id
         WHERE 1=1",
    );
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    if let Some(book_id) = query.book_id {
        sql.push_str(" AND f.book_id = ?");
        params.push(Box::new(book_id.to_string()));
    }

    for allergen in &query.exclude_allergens {
        let needle = allergen.trim().to_ascii_lowercase();
        if needle.is_empty() {
            continue;
        }
        sql.push_str(
            " AND NOT EXISTS (
            SELECT 1 FROM json_each(f.allergens_json) AS allergen
            WHERE lower(allergen.value) = ?
        )",
        );
        params.push(Box::new(needle));
    }

    if let Some(max_active) = query.max_active_minutes {
        sql.push_str(" AND (f.max_active_minutes IS NULL OR f.max_active_minutes <= ?)");
        params.push(Box::new(max_active));
    }

    apply_hydration_filter(&mut sql, &mut params, query.hydration.as_ref());

    if let Some(text) = text {
        sql.push_str(" AND recipe_search MATCH ?");
        params.push(Box::new(format_match_query(text)));
    }

    sql.push_str(" ORDER BY score LIMIT ?");
    params.push(Box::new(limit as i64));

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|value| value.as_ref()).collect();

    let mut statement = connection.prepare(&sql)?;
    let hits = statement
        .query_map(param_refs.as_slice(), |row| {
            Ok(SearchHit {
                recipe_id: parse_uuid(row.get::<_, String>(0)?)?,
                book_id: row
                    .get::<_, Option<String>>(1)?
                    .map(parse_uuid)
                    .transpose()?,
                title: row.get(2)?,
                snippet: row.get(3)?,
                score: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(hits)
}

fn apply_hydration_filter(
    sql: &mut String,
    params: &mut Vec<Box<dyn rusqlite::types::ToSql>>,
    hydration: Option<&RangeF64>,
) {
    let Some(range) = hydration else {
        return;
    };
    if let Some(min) = range.min {
        sql.push_str(" AND (f.hydration_percent IS NULL OR f.hydration_percent >= ?)");
        params.push(Box::new(min));
    }
    if let Some(max) = range.max {
        sql.push_str(" AND (f.hydration_percent IS NULL OR f.hydration_percent <= ?)");
        params.push(Box::new(max));
    }
}

fn format_match_query(text: &str) -> String {
    text.split_whitespace()
        .map(|token| format!("\"{}\"", token.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" AND ")
}

#[cfg(test)]
#[path = "search/test.rs"]
mod test;
