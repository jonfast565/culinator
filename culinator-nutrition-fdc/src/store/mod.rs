use anyhow::Result as AnyResult;
use culinator_models::{
    ApplicationError, FoodNutrientRecord, FoodRecord, NutrientDefinition, NutritionCatalog,
    NutritionImportStore, NutritionSearchResult,
};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;
use std::sync::Mutex;

const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
CREATE TABLE IF NOT EXISTS metadata(key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS nutrients(
  id INTEGER PRIMARY KEY,
  number TEXT,
  name TEXT NOT NULL,
  unit_name TEXT NOT NULL,
  rank INTEGER
);
CREATE TABLE IF NOT EXISTS foods(
  fdc_id INTEGER PRIMARY KEY,
  data_type TEXT NOT NULL,
  description TEXT NOT NULL,
  food_category_id INTEGER,
  publication_date TEXT,
  brand_owner TEXT,
  brand_name TEXT,
  gtin_upc TEXT,
  ingredients TEXT,
  serving_size REAL,
  serving_size_unit TEXT
);
CREATE TABLE IF NOT EXISTS food_nutrients(
  id INTEGER PRIMARY KEY,
  fdc_id INTEGER NOT NULL REFERENCES foods(fdc_id) ON DELETE CASCADE,
  nutrient_id INTEGER NOT NULL REFERENCES nutrients(id),
  amount REAL,
  data_points INTEGER,
  derivation_id INTEGER,
  min REAL,
  max REAL,
  median REAL,
  UNIQUE(fdc_id, nutrient_id, id)
);
CREATE INDEX IF NOT EXISTS idx_food_nutrients_food ON food_nutrients(fdc_id);
CREATE INDEX IF NOT EXISTS idx_food_nutrients_nutrient ON food_nutrients(nutrient_id);
CREATE INDEX IF NOT EXISTS idx_foods_data_type ON foods(data_type);
CREATE INDEX IF NOT EXISTS idx_foods_gtin ON foods(gtin_upc);
CREATE VIRTUAL TABLE IF NOT EXISTS foods_fts USING fts5(
  description, brand_owner, brand_name, ingredients,
  content='foods', content_rowid='fdc_id', tokenize='unicode61 remove_diacritics 2'
);
CREATE TRIGGER IF NOT EXISTS foods_ai AFTER INSERT ON foods BEGIN
  INSERT INTO foods_fts(rowid, description, brand_owner, brand_name, ingredients)
  VALUES (new.fdc_id, new.description, new.brand_owner, new.brand_name, new.ingredients);
END;
CREATE TRIGGER IF NOT EXISTS foods_ad AFTER DELETE ON foods BEGIN
  INSERT INTO foods_fts(foods_fts, rowid, description, brand_owner, brand_name, ingredients)
  VALUES ('delete', old.fdc_id, old.description, old.brand_owner, old.brand_name, old.ingredients);
END;
CREATE TRIGGER IF NOT EXISTS foods_au AFTER UPDATE ON foods BEGIN
  INSERT INTO foods_fts(foods_fts, rowid, description, brand_owner, brand_name, ingredients)
  VALUES ('delete', old.fdc_id, old.description, old.brand_owner, old.brand_name, old.ingredients);
  INSERT INTO foods_fts(rowid, description, brand_owner, brand_name, ingredients)
  VALUES (new.fdc_id, new.description, new.brand_owner, new.brand_name, new.ingredients);
END;
"#;

#[derive(Debug, Clone, Copy)]
pub struct BrandedFoodFields<'a> {
    pub brand_owner: Option<&'a str>,
    pub brand_name: Option<&'a str>,
    pub gtin_upc: Option<&'a str>,
    pub ingredients: Option<&'a str>,
    pub serving_size: Option<f64>,
    pub serving_size_unit: Option<&'a str>,
}

pub struct SqliteNutritionCatalog {
    connection: Mutex<Connection>,
    imported_rows: usize,
}

impl SqliteNutritionCatalog {
    pub fn open(path: impl AsRef<Path>) -> AnyResult<Self> {
        let connection = Connection::open(path)?;
        connection.execute_batch(SCHEMA)?;
        Ok(Self {
            connection: Mutex::new(connection),
            imported_rows: 0,
        })
    }

    pub fn update_branded_fields(
        &mut self,
        fdc_id: i64,
        fields: BrandedFoodFields<'_>,
    ) -> AnyResult<()> {
        self.lock().execute(
            "UPDATE foods SET brand_owner=?2, brand_name=?3, gtin_upc=?4, ingredients=?5, serving_size=?6, serving_size_unit=?7 WHERE fdc_id=?1",
            params![
                fdc_id,
                fields.brand_owner,
                fields.brand_name,
                fields.gtin_upc,
                fields.ingredients,
                fields.serving_size,
                fields.serving_size_unit
            ],
        )?;
        Ok(())
    }

    fn persistence(error: impl std::fmt::Display) -> ApplicationError {
        ApplicationError::Persistence(error.to_string())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.connection
            .lock()
            .expect("nutrition catalog connection mutex poisoned")
    }
}

impl NutritionImportStore for SqliteNutritionCatalog {
    fn begin_import(&mut self, release: &str) -> std::result::Result<(), ApplicationError> {
        self.lock()
            .execute_batch("PRAGMA foreign_keys=OFF; PRAGMA defer_foreign_keys=ON;")
            .map_err(Self::persistence)?;
        self.lock().execute(
            "INSERT INTO metadata(key,value) VALUES('fdc_release',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [release],
        ).map_err(Self::persistence)?;
        self.lock().execute(
            "INSERT INTO metadata(key,value) VALUES('import_complete','false') ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [],
        ).map_err(Self::persistence)?;
        Ok(())
    }

    fn upsert_nutrient(
        &mut self,
        item: &NutrientDefinition,
    ) -> std::result::Result<(), ApplicationError> {
        self.lock().execute(
            "INSERT INTO nutrients(id,number,name,unit_name,rank) VALUES(?1,?2,?3,?4,?5)
             ON CONFLICT(id) DO UPDATE SET number=excluded.number,name=excluded.name,unit_name=excluded.unit_name,rank=excluded.rank",
            params![item.id, item.number, item.name, item.unit_name, item.rank],
        ).map_err(Self::persistence)?;
        Ok(())
    }

    fn upsert_food(&mut self, item: &FoodRecord) -> std::result::Result<(), ApplicationError> {
        self.lock().execute(
            "INSERT INTO foods(fdc_id,data_type,description,food_category_id,publication_date,brand_owner,brand_name,gtin_upc,ingredients,serving_size,serving_size_unit)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
             ON CONFLICT(fdc_id) DO UPDATE SET data_type=excluded.data_type,description=excluded.description,food_category_id=excluded.food_category_id,publication_date=excluded.publication_date",
            params![item.fdc_id,item.data_type,item.description,item.food_category_id,item.publication_date,item.brand_owner,item.brand_name,item.gtin_upc,item.ingredients,item.serving_size,item.serving_size_unit],
        ).map_err(Self::persistence)?;
        self.imported_rows += 1;
        if self.imported_rows.is_multiple_of(50_000) {
            self.lock()
                .execute_batch("PRAGMA wal_checkpoint(PASSIVE);")
                .map_err(Self::persistence)?;
        }
        Ok(())
    }

    fn upsert_food_nutrient(
        &mut self,
        item: &FoodNutrientRecord,
    ) -> std::result::Result<(), ApplicationError> {
        self.lock().execute(
            "INSERT OR REPLACE INTO food_nutrients(id,fdc_id,nutrient_id,amount,data_points,derivation_id,min,max,median)
             VALUES(COALESCE(?1, (SELECT IFNULL(MAX(id),0)+1 FROM food_nutrients)),?2,?3,?4,?5,?6,?7,?8,?9)",
            params![item.id,item.fdc_id,item.nutrient_id,item.amount,item.data_points,item.derivation_id,item.min,item.max,item.median],
        ).map_err(Self::persistence)?;
        Ok(())
    }

    fn finish_import(&mut self) -> std::result::Result<(), ApplicationError> {
        self.lock().execute(
            "INSERT INTO metadata(key,value) VALUES('import_complete','true') ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [],
        ).map_err(Self::persistence)?;
        self.lock().execute_batch(
            "PRAGMA foreign_keys=ON; INSERT INTO foods_fts(foods_fts) VALUES('rebuild'); ANALYZE; PRAGMA optimize; PRAGMA wal_checkpoint(TRUNCATE);",
        ).map_err(Self::persistence)
    }
}

impl NutritionCatalog for SqliteNutritionCatalog {
    fn search_foods(
        &self,
        query: &str,
        limit: usize,
    ) -> std::result::Result<Vec<NutritionSearchResult>, ApplicationError> {
        let connection = self.lock();
        let mut statement = connection.prepare(
            "SELECT f.fdc_id,f.description,f.data_type,f.brand_owner,f.serving_size,f.serving_size_unit
             FROM foods_fts x JOIN foods f ON f.fdc_id=x.rowid WHERE foods_fts MATCH ?1 ORDER BY rank LIMIT ?2",
        ).map_err(Self::persistence)?;
        statement
            .query_map(params![query, limit as i64], |row| {
                Ok(NutritionSearchResult {
                    fdc_id: row.get(0)?,
                    description: row.get(1)?,
                    data_type: row.get(2)?,
                    brand_owner: row.get(3)?,
                    serving_size: row.get(4)?,
                    serving_size_unit: row.get(5)?,
                })
            })
            .map_err(Self::persistence)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Self::persistence)
    }

    fn food(&self, fdc_id: i64) -> std::result::Result<Option<FoodRecord>, ApplicationError> {
        self.lock().query_row(
            "SELECT fdc_id,data_type,description,food_category_id,publication_date,brand_owner,brand_name,gtin_upc,ingredients,serving_size,serving_size_unit FROM foods WHERE fdc_id=?1",
            [fdc_id], |row| Ok(FoodRecord {
                fdc_id: row.get(0)?, data_type: row.get(1)?, description: row.get(2)?, food_category_id: row.get(3)?, publication_date: row.get(4)?,
                brand_owner: row.get(5)?, brand_name: row.get(6)?, gtin_upc: row.get(7)?, ingredients: row.get(8)?, serving_size: row.get(9)?, serving_size_unit: row.get(10)?,
            }),
        ).optional().map_err(Self::persistence)
    }

    fn nutrients_for_food(
        &self,
        fdc_id: i64,
    ) -> std::result::Result<Vec<FoodNutrientRecord>, ApplicationError> {
        let connection = self.lock();
        let mut statement = connection.prepare(
            "SELECT id,fdc_id,nutrient_id,amount,data_points,derivation_id,min,max,median FROM food_nutrients WHERE fdc_id=?1",
        ).map_err(Self::persistence)?;
        statement
            .query_map([fdc_id], |row| {
                Ok(FoodNutrientRecord {
                    id: row.get(0)?,
                    fdc_id: row.get(1)?,
                    nutrient_id: row.get(2)?,
                    amount: row.get(3)?,
                    data_points: row.get(4)?,
                    derivation_id: row.get(5)?,
                    min: row.get(6)?,
                    max: row.get(7)?,
                    median: row.get(8)?,
                })
            })
            .map_err(Self::persistence)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Self::persistence)
    }
}

#[cfg(test)]
mod test;
