use crate::{
    delete_recipe, get_formula, list_formulas_for_recipe, list_recipe_books, migrate,
    move_recipe_to_book, save_formula, save_formula_run, save_recipe, save_recipe_book,
};
use culinator_core::{Formula, FormulaResult, Recipe, RecipeBook, TypeRef};
use culinator_models::{
    ApplicationError, FormulaRepository, NewRecipe, NewRecipeBook, RecipeBookRepository,
    RecipeBookSummary, RecipeDocument, RecipeRepository, RecipeSummary,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteCatalogRepository {
    path: PathBuf,
}

impl SqliteCatalogRepository {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn initialize(&self) -> Result<(), ApplicationError> {
        self.with_connection(|_| Ok(()))
    }

    pub(crate) fn with_connection<T>(
        &self,
        operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error>,
    ) -> Result<T, ApplicationError> {
        let mut connection = Connection::open(&self.path).map_err(map_error)?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .map_err(map_error)?;
        migrate(&connection).map_err(map_error)?;
        operation(&mut connection).map_err(map_error)
    }
}

impl RecipeRepository for SqliteCatalogRepository {
    fn list_recipes(&self) -> Result<Vec<RecipeSummary>, ApplicationError> {
        self.with_connection(|connection| {
            let mut statement = connection.prepare(
                "SELECT id, book_id, symbol, title, protocol_version, updated_at
                 FROM recipes ORDER BY book_id, book_position, title",
            )?;
            statement
                .query_map([], |row| {
                    Ok(RecipeSummary {
                        id: parse_uuid(row.get::<_, String>(0)?)?,
                        book_id: row
                            .get::<_, Option<String>>(1)?
                            .map(parse_uuid)
                            .transpose()?,
                        symbol: row.get(2)?,
                        title: row.get(3)?,
                        protocol_version: row.get(4)?,
                        updated_at: row.get(5)?,
                    })
                })?
                .collect()
        })
    }

    fn get_recipe(&self, id: Uuid) -> Result<Option<RecipeDocument>, ApplicationError> {
        self.with_connection(|connection| {
            connection
                .query_row(
                    "SELECT id, book_id, symbol, title, protocol_version, source_text, updated_at
                     FROM recipes WHERE id=?1",
                    [id.to_string()],
                    |row| {
                        Ok(RecipeDocument {
                            id: parse_uuid(row.get::<_, String>(0)?)?,
                            book_id: row
                                .get::<_, Option<String>>(1)?
                                .map(parse_uuid)
                                .transpose()?,
                            symbol: row.get(2)?,
                            title: row.get(3)?,
                            protocol_version: row.get(4)?,
                            source_text: row.get(5)?,
                            updated_at: row.get(6)?,
                        })
                    },
                )
                .optional()
        })
    }

    fn create_recipe(&self, input: NewRecipe) -> Result<RecipeDocument, ApplicationError> {
        let id = Uuid::new_v4();
        self.with_connection(|connection| {
            connection.execute(
                "INSERT INTO recipes
                 (id, book_id, symbol, title, protocol_version, declared_type_json, source_text)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    id.to_string(),
                    input.book_id.map(|value| value.to_string()),
                    input.symbol,
                    input.title,
                    input.protocol_version,
                    serde_json::to_string(&TypeRef::named("Recipe")).expect("type serialization"),
                    input.source_text,
                ],
            )?;
            Ok(())
        })?;
        self.get_recipe(id)?.ok_or_else(|| {
            ApplicationError::Internal("created recipe could not be read".to_owned())
        })
    }

    fn save_recipe(
        &self,
        recipe: &Recipe,
        source_text: &str,
    ) -> Result<RecipeDocument, ApplicationError> {
        self.with_connection(|connection| save_recipe(connection, recipe, source_text))?;
        self.get_recipe(recipe.id)?
            .ok_or_else(|| ApplicationError::Internal("saved recipe could not be read".to_owned()))
    }

    fn delete_recipe(&self, id: Uuid) -> Result<bool, ApplicationError> {
        self.with_connection(|connection| delete_recipe(connection, &id.to_string()))
    }

    fn move_recipe(
        &self,
        id: Uuid,
        book_id: Option<Uuid>,
        position: i64,
    ) -> Result<bool, ApplicationError> {
        self.with_connection(|connection| {
            let book = book_id.map(|value| value.to_string());
            move_recipe_to_book(connection, &id.to_string(), book.as_deref(), position)
        })
    }
}

impl RecipeBookRepository for SqliteCatalogRepository {
    fn list_recipe_books(&self) -> Result<Vec<RecipeBookSummary>, ApplicationError> {
        self.with_connection(|connection| list_recipe_books(connection))?
            .into_iter()
            .map(|book| {
                Ok(RecipeBookSummary {
                    id: Uuid::parse_str(&book.id)
                        .map_err(|error| ApplicationError::Persistence(error.to_string()))?,
                    symbol: book.symbol,
                    title: book.title,
                    description: book.description,
                    protocol_version: book.protocol_version,
                    recipe_count: book.recipe_count,
                    updated_at: book.updated_at,
                })
            })
            .collect()
    }

    fn create_recipe_book(
        &self,
        input: NewRecipeBook,
    ) -> Result<RecipeBookSummary, ApplicationError> {
        let id = Uuid::new_v4();
        let book = RecipeBook {
            id,
            symbol: input.symbol.unwrap_or_else(|| slug(&input.title)),
            declared_type: TypeRef::named("RecipeBook"),
            title: input.title,
            description: input.description,
            protocol_version: "0.3".to_owned(),
            recipes: Vec::new(),
            properties: BTreeMap::new(),
        };
        self.save_recipe_book(&book)?;
        self.list_recipe_books()?
            .into_iter()
            .find(|candidate| candidate.id == id)
            .ok_or_else(|| ApplicationError::Internal("created book could not be read".to_owned()))
    }

    fn update_recipe_book(
        &self,
        id: Uuid,
        input: NewRecipeBook,
    ) -> Result<Option<RecipeBookSummary>, ApplicationError> {
        let symbol = input.symbol.unwrap_or_else(|| slug(&input.title));
        let changed = self.with_connection(|connection| {
            connection.execute(
                "UPDATE recipe_books
                 SET symbol=?2,title=?3,description=?4,updated_at=CURRENT_TIMESTAMP
                 WHERE id=?1",
                params![id.to_string(), symbol, input.title, input.description],
            )
        })?;
        if changed == 0 {
            return Ok(None);
        }
        Ok(self
            .list_recipe_books()?
            .into_iter()
            .find(|book| book.id == id))
    }

    fn save_recipe_book(&self, book: &RecipeBook) -> Result<(), ApplicationError> {
        self.with_connection(|connection| save_recipe_book(connection, book))
    }

    fn delete_recipe_book(&self, id: Uuid) -> Result<bool, ApplicationError> {
        self.with_connection(|connection| {
            Ok(connection.execute("DELETE FROM recipe_books WHERE id=?1", [id.to_string()])? > 0)
        })
    }
}

impl FormulaRepository for SqliteCatalogRepository {
    fn save_formula(&self, formula: &Formula) -> Result<(), ApplicationError> {
        self.with_connection(|connection| save_formula(connection, formula))
    }

    fn get_formula(&self, id: Uuid) -> Result<Option<Formula>, ApplicationError> {
        self.with_connection(|connection| get_formula(connection, &id.to_string()))
    }

    fn list_formulas_for_recipe(&self, recipe_id: Uuid) -> Result<Vec<Formula>, ApplicationError> {
        self.with_connection(|connection| {
            list_formulas_for_recipe(connection, &recipe_id.to_string())
        })
    }

    fn save_formula_run(
        &self,
        formula_id: Uuid,
        target_mass_grams: f64,
        result: &FormulaResult,
    ) -> Result<(), ApplicationError> {
        self.with_connection(|connection| {
            save_formula_run(
                connection,
                &formula_id.to_string(),
                target_mass_grams,
                result,
            )
            .map(|_| ())
        })
    }
}

pub(crate) fn parse_uuid(value: String) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(&value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            value.len(),
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

pub(crate) fn map_error(error: rusqlite::Error) -> ApplicationError {
    ApplicationError::Persistence(error.to_string())
}

fn slug(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

#[cfg(test)]
mod test;
