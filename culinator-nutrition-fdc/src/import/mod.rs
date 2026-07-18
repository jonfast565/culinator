mod source;

use crate::store::{BrandedFoodFields, SqliteNutritionCatalog};
use anyhow::{Context, Result};
use csv::StringRecord;
use culinator_models::{FoodNutrientRecord, FoodRecord, NutrientDefinition, NutritionImportStore};
use source::PreparedDataset;
use std::path::{Path, PathBuf};

pub const DEFAULT_FULL_DOWNLOAD_URL: &str =
    "https://fdc.nal.usda.gov/fdc-datasets/FoodData_Central_csv_2026-04-30.zip";

#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub release: String,
    pub replace: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildReport {
    pub foods: u64,
    pub nutrients: u64,
    pub food_nutrients: u64,
}

/// Download the USDA full CSV archive and build a searchable SQLite catalog.
pub fn download_and_build(
    destination: impl AsRef<Path>,
    release: &str,
    url: &str,
    replace: bool,
) -> Result<BuildReport> {
    let downloaded = tempfile::NamedTempFile::new()?;
    let mut response = reqwest::blocking::get(url)?.error_for_status()?;
    let mut target = downloaded.reopen()?;
    std::io::copy(&mut response, &mut target).context("download USDA archive")?;
    FdcDatabaseBuilder::build(&BuildOptions {
        source: downloaded.path().to_path_buf(),
        destination: destination.as_ref().to_path_buf(),
        release: release.to_owned(),
        replace,
    })
}

pub struct FdcDatabaseBuilder;

impl FdcDatabaseBuilder {
    pub fn build(options: &BuildOptions) -> Result<BuildReport> {
        let prepared = PreparedDataset::open(&options.source)?;
        if options.replace && options.destination.exists() {
            std::fs::remove_file(&options.destination)
                .with_context(|| format!("remove {}", options.destination.display()))?;
        }
        let mut store = SqliteNutritionCatalog::open(&options.destination)?;
        store.begin_import(&options.release)?;
        let mut report = BuildReport::default();

        import_nutrients(prepared.root(), &mut store, &mut report)?;
        import_foods(prepared.root(), &mut store, &mut report)?;
        import_branded_foods(prepared.root(), &mut store)?;
        import_food_nutrients(prepared.root(), &mut store, &mut report)?;
        store.finish_import()?;
        Ok(report)
    }
}

fn import_nutrients(
    root: &Path,
    store: &mut SqliteNutritionCatalog,
    report: &mut BuildReport,
) -> Result<()> {
    read_csv(root, "nutrient.csv", |headers, row| {
        store.upsert_nutrient(&NutrientDefinition {
            id: required_i64(headers, row, "id")?,
            name: required(headers, row, "name")?.to_owned(),
            unit_name: required(headers, row, "unit_name")?.to_owned(),
            number: optional(headers, row, "nutrient_nbr").map(ToOwned::to_owned),
            rank: optional_i64(headers, row, "rank")?,
        })?;
        report.nutrients += 1;
        Ok(())
    })
}

fn import_foods(
    root: &Path,
    store: &mut SqliteNutritionCatalog,
    report: &mut BuildReport,
) -> Result<()> {
    read_csv(root, "food.csv", |headers, row| {
        let fdc_id = required_i64(headers, row, "fdc_id")?;
        let description = optional(headers, row, "description")
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("USDA food {fdc_id}"));
        store.upsert_food(&FoodRecord {
            fdc_id,
            data_type: required(headers, row, "data_type")?.to_owned(),
            description,
            food_category_id: optional_i64(headers, row, "food_category_id")?,
            publication_date: optional(headers, row, "publication_date").map(ToOwned::to_owned),
            brand_owner: None,
            brand_name: None,
            gtin_upc: None,
            ingredients: None,
            serving_size: None,
            serving_size_unit: None,
        })?;
        report.foods += 1;
        Ok(())
    })
}

fn import_branded_foods(root: &Path, store: &mut SqliteNutritionCatalog) -> Result<()> {
    read_csv(root, "branded_food.csv", |headers, row| {
        store.update_branded_fields(
            required_i64(headers, row, "fdc_id")?,
            BrandedFoodFields {
                brand_owner: optional(headers, row, "brand_owner"),
                brand_name: optional(headers, row, "brand_name"),
                gtin_upc: optional(headers, row, "gtin_upc"),
                ingredients: optional(headers, row, "ingredients"),
                serving_size: optional_f64(headers, row, "serving_size")?,
                serving_size_unit: optional(headers, row, "serving_size_unit"),
            },
        )
    })
}

fn import_food_nutrients(
    root: &Path,
    store: &mut SqliteNutritionCatalog,
    report: &mut BuildReport,
) -> Result<()> {
    read_csv(root, "food_nutrient.csv", |headers, row| {
        store.upsert_food_nutrient(&FoodNutrientRecord {
            id: optional_i64(headers, row, "id")?,
            fdc_id: required_i64(headers, row, "fdc_id")?,
            nutrient_id: required_i64(headers, row, "nutrient_id")?,
            amount: optional_f64(headers, row, "amount")?,
            data_points: optional_i64(headers, row, "data_points")?,
            derivation_id: optional_i64(headers, row, "derivation_id")?,
            min: optional_f64(headers, row, "min")?,
            max: optional_f64(headers, row, "max")?,
            median: optional_f64(headers, row, "median")?,
        })?;
        report.food_nutrients += 1;
        Ok(())
    })
}

fn read_csv<F>(root: &Path, name: &str, mut callback: F) -> Result<()>
where
    F: FnMut(&StringRecord, &StringRecord) -> Result<()>,
{
    let path = root.join(name);
    if !path.exists() {
        return Ok(());
    }
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_path(&path)
        .with_context(|| format!("open {}", path.display()))?;
    let headers = reader.headers()?.clone();
    for row in reader.records() {
        callback(&headers, &row?)?;
    }
    Ok(())
}

fn index(headers: &StringRecord, name: &str) -> Result<usize> {
    headers
        .iter()
        .position(|value| value == name)
        .with_context(|| format!("missing CSV column {name}"))
}
fn required<'a>(headers: &StringRecord, row: &'a StringRecord, name: &str) -> Result<&'a str> {
    row.get(index(headers, name)?)
        .filter(|v| !v.is_empty())
        .with_context(|| format!("missing value for {name}"))
}
fn optional<'a>(headers: &StringRecord, row: &'a StringRecord, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .position(|value| value == name)
        .and_then(|i| row.get(i))
        .filter(|v| !v.is_empty())
}
fn parse_loose_i64(raw: &str) -> Result<i64> {
    let raw = raw.trim();
    raw.parse::<i64>()
        .or_else(|_| raw.parse::<f64>().map(|value| value.trunc() as i64))
        .with_context(|| format!("parse integer from {raw:?}"))
}

fn required_i64(headers: &StringRecord, row: &StringRecord, name: &str) -> Result<i64> {
    parse_loose_i64(required(headers, row, name)?)
}
fn optional_i64(headers: &StringRecord, row: &StringRecord, name: &str) -> Result<Option<i64>> {
    Ok(optional(headers, row, name)
        .and_then(|raw| parse_loose_i64(raw).ok()))
}
fn optional_f64(headers: &StringRecord, row: &StringRecord, name: &str) -> Result<Option<f64>> {
    optional(headers, row, name)
        .map(str::parse)
        .transpose()
        .map_err(Into::into)
}

#[cfg(test)]
mod test;
