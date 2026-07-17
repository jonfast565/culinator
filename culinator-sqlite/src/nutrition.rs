use culinator_models::{
    ApplicationError, IngredientManualNutrition, LinkResourceNutritionRequest, NutritionFacts,
    RecipeNutritionState, ResourceNutritionLink, ResourceNutritionRepository,
    SaveIngredientManualNutritionRequest, SaveRecipeNutritionRequest,
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

    fn get_recipe_nutrition(
        &self,
        recipe_id: Uuid,
    ) -> Result<RecipeNutritionState, ApplicationError> {
        self.with_connection(|connection| {
            get_recipe_nutrition_state(connection, &recipe_id.to_string())
        })
    }

    fn save_recipe_nutrition(
        &self,
        recipe_id: Uuid,
        input: SaveRecipeNutritionRequest,
    ) -> Result<RecipeNutritionState, ApplicationError> {
        let facts_json = input
            .facts
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|error| ApplicationError::Internal(error.to_string()))?;
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO recipe_nutrition (recipe_id, manual_override, facts_json)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(recipe_id) DO UPDATE SET
                   manual_override=excluded.manual_override,
                   facts_json=excluded.facts_json,
                   updated_at=CURRENT_TIMESTAMP",
                params![
                    recipe_id.to_string(),
                    i64::from(input.manual_override),
                    facts_json
                ],
            )?;
            Ok(())
        })?;
        self.get_recipe_nutrition(recipe_id)
    }

    fn list_manual_ingredient_nutrition(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<IngredientManualNutrition>, ApplicationError> {
        self.with_connection(|connection| {
            list_manual_ingredient_nutrition(connection, &recipe_id.to_string())
        })
    }

    fn save_manual_ingredient_nutrition(
        &self,
        recipe_id: Uuid,
        input: SaveIngredientManualNutritionRequest,
    ) -> Result<IngredientManualNutrition, ApplicationError> {
        if input.resource_symbol.trim().is_empty() {
            return Err(ApplicationError::InvalidInput(
                "resource symbol cannot be empty".to_owned(),
            ));
        }
        let facts_json = serde_json::to_string(&input.facts_per_100g)
            .map_err(|error| ApplicationError::Internal(error.to_string()))?;
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO resource_nutrition_manual
                 (recipe_id, resource_symbol, facts_per_100g_json)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(recipe_id, resource_symbol) DO UPDATE SET
                   facts_per_100g_json=excluded.facts_per_100g_json,
                   updated_at=CURRENT_TIMESTAMP",
                params![
                    recipe_id.to_string(),
                    input.resource_symbol.clone(),
                    facts_json
                ],
            )?;
            Ok(())
        })?;
        self.with_connection(|connection| {
            get_manual_ingredient_nutrition(
                connection,
                &recipe_id.to_string(),
                &input.resource_symbol,
            )?
            .ok_or_else(|| {
                rusqlite::Error::InvalidQuery
            })
        })
    }

    fn delete_manual_ingredient_nutrition(
        &self,
        recipe_id: Uuid,
        resource_symbol: &str,
    ) -> Result<bool, ApplicationError> {
        self.with_connection(|connection| {
            Ok(connection.execute(
                "DELETE FROM resource_nutrition_manual WHERE recipe_id=?1 AND resource_symbol=?2",
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

pub fn get_recipe_nutrition_state(
    connection: &Connection,
    recipe_id: &str,
) -> Result<RecipeNutritionState, rusqlite::Error> {
    let links = list_links(connection, recipe_id)?;
    let manual_ingredients = list_manual_ingredient_nutrition(connection, recipe_id)?;
    let (manual_override, manual_facts) = connection
        .query_row(
            "SELECT manual_override, facts_json FROM recipe_nutrition WHERE recipe_id=?1",
            [recipe_id],
            |row| {
                let override_flag: i64 = row.get(0)?;
                let facts_json: Option<String> = row.get(1)?;
                let facts = facts_json
                    .as_deref()
                    .map(parse_nutrition_facts_json)
                    .transpose()
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            1,
                            "facts_json".to_owned(),
                            rusqlite::types::Type::Text,
                        )
                    })?;
                Ok((override_flag != 0, facts))
            },
        )
        .optional()?
        .unwrap_or((false, None));
    Ok(RecipeNutritionState {
        recipe_id: parse_uuid(recipe_id.to_owned())?,
        links,
        manual_ingredients,
        manual_override,
        manual_facts,
    })
}

pub fn list_manual_ingredient_nutrition(
    connection: &Connection,
    recipe_id: &str,
) -> Result<Vec<IngredientManualNutrition>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT recipe_id, resource_symbol, facts_per_100g_json, updated_at
         FROM resource_nutrition_manual
         WHERE recipe_id=?1
         ORDER BY resource_symbol",
    )?;
    statement
        .query_map([recipe_id], |row| {
            let facts_json: String = row.get(2)?;
            Ok(IngredientManualNutrition {
                recipe_id: parse_uuid(row.get::<_, String>(0)?)?,
                resource_symbol: row.get(1)?,
                facts_per_100g: parse_nutrition_facts_json(&facts_json).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        2,
                        "facts_per_100g_json".to_owned(),
                        rusqlite::types::Type::Text,
                    )
                })?,
                updated_at: row.get(3)?,
            })
        })?
        .collect()
}

pub fn get_manual_ingredient_nutrition(
    connection: &Connection,
    recipe_id: &str,
    resource_symbol: &str,
) -> Result<Option<IngredientManualNutrition>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT recipe_id, resource_symbol, facts_per_100g_json, updated_at
             FROM resource_nutrition_manual
             WHERE recipe_id=?1 AND resource_symbol=?2",
            params![recipe_id, resource_symbol],
            |row| {
                let facts_json: String = row.get(2)?;
                Ok(IngredientManualNutrition {
                    recipe_id: parse_uuid(row.get::<_, String>(0)?)?,
                    resource_symbol: row.get(1)?,
                    facts_per_100g: parse_nutrition_facts_json(&facts_json).map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            2,
                            "facts_per_100g_json".to_owned(),
                            rusqlite::types::Type::Text,
                        )
                    })?,
                    updated_at: row.get(3)?,
                })
            },
        )
        .optional()
}

fn parse_nutrition_facts_json(value: &str) -> Result<NutritionFacts, serde_json::Error> {
    serde_json::from_str(value)
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
