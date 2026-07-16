use culinator_models::{
    ApplicationError, LinkResourceNutritionRequest, ResourceNutritionLink,
    ResourceNutritionRepository,
};
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::repository::{SqliteCatalogRepository, parse_uuid};

impl ResourceNutritionRepository for SqliteCatalogRepository {
    fn list_links_for_recipe(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<ResourceNutritionLink>, ApplicationError> {
        self.with_connection(|connection| list_links(connection, &recipe_id.to_string()))
    }

    fn get_link(
        &self,
        recipe_id: Uuid,
        resource_symbol: &str,
    ) -> Result<Option<ResourceNutritionLink>, ApplicationError> {
        self.with_connection(|connection| {
            get_link(connection, &recipe_id.to_string(), resource_symbol)
        })
    }

    fn link_resource(
        &self,
        recipe_id: Uuid,
        input: LinkResourceNutritionRequest,
        food_description: String,
    ) -> Result<ResourceNutritionLink, ApplicationError> {
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO resource_nutrition_links
                 (recipe_id, resource_symbol, fdc_id, food_description)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(recipe_id, resource_symbol) DO UPDATE SET
                   fdc_id=excluded.fdc_id,
                   food_description=excluded.food_description,
                   linked_at=CURRENT_TIMESTAMP",
                params![
                    recipe_id.to_string(),
                    input.resource_symbol.clone(),
                    input.fdc_id,
                    food_description
                ],
            )?;
            Ok(())
        })?;
        self.get_link(recipe_id, &input.resource_symbol)?
            .ok_or_else(|| {
                ApplicationError::Internal("nutrition link could not be read after save".to_owned())
            })
    }

    fn unlink_resource(
        &self,
        recipe_id: Uuid,
        resource_symbol: &str,
    ) -> Result<bool, ApplicationError> {
        self.with_connection(|connection| {
            Ok(connection.execute(
                "DELETE FROM resource_nutrition_links WHERE recipe_id=?1 AND resource_symbol=?2",
                params![recipe_id.to_string(), resource_symbol],
            )? > 0)
        })
    }
}

pub fn list_links(
    connection: &Connection,
    recipe_id: &str,
) -> Result<Vec<ResourceNutritionLink>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT recipe_id, resource_symbol, fdc_id, food_description, linked_at
         FROM resource_nutrition_links
         WHERE recipe_id=?1
         ORDER BY resource_symbol",
    )?;
    statement.query_map([recipe_id], map_link_row)?.collect()
}

pub fn get_link(
    connection: &Connection,
    recipe_id: &str,
    resource_symbol: &str,
) -> Result<Option<ResourceNutritionLink>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT recipe_id, resource_symbol, fdc_id, food_description, linked_at
             FROM resource_nutrition_links
             WHERE recipe_id=?1 AND resource_symbol=?2",
            params![recipe_id, resource_symbol],
            map_link_row,
        )
        .optional()
}

fn map_link_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ResourceNutritionLink> {
    Ok(ResourceNutritionLink {
        recipe_id: parse_uuid(row.get::<_, String>(0)?)?,
        resource_symbol: row.get(1)?,
        fdc_id: row.get(2)?,
        food_description: row.get(3)?,
        linked_at: row.get(4)?,
    })
}

#[cfg(test)]
#[path = "nutrition/test.rs"]
mod test;
