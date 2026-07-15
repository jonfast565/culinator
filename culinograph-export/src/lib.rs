mod content;
mod csv;
mod epub;
mod html;
mod label;
mod manifest;
mod markdown;
mod plain_text;

use culinograph_core::Recipe;
use culinograph_models::{
    ApplicationError, ExportFile, RecipeExportBundle, RecipeExportFormat, RecipeExportOptions,
    RecipeExporter,
};
use std::io::{Cursor, Write};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

#[derive(Debug, Default, Clone, Copy)]
pub struct StaticRecipeExporter;

impl RecipeExporter for StaticRecipeExporter {
    fn export(
        &self,
        recipe: &Recipe,
        source_text: &str,
        options: &RecipeExportOptions,
    ) -> Result<RecipeExportBundle, ApplicationError> {
        let formats = if options.formats.is_empty() {
            vec![RecipeExportFormat::Web, RecipeExportFormat::Json]
        } else {
            options.formats.clone()
        };
        let label_svg = label::render(&options.nutrition);
        let mut files = Vec::new();

        for format in formats {
            match format {
                RecipeExportFormat::Web => {
                    files.push(file(
                        "index.html",
                        "text/html; charset=utf-8",
                        html::render(recipe, options, &label_svg)?.into_bytes(),
                    ));
                    files.push(file(
                        "nutrition-facts.svg",
                        "image/svg+xml",
                        label_svg.clone().into_bytes(),
                    ));
                }
                RecipeExportFormat::Markdown => files.push(file(
                    "recipe.md",
                    "text/markdown; charset=utf-8",
                    markdown::render(recipe, options).into_bytes(),
                )),
                RecipeExportFormat::PlainText => files.push(file(
                    "recipe.txt",
                    "text/plain; charset=utf-8",
                    plain_text::render(recipe, options).into_bytes(),
                )),
                RecipeExportFormat::IngredientCsv => files.push(file(
                    "ingredients.csv",
                    "text/csv; charset=utf-8",
                    csv::render(recipe).into_bytes(),
                )),
                RecipeExportFormat::Json => {
                    let recipe_json = serde_json::to_vec_pretty(recipe)
                        .map_err(|error| ApplicationError::Internal(error.to_string()))?;
                    files.push(file("recipe.json", "application/json", recipe_json));
                }
                RecipeExportFormat::PrintHtml => files.push(file(
                    "print.html",
                    "text/html; charset=utf-8",
                    html::render(recipe, options, &label_svg)?.into_bytes(),
                )),
                RecipeExportFormat::Epub => files.push(file(
                    "recipe.epub",
                    "application/epub+zip",
                    epub::render(recipe, options)?,
                )),
            }
        }

        if options.include_source {
            files.push(file(
                "recipe.cg",
                "text/plain; charset=utf-8",
                source_text.as_bytes().to_vec(),
            ));
        }
        let manifest = manifest::render(recipe, options, &files);
        files.push(file(
            "manifest.json",
            "application/json",
            manifest.into_bytes(),
        ));
        let archive = zip_files(&files)?;
        Ok(RecipeExportBundle {
            file_name: format!("{}.zip", slug(&recipe.title)),
            files,
            archive,
        })
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
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;
        writer
            .write_all(&file.contents)
            .map_err(|e| ApplicationError::Internal(e.to_string()))?;
    }
    writer
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(|e| ApplicationError::Internal(e.to_string()))
}

fn slug(value: &str) -> String {
    let mut out = String::new();
    for c in value.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

#[cfg(test)]
mod test;
