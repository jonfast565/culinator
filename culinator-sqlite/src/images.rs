use culinator_models::{
    ApplicationError, RecipeImageAsset, RecipeImageData, RecipeImageRepository,
    UploadRecipeImageRequest,
};
use rusqlite::{Connection, OptionalExtension, params};
use uuid::Uuid;

use crate::repository::{SqliteCatalogRepository, parse_uuid};

impl RecipeImageRepository for SqliteCatalogRepository {
    fn list_recipe_images(
        &self,
        recipe_id: Uuid,
    ) -> Result<Vec<RecipeImageAsset>, ApplicationError> {
        self.with_connection(|connection| list_assets(connection, &recipe_id.to_string()))
    }

    fn get_recipe_image(
        &self,
        recipe_id: Uuid,
        handle: &str,
    ) -> Result<Option<RecipeImageData>, ApplicationError> {
        self.with_connection(|connection| get_image(connection, &recipe_id.to_string(), handle))
    }

    fn upload_recipe_image(
        &self,
        recipe_id: Uuid,
        input: UploadRecipeImageRequest,
    ) -> Result<RecipeImageAsset, ApplicationError> {
        let handle = input
            .handle
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| format!("img_{}_{}", input.role, Uuid::new_v4().simple()));
        // base64 expands bytes ~4:3; estimate the decoded size for display.
        let byte_size = (input.data_base64.len() as i64 * 3) / 4;
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO recipe_images
                 (id, recipe_id, handle, role, operation_symbol, media_type, file_name, data_base64, byte_size)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(recipe_id, handle) DO UPDATE SET
                   role=excluded.role,
                   operation_symbol=excluded.operation_symbol,
                   media_type=excluded.media_type,
                   file_name=excluded.file_name,
                   data_base64=excluded.data_base64,
                   byte_size=excluded.byte_size,
                   created_at=CURRENT_TIMESTAMP",
                params![
                    Uuid::new_v4().to_string(),
                    recipe_id.to_string(),
                    handle,
                    input.role,
                    input.operation_symbol,
                    input.media_type,
                    input.file_name,
                    input.data_base64,
                    byte_size,
                ],
            )?;
            Ok(())
        })?;
        self.get_recipe_image(recipe_id, &handle)?
            .map(|data| data.asset)
            .ok_or_else(|| {
                ApplicationError::Internal("recipe image could not be read after upload".to_owned())
            })
    }

    fn delete_recipe_image(&self, recipe_id: Uuid, handle: &str) -> Result<bool, ApplicationError> {
        self.with_connection(|connection| {
            Ok(connection.execute(
                "DELETE FROM recipe_images WHERE recipe_id=?1 AND handle=?2",
                params![recipe_id.to_string(), handle],
            )? > 0)
        })
    }
}

fn list_assets(
    connection: &Connection,
    recipe_id: &str,
) -> Result<Vec<RecipeImageAsset>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT id, recipe_id, handle, role, operation_symbol, media_type, file_name, byte_size, created_at
         FROM recipe_images
         WHERE recipe_id=?1
         ORDER BY created_at",
    )?;
    statement.query_map([recipe_id], map_asset_row)?.collect()
}

fn get_image(
    connection: &Connection,
    recipe_id: &str,
    handle: &str,
) -> Result<Option<RecipeImageData>, rusqlite::Error> {
    connection
        .query_row(
            "SELECT id, recipe_id, handle, role, operation_symbol, media_type, file_name, byte_size, created_at, data_base64
             FROM recipe_images
             WHERE recipe_id=?1 AND handle=?2",
            params![recipe_id, handle],
            |row| {
                Ok(RecipeImageData {
                    asset: map_asset_row(row)?,
                    data_base64: row.get(9)?,
                })
            },
        )
        .optional()
}

fn map_asset_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RecipeImageAsset> {
    Ok(RecipeImageAsset {
        id: parse_uuid(row.get::<_, String>(0)?)?,
        recipe_id: parse_uuid(row.get::<_, String>(1)?)?,
        handle: row.get(2)?,
        role: row.get(3)?,
        operation_symbol: row.get(4)?,
        media_type: row.get(5)?,
        file_name: row.get(6)?,
        byte_size: row.get(7)?,
        created_at: row.get(8)?,
    })
}

#[cfg(test)]
#[path = "images/test.rs"]
mod test;
