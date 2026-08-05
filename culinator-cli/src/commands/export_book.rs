use anyhow::{Context, Result};
use culinator_application::{ExportService, NutritionService};
use culinator_export::StaticRecipeBookExporter;
use culinator_models::{BookExportFormat, BookExportOptions};
use culinator_parser::CulinatorParser;
use culinator_sqlite::SqliteCatalogRepository;
use std::{fs, path::Path, sync::{Arc, RwLock}};
use uuid::Uuid;

pub fn export_book(database: &Path, book_id: Uuid, output: &Path) -> Result<()> {
    let repository = Arc::new(SqliteCatalogRepository::new(database));
    repository
        .initialize()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let parser = Arc::new(CulinatorParser);
    let nutrition = NutritionService::new(
        repository.clone(),
        repository.clone(),
        parser.clone(),
        Arc::new(RwLock::new(None)),
    );
    let service = ExportService::new(
        repository.clone(),
        repository,
        parser,
        Arc::new(culinator_export::StaticRecipeExporter),
        Arc::new(StaticRecipeBookExporter),
        nutrition,
    );
    let options = BookExportOptions {
        formats: vec![
            BookExportFormat::Epub,
            BookExportFormat::PrintHtml,
            BookExportFormat::Web,
        ],
        ..BookExportOptions::default()
    };
    let bundle = service
        .export_book(book_id, &options)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    fs::write(output, bundle.archive).with_context(|| format!("write {}", output.display()))?;
    println!("Exported {} to {}", bundle.file_name, output.display());
    Ok(())
}
