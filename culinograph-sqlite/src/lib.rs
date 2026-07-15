mod repository;
pub use repository::SqliteCatalogRepository;

use culinograph_core::{Formula, FormulaBasis, FormulaIngredient, FormulaResult, Recipe, RecipeBook};
use rusqlite::{params, Connection, OptionalExtension, Result, Transaction};
use std::collections::HashMap;

pub const MIGRATION_001: &str = include_str!("../../migrations/001_initial.sql");
pub const MIGRATION_002: &str = include_str!("../../migrations/002_domain_extensions.sql");
pub const MIGRATION_003: &str = include_str!("../../migrations/003_formulas.sql");
pub const MIGRATION_004: &str = include_str!("../../migrations/004_general_formula.sql");
pub const MIGRATION_005: &str = include_str!("../../migrations/005_recipe_books.sql");

#[derive(Debug, Clone)]
pub struct RecipeRecord {
    pub id: String,
    pub symbol: String,
    pub title: String,
    pub protocol_version: String,
    pub source_text: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn migrate(connection: &Connection) -> Result<()> {
    connection.execute_batch("PRAGMA foreign_keys = ON;")?;
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version < 1 {
        connection.execute_batch(MIGRATION_001)?;
        connection.pragma_update(None, "user_version", 1)?;
    }
    if version < 2 {
        connection.execute_batch(MIGRATION_002)?;
        connection.pragma_update(None, "user_version", 2)?;
    }
    if version < 3 {
        connection.execute_batch(MIGRATION_003)?;
        connection.pragma_update(None, "user_version", 3)?;
    }
    if version < 4 {
        connection.execute_batch(MIGRATION_004)?;
        connection.pragma_update(None, "user_version", 4)?;
    }
    if version < 5 {
        connection.execute_batch(MIGRATION_005)?;
        connection.pragma_update(None, "user_version", 5)?;
    }
    Ok(())
}

pub fn save_recipe(connection: &mut Connection, recipe: &Recipe, source: &str) -> Result<()> {
    let tx = connection.transaction()?;
    upsert_recipe(&tx, recipe, source)?;
    replace_recipe_entities(&tx, recipe)?;
    tx.commit()
}

fn upsert_recipe(tx: &Transaction<'_>, recipe: &Recipe, source: &str) -> Result<()> {
    tx.execute(
        "INSERT INTO recipes
         (id, book_id, symbol, title, protocol_version, declared_type_json, source_text, properties_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
           book_id=excluded.book_id,
           symbol=excluded.symbol,
           title=excluded.title,
           protocol_version=excluded.protocol_version,
           declared_type_json=excluded.declared_type_json,
           source_text=excluded.source_text,
           properties_json=excluded.properties_json,
           updated_at=CURRENT_TIMESTAMP",
        params![
            recipe.id.to_string(),
            recipe.book_id.map(|value| value.to_string()),
            recipe.symbol,
            recipe.title,
            recipe.protocol_version,
            serde_json::to_string(&recipe.declared_type).unwrap(),
            source,
            serde_json::to_string(&recipe.properties).unwrap(),
        ],
    )?;
    Ok(())
}

fn replace_recipe_entities(tx: &Transaction<'_>, recipe: &Recipe) -> Result<()> {
    let recipe_id = recipe.id.to_string();
    // Child rows cascade from these roots; delete in dependency-safe order for clarity.
    tx.execute("DELETE FROM operation_requirements WHERE operation_id IN (SELECT id FROM operations WHERE recipe_id=?1)", [&recipe_id])?;
    tx.execute("DELETE FROM operation_effects WHERE operation_id IN (SELECT id FROM operations WHERE recipe_id=?1)", [&recipe_id])?;
    tx.execute("DELETE FROM operation_bindings WHERE operation_id IN (SELECT id FROM operations WHERE recipe_id=?1)", [&recipe_id])?;
    tx.execute("DELETE FROM operation_dependencies WHERE operation_id IN (SELECT id FROM operations WHERE recipe_id=?1)", [&recipe_id])?;
    tx.execute("DELETE FROM operations WHERE recipe_id=?1", [&recipe_id])?;
    tx.execute("DELETE FROM processes WHERE recipe_id=?1", [&recipe_id])?;
    tx.execute("DELETE FROM resource_nutrients WHERE resource_id IN (SELECT id FROM resources WHERE recipe_id=?1)", [&recipe_id])?;
    tx.execute("DELETE FROM resource_allergens WHERE resource_id IN (SELECT id FROM resources WHERE recipe_id=?1)", [&recipe_id])?;
    tx.execute("DELETE FROM resources WHERE recipe_id=?1", [&recipe_id])?;
    tx.execute("DELETE FROM type_declarations WHERE recipe_id=?1", [&recipe_id])?;
    tx.execute("DELETE FROM servings WHERE recipe_id=?1", [&recipe_id])?;
    tx.execute("DELETE FROM recipe_yields WHERE recipe_id=?1", [&recipe_id])?;
    tx.execute("DELETE FROM formulas WHERE recipe_id=?1", [&recipe_id])?;

    for declaration in &recipe.types {
        tx.execute(
            "INSERT INTO type_declarations (id, recipe_id, symbol, base_type_json, states_json, properties_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![declaration.id.to_string(), recipe_id, declaration.name,
                serde_json::to_string(&declaration.base).unwrap(),
                serde_json::to_string(&declaration.states).unwrap(),
                serde_json::to_string(&declaration.properties).unwrap()],
        )?;
    }

    let mut resource_ids = HashMap::new();
    for resource in &recipe.resources {
        resource_ids.insert(resource.symbol.clone(), resource.id.to_string());
        tx.execute(
            "INSERT INTO resources (id, recipe_id, symbol, resource_kind, declared_type_json, properties_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![resource.id.to_string(), recipe_id, resource.symbol,
                format!("{:?}", resource.kind).to_lowercase(),
                serde_json::to_string(&resource.declared_type).unwrap(),
                serde_json::to_string(&resource.properties).unwrap()],
        )?;
    }

    let mut process_ids = HashMap::new();
    for process in &recipe.processes {
        process_ids.insert(process.symbol.clone(), process.id.to_string());
    }
    for process in &recipe.processes {
        let parent_id = process.parent.as_ref().and_then(|name| process_ids.get(name)).cloned();
        tx.execute(
            "INSERT INTO processes (id, recipe_id, parent_process_id, symbol, declared_type_json, properties_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![process.id.to_string(), recipe_id, parent_id, process.symbol,
                serde_json::to_string(&process.declared_type).unwrap(),
                serde_json::to_string(&process.properties).unwrap()],
        )?;
    }

    let operation_ids: HashMap<_, _> = recipe.operations.iter()
        .map(|operation| (operation.symbol.clone(), operation.id.to_string())).collect();

    for operation in &recipe.operations {
        let process_id = process_ids.get(&operation.process).cloned();
        tx.execute(
            "INSERT INTO operations
             (id, recipe_id, process_id, symbol, declared_type_json, labor_mode,
              duration_min_seconds, duration_max_seconds, properties_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![operation.id.to_string(), recipe_id, process_id, operation.symbol,
                serde_json::to_string(&operation.declared_type).unwrap(),
                operation.labor.map(|v| format!("{:?}", v).to_lowercase()),
                operation.duration_min_seconds.map(|v| v as i64),
                operation.duration_max_seconds.map(|v| v as i64),
                serde_json::to_string(&operation.properties).unwrap()],
        )?;
    }

    for operation in &recipe.operations {
        let operation_id = operation.id.to_string();
        for dependency in &operation.dependencies {
            let Some(predecessor_id) = operation_ids.get(&dependency.predecessor) else { continue; };
            tx.execute(
                "INSERT INTO operation_dependencies
                 (operation_id, predecessor_operation_id, dependency_kind,
                  minimum_lag_seconds, maximum_lag_seconds, optional)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![operation_id, predecessor_id,
                    format!("{:?}", dependency.kind).to_lowercase(),
                    dependency.minimum_lag_seconds.map(|v| v as i64),
                    dependency.maximum_lag_seconds.map(|v| v as i64),
                    dependency.optional as i64],
            )?;
        }
        for binding in &operation.bindings {
            let Some(resource_id) = resource_ids.get(&binding.resource) else { continue; };
            tx.execute(
                "INSERT INTO operation_bindings
                 (operation_id, resource_id, role, quantity_json, exclusive)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![operation_id, resource_id,
                    format!("{:?}", binding.role).to_lowercase(),
                    binding.quantity.as_ref().map(|q| serde_json::to_string(q).unwrap()),
                    binding.exclusive as i64],
            )?;
        }
        for requirement in &operation.requirements {
            tx.execute(
                "INSERT INTO operation_requirements (id, operation_id, source_expression)
                 VALUES (lower(hex(randomblob(16))), ?1, ?2)",
                params![operation_id, requirement.source],
            )?;
        }
        for effect in &operation.effects {
            tx.execute(
                "INSERT INTO operation_effects (id, operation_id, target_path, operator, value_json)
                 VALUES (lower(hex(randomblob(16))), ?1, ?2, ?3, ?4)",
                params![operation_id, effect.target_path, effect.operator,
                    serde_json::to_string(&effect.value).unwrap()],
            )?;
        }
    }

    for serving in &recipe.servings {
        tx.execute(
            "INSERT INTO servings
             (recipe_id, symbol, declared_type_json, amount_json, mass_grams, is_default)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![recipe_id, serving.symbol,
                serde_json::to_string(&serving.declared_type).unwrap(),
                serde_json::to_string(&serving.amount).unwrap(),
                serving.mass_grams, serving.is_default as i64],
        )?;
    }
    for yield_def in &recipe.yields {
        tx.execute(
            "INSERT INTO recipe_yields (id, recipe_id, amount_json, finished_mass_grams, properties_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![uuid::Uuid::new_v4().to_string(), recipe_id,
                serde_json::to_string(&yield_def.amount).unwrap(), yield_def.mass_grams,
                serde_json::to_string(&yield_def.properties).unwrap()],
        )?;
    }
    for formula in &recipe.formulas {
        tx.execute(
            "INSERT INTO formulas (id, recipe_id, symbol, name, basis, properties_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![formula.id.to_string(), recipe_id, formula.symbol, formula.name,
                formula_basis_text(formula.basis), serde_json::to_string(&formula.properties).unwrap()],
        )?;
        for (position, item) in formula.ingredients.iter().enumerate() {
            tx.execute(
                "INSERT INTO formula_ingredients
                 (id, formula_id, symbol, name, stage, basis, percentage, mass_grams, is_flour,
                  water_fraction, position, properties_json, is_reference, scalable)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![item.id.to_string(), formula.id.to_string(), item.symbol, item.name, item.stage,
                    formula_basis_text(item.basis), item.percentage, item.mass_grams, item.is_flour as i64,
                    item.water_fraction, position as i64, serde_json::to_string(&item.properties).unwrap(),
                    item.is_reference as i64, item.scalable as i64],
            )?;
        }
    }
    Ok(())
}

pub fn list_recipes(connection: &Connection) -> Result<Vec<RecipeRecord>> {
    let mut statement = connection.prepare(
        "SELECT id, symbol, title, protocol_version, source_text, created_at, updated_at
         FROM recipes ORDER BY updated_at DESC, title"
    )?;
    statement.query_map([], |row| Ok(RecipeRecord {
        id: row.get(0)?, symbol: row.get(1)?, title: row.get(2)?,
        protocol_version: row.get(3)?, source_text: row.get(4)?,
        created_at: row.get(5)?, updated_at: row.get(6)?,
    }))?.collect()
}

pub fn get_recipe_record(connection: &Connection, id: &str) -> Result<Option<RecipeRecord>> {
    connection.query_row(
        "SELECT id, symbol, title, protocol_version, source_text, created_at, updated_at
         FROM recipes WHERE id=?1", [id], |row| Ok(RecipeRecord {
            id: row.get(0)?, symbol: row.get(1)?, title: row.get(2)?,
            protocol_version: row.get(3)?, source_text: row.get(4)?,
            created_at: row.get(5)?, updated_at: row.get(6)?,
        })
    ).optional()
}

pub fn delete_recipe(connection: &Connection, id: &str) -> Result<bool> {
    Ok(connection.execute("DELETE FROM recipes WHERE id=?1", [id])? > 0)
}


#[derive(Debug, Clone)]
pub struct RecipeBookRecord {
    pub id: String, pub symbol: String, pub title: String, pub description: Option<String>,
    pub protocol_version: String, pub recipe_count: i64, pub created_at: String, pub updated_at: String,
}

pub fn save_recipe_book(connection: &mut Connection, book: &RecipeBook) -> Result<()> {
    let tx = connection.transaction()?;
    tx.execute(
        "INSERT INTO recipe_books (id, symbol, title, description, protocol_version, declared_type_json, properties_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET symbol=excluded.symbol, title=excluded.title, description=excluded.description,
         protocol_version=excluded.protocol_version, declared_type_json=excluded.declared_type_json,
         properties_json=excluded.properties_json, updated_at=CURRENT_TIMESTAMP",
        params![book.id.to_string(), book.symbol, book.title, book.description, book.protocol_version,
            serde_json::to_string(&book.declared_type).unwrap(), serde_json::to_string(&book.properties).unwrap()],
    )?;
    for (position, recipe) in book.recipes.iter().enumerate() {
        let mut recipe = recipe.clone();
        recipe.book_id = Some(book.id);
        upsert_recipe(&tx, &recipe, "")?;
        tx.execute("UPDATE recipes SET book_position=?2 WHERE id=?1", params![recipe.id.to_string(), position as i64])?;
        replace_recipe_entities(&tx, &recipe)?;
    }
    tx.commit()
}

pub fn list_recipe_books(connection: &Connection) -> Result<Vec<RecipeBookRecord>> {
    let mut statement = connection.prepare(
        "SELECT b.id, b.symbol, b.title, b.description, b.protocol_version, COUNT(r.id), b.created_at, b.updated_at
         FROM recipe_books b LEFT JOIN recipes r ON r.book_id=b.id GROUP BY b.id ORDER BY b.title")?;
    statement.query_map([], |row| Ok(RecipeBookRecord { id: row.get(0)?, symbol: row.get(1)?, title: row.get(2)?,
        description: row.get(3)?, protocol_version: row.get(4)?, recipe_count: row.get(5)?, created_at: row.get(6)?, updated_at: row.get(7)? }))?.collect()
}

pub fn move_recipe_to_book(connection: &Connection, recipe_id: &str, book_id: Option<&str>, position: i64) -> Result<bool> {
    Ok(connection.execute("UPDATE recipes SET book_id=?2, book_position=?3, updated_at=CURRENT_TIMESTAMP WHERE id=?1",
        params![recipe_id, book_id, position])? > 0)
}

pub fn save_formula(connection: &mut Connection, formula: &Formula) -> Result<()> {
    let tx = connection.transaction()?;
    tx.execute(
        "INSERT INTO formulas (id, recipe_id, symbol, name, basis, properties_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(id) DO UPDATE SET recipe_id=excluded.recipe_id, symbol=excluded.symbol,
           name=excluded.name, basis=excluded.basis, properties_json=excluded.properties_json,
           updated_at=CURRENT_TIMESTAMP",
        params![
            formula.id.to_string(), formula.recipe_id.map(|v| v.to_string()), formula.symbol,
            formula.name, formula_basis_text(formula.basis), serde_json::to_string(&formula.properties).unwrap()
        ],
    )?;
    tx.execute("DELETE FROM formula_ingredients WHERE formula_id=?1", [formula.id.to_string()])?;
    for (position, item) in formula.ingredients.iter().enumerate() {
        tx.execute(
            "INSERT INTO formula_ingredients
             (id, formula_id, symbol, name, stage, basis, percentage, mass_grams, is_flour, water_fraction, position, properties_json, is_reference, scalable)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![item.id.to_string(), formula.id.to_string(), item.symbol, item.name, item.stage,
                formula_basis_text(item.basis), item.percentage, item.mass_grams, item.is_flour as i64,
                item.water_fraction, position as i64, serde_json::to_string(&item.properties).unwrap(), item.is_reference as i64, item.scalable as i64],
        )?;
    }
    tx.commit()
}

pub fn list_formulas_for_recipe(connection: &Connection, recipe_id: &str) -> Result<Vec<Formula>> {
    let mut statement = connection.prepare(
        "SELECT id, recipe_id, symbol, name, basis, properties_json FROM formulas WHERE recipe_id=?1 ORDER BY name"
    )?;
    let rows = statement.query_map([recipe_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?,
            row.get::<_, String>(3)?, row.get::<_, String>(4)?, row.get::<_, String>(5)?))
    })?;
    let mut out = Vec::new();
    for row in rows {
        let (id, recipe_id, symbol, name, basis, properties_json) = row?;
        out.push(load_formula_parts(connection, id, recipe_id, symbol, name, basis, properties_json)?);
    }
    Ok(out)
}

pub fn get_formula(connection: &Connection, formula_id: &str) -> Result<Option<Formula>> {
    let header = connection.query_row(
        "SELECT id, recipe_id, symbol, name, basis, properties_json FROM formulas WHERE id=?1", [formula_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, String>(2)?,
            row.get::<_, String>(3)?, row.get::<_, String>(4)?, row.get::<_, String>(5)?)),
    ).optional()?;
    match header {
        Some((id, recipe_id, symbol, name, basis, properties_json)) =>
            Ok(Some(load_formula_parts(connection, id, recipe_id, symbol, name, basis, properties_json)?)),
        None => Ok(None),
    }
}

pub fn save_formula_run(connection: &Connection, formula_id: &str, target_mass_grams: f64, result: &FormulaResult) -> Result<String> {
    let id = uuid::Uuid::new_v4().to_string();
    connection.execute(
        "INSERT INTO formula_runs (id, formula_id, target_mass_grams, result_json) VALUES (?1, ?2, ?3, ?4)",
        params![id, formula_id, target_mass_grams, serde_json::to_string(result).unwrap()],
    )?;
    Ok(id)
}

fn load_formula_parts(connection: &Connection, id: String, recipe_id: Option<String>, symbol: String, name: String, basis: String, properties_json: String) -> Result<Formula> {
    let mut statement = connection.prepare(
        "SELECT id, symbol, name, stage, basis, percentage, mass_grams, is_flour, water_fraction, properties_json, is_reference, scalable
         FROM formula_ingredients WHERE formula_id=?1 ORDER BY position, name"
    )?;
    let ingredients = statement.query_map([&id], |row| {
        let basis_text: String = row.get(4)?;
        let properties: String = row.get(9)?;
        Ok(FormulaIngredient {
            id: uuid::Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
            symbol: row.get(1)?, name: row.get(2)?, stage: row.get(3)?,
            basis: parse_formula_basis(&basis_text), percentage: row.get(5)?, mass_grams: row.get(6)?,
            is_flour: row.get::<_, i64>(7)? != 0, water_fraction: row.get(8)?,
            is_reference: row.get::<_, i64>(10)? != 0, scalable: row.get::<_, i64>(11)? != 0,
            properties: serde_json::from_str(&properties).unwrap_or_default(),
        })
    })?.collect::<Result<Vec<_>>>()?;
    Ok(Formula {
        id: uuid::Uuid::parse_str(&id).unwrap(),
        recipe_id: recipe_id.and_then(|v| uuid::Uuid::parse_str(&v).ok()),
        symbol, name, basis: parse_formula_basis(&basis), ingredients,
        properties: serde_json::from_str(&properties_json).unwrap_or_default(),
    })
}

fn formula_basis_text(basis: FormulaBasis) -> &'static str {
    match basis { FormulaBasis::ReferencePercent => "bakers_percent", FormulaBasis::PercentOfTotal => "percent_of_total", FormulaBasis::AbsoluteMass => "absolute_mass" }
}
fn parse_formula_basis(value: &str) -> FormulaBasis {
    match value { "percent_of_total" => FormulaBasis::PercentOfTotal, "absolute_mass" => FormulaBasis::AbsoluteMass, _ => FormulaBasis::ReferencePercent }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_create_domain_tables() {
        let connection = Connection::open_in_memory().unwrap();
        migrate(&connection).unwrap();
        let count: i64 = connection.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN
             ('recipes','resources','processes','operations','operation_dependencies',
              'servings','recipe_revisions','recipe_yields','nutrient_definitions','executions',
              'formulas','formula_ingredients','formula_runs')",
            [], |row| row.get(0)).unwrap();
        assert_eq!(count, 13);
    }
}
#[cfg(test)]
mod test;
