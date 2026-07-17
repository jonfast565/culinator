mod epub;
mod print_html;
mod site;
mod util;

use culinator_core::{Recipe, RecipeBook};
use culinator_models::{
    ApplicationError, BookExportFormat, BookExportOptions, ExportFile, NutritionFacts,
    RecipeBookExporter, RecipeExportBundle, RecipeExportOptions,
};
use std::io::{Cursor, Write};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

#[derive(Debug, Default, Clone, Copy)]
pub struct StaticRecipeBookExporter;

impl RecipeBookExporter for StaticRecipeBookExporter {
    fn export_book(
        &self,
        book: &RecipeBook,
        recipes: &[(Recipe, String)],
        options: &BookExportOptions,
    ) -> Result<RecipeExportBundle, ApplicationError> {
        let formats = if options.formats.is_empty() {
            vec![BookExportFormat::Epub, BookExportFormat::PrintHtml]
        } else {
            options.formats.clone()
        };
        let title = options.title.as_deref().unwrap_or(&book.title).to_owned();
        let mut files = Vec::new();

        for format in formats {
            match format {
                BookExportFormat::Epub => files.push(file(
                    "book.epub",
                    "application/epub+zip",
                    epub::render(&title, recipes, options)?,
                )),
                BookExportFormat::PrintHtml => files.push(file(
                    "print.html",
                    "text/html; charset=utf-8",
                    print_html::render(&title, recipes, options).into_bytes(),
                )),
                BookExportFormat::Web => files.extend(site::render(&title, recipes, options)?),
            }
        }

        let archive = zip_files(&files)?;
        Ok(RecipeExportBundle {
            file_name: format!("{}.zip", util::slug(&title)),
            files,
            archive,
        })
    }
}

pub(crate) fn recipe_options_from_book(options: &BookExportOptions) -> RecipeExportOptions {
    RecipeExportOptions {
        site_title: options.title.clone(),
        author: options.author.clone(),
        description: options.description.clone(),
        include_source: false,
        formats: vec![],
        nutrition: NutritionFacts::default(),
    }
}

fn file(path: &str, media_type: &str, contents: Vec<u8>) -> ExportFile {
    ExportFile {
        path: path.to_owned(),
        media_type: media_type.to_owned(),
        contents,
    }
}

fn zip_files(files: &[ExportFile]) -> Result<Vec<u8>, ApplicationError> {
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for file in files {
        writer
            .start_file(&file.path, options)
            .map_err(|error| ApplicationError::Internal(error.to_string()))?;
        writer
            .write_all(&file.contents)
            .map_err(|error| ApplicationError::Internal(error.to_string()))?;
    }
    writer
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|error| ApplicationError::Internal(error.to_string()))
}

#[cfg(test)]
mod test;
