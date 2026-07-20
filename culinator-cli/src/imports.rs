use crate::{output::OutputFormat, runtime::Runtime};
use anyhow::{Context, Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD};
use culinator_models::{
    ImportSettings, RecipeImage, RecipeImportRequest, StructuredInput, StructuredInputFormat,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn structured(
    runtime: &Runtime,
    file: &PathBuf,
    format: StructuredInputFormat,
    source_output: Option<&PathBuf>,
    save: bool,
    book: Option<&str>,
    output: OutputFormat,
) -> Result<()> {
    let content = fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
    let draft = runtime
        .state
        .imports()
        .import_structured(StructuredInput { format, content })?;
    if let Some(path) = source_output {
        fs::write(path, &draft.source_text).with_context(|| format!("write {}", path.display()))?;
    }
    let saved_id = if save {
        Some(save_source(runtime, &draft.source_text, book)?)
    } else {
        None
    };
    match output {
        OutputFormat::Human if source_output.is_none() && !save => {
            print!("{}", draft.source_text);
            for warning in draft.warnings {
                eprintln!("warning: {warning}");
            }
            Ok(())
        }
        OutputFormat::Human => {
            println!(
                "{}{}",
                draft.title,
                saved_id.map_or_else(String::new, |id| format!(" ({id})"))
            );
            Ok(())
        }
        _ => output.value(&serde_json::json!({
            "draft": draft,
            "savedRecipeId": saved_id,
            "sourceOutput": source_output,
        })),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn scan(
    runtime: &Runtime,
    files: Vec<PathBuf>,
    target_language: Option<String>,
    source_output: Option<&PathBuf>,
    save: bool,
    book: Option<&str>,
    output: OutputFormat,
) -> Result<()> {
    let images = files
        .into_iter()
        .map(|path| {
            let data = fs::read(&path).with_context(|| format!("read {}", path.display()))?;
            Ok(RecipeImage {
                file_name: path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("recipe-image")
                    .to_owned(),
                media_type: media_type(&path).into(),
                data_base64: STANDARD.encode(data),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let tokio = tokio::runtime::Runtime::new().context("create async import runtime")?;
    let result = tokio.block_on(runtime.state.imports().translate(RecipeImportRequest {
        images,
        target_language,
        recipe_book_title: None,
    }))?;
    if let Some(path) = source_output {
        fs::write(path, &result.source_text)
            .with_context(|| format!("write {}", path.display()))?;
    }
    let saved_id = if save {
        Some(save_source(runtime, &result.source_text, book)?)
    } else {
        None
    };
    match output {
        OutputFormat::Human if source_output.is_none() && !save => {
            print!("{}", result.source_text);
            for warning in result.warnings {
                eprintln!("warning: {warning}");
            }
            Ok(())
        }
        OutputFormat::Human => {
            println!(
                "{}{}",
                result.title,
                saved_id.map_or_else(String::new, |id| format!(" ({id})"))
            );
            Ok(())
        }
        _ => output.value(&serde_json::json!({
            "result": result,
            "savedRecipeId": saved_id,
            "sourceOutput": source_output,
        })),
    }
}

pub fn settings(runtime: &Runtime, output: OutputFormat) -> Result<()> {
    output.value(&runtime.state.imports().public_settings()?)
}

#[allow(clippy::too_many_arguments)]
pub fn configure(
    runtime: &Runtime,
    api_key: Option<String>,
    model: Option<String>,
    local_ocr: Option<bool>,
    tesseract_command: Option<String>,
    output: OutputFormat,
) -> Result<()> {
    let mut settings: ImportSettings = runtime.state.imports().settings()?;
    if let Some(value) = api_key {
        settings.openai_api_key = value;
    }
    if let Some(value) = model {
        settings.openai_model = value;
    }
    if let Some(value) = local_ocr {
        settings.use_local_ocr = value;
    }
    if let Some(value) = tesseract_command {
        settings.tesseract_command = value;
    }
    runtime.state.imports().save_settings(&settings)?;
    output.value(&runtime.state.imports().public_settings()?)
}

fn save_source(runtime: &Runtime, source: &str, book: Option<&str>) -> Result<uuid::Uuid> {
    let book_id = book
        .map(|selector| runtime.book(selector).map(|item| item.id))
        .transpose()?;
    let created = runtime.state.recipes().create(book_id)?;
    match runtime.state.recipes().save(created.id, source) {
        Ok(document) => Ok(document.id),
        Err(error) => {
            let _ = runtime.state.recipes().delete(created.id);
            Err(anyhow!(error.to_string()))
        }
    }
}

fn media_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => "image/png",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "image/jpeg",
    }
}
