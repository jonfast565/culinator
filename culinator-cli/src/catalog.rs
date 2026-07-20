use crate::{output::OutputFormat, runtime::Runtime};
use anyhow::{Context, Result, anyhow, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use culinator_core::PercentageView;
use culinator_models::{
    BookExportFormat, BookExportOptions, DoughTempRequest, NewRecipeBook, NutritionFacts,
    PrefermentBuildRequest, RecipeExportFormat, RecipeExportOptions, ScheduleOptions, SearchQuery,
    UnitConvertRequest, UnitFormatRequest, UploadRecipeImageRequest,
};
use std::{env, fs, io::Write, path::Path, process::Command};
use uuid::Uuid;

pub fn list_recipes(runtime: &Runtime, output: OutputFormat) -> Result<()> {
    let items = runtime.state.recipes().list()?;
    output.values(&items, |item| {
        format!(
            "{}\t{}\t{}\t{}",
            item.id,
            item.symbol,
            item.book_id
                .map_or_else(|| "unfiled".into(), |id| id.to_string()),
            item.title
        )
    })
}

pub fn get_recipe(
    runtime: &Runtime,
    selector: &str,
    source_only: bool,
    output: OutputFormat,
) -> Result<()> {
    let summary = runtime.recipe(selector)?;
    let document = runtime.state.recipes().get(summary.id)?;
    if source_only {
        print!("{}", document.source_text);
        Ok(())
    } else {
        output.value(&document)
    }
}

pub fn new_recipe(
    runtime: &Runtime,
    book: Option<&str>,
    title: Option<&str>,
    output: OutputFormat,
) -> Result<()> {
    let book_id = book
        .map(|value| runtime.book(value).map(|item| item.id))
        .transpose()?;
    let mut document = runtime.state.recipes().create(book_id)?;
    if let Some(title) = title {
        let symbol = slug(title);
        let source = format!(
            "culinator 0.3;\n\nrecipe {symbol} {{\n    title \"{}\";\n}}\n",
            escape_string(title)
        );
        document = runtime.state.recipes().save(document.id, &source)?;
    }
    output.message(
        &format!("created {} ({})", document.title, document.id),
        &document,
    )
}

pub fn import_recipe(
    runtime: &Runtime,
    file: &Path,
    book: Option<&str>,
    output: OutputFormat,
) -> Result<()> {
    let source = fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
    let book_id = book
        .map(|value| runtime.book(value).map(|item| item.id))
        .transpose()?;
    let created = runtime.state.recipes().create(book_id)?;
    match runtime.state.recipes().save(created.id, &source) {
        Ok(document) => output.message(
            &format!("imported {} ({})", document.title, document.id),
            &document,
        ),
        Err(error) => {
            let _ = runtime.state.recipes().delete(created.id);
            Err(anyhow!(error.to_string()))
        }
    }
}

pub fn save_recipe(
    runtime: &Runtime,
    selector: &str,
    file: &Path,
    output: OutputFormat,
) -> Result<()> {
    let summary = runtime.recipe(selector)?;
    let source = fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
    let document = runtime.state.recipes().save(summary.id, &source)?;
    output.message(&format!("saved {}", document.title), &document)
}

pub fn edit_recipe(runtime: &Runtime, selector: &str, editor: Option<&str>) -> Result<()> {
    let summary = runtime.recipe(selector)?;
    let document = runtime.state.recipes().get(summary.id)?;
    let mut file = tempfile::Builder::new().suffix(".cg").tempfile()?;
    file.write_all(document.source_text.as_bytes())?;
    file.flush()?;
    let editor = editor
        .map(str::to_owned)
        .or_else(|| env::var("VISUAL").ok())
        .or_else(|| env::var("EDITOR").ok())
        .ok_or_else(|| anyhow!("set $VISUAL or $EDITOR, or pass --editor"))?;
    let status = Command::new(&editor)
        .arg(file.path())
        .status()
        .with_context(|| format!("launch editor {editor}"))?;
    if !status.success() {
        bail!("editor exited with {status}");
    }
    let source = fs::read_to_string(file.path())?;
    let report = runtime.state.recipes().validate_source(&source);
    if !report.valid {
        for diagnostic in report.diagnostics {
            eprintln!("{}: {}", diagnostic.code, diagnostic.message);
        }
        bail!("edited recipe is invalid; no changes were saved");
    }
    let saved = runtime.state.recipes().save(summary.id, &source)?;
    println!("saved {}", saved.title);
    Ok(())
}

pub fn delete_recipe(runtime: &Runtime, selector: &str, confirmed: bool) -> Result<()> {
    if !confirmed {
        bail!("refusing to delete without --yes");
    }
    let summary = runtime.recipe(selector)?;
    runtime.state.recipes().delete(summary.id)?;
    println!("deleted {}", summary.title);
    Ok(())
}

pub fn move_recipe(
    runtime: &Runtime,
    selector: &str,
    book: Option<&str>,
    position: i64,
) -> Result<()> {
    let recipe = runtime.recipe(selector)?;
    let book_id = book
        .map(|value| runtime.book(value).map(|item| item.id))
        .transpose()?;
    runtime
        .state
        .recipes()
        .move_to_book(recipe.id, book_id, position)?;
    println!("moved {}", recipe.title);
    Ok(())
}

pub fn schedule_recipe(
    runtime: &Runtime,
    selector: &str,
    default_duration_seconds: u64,
    output: OutputFormat,
) -> Result<()> {
    let recipe = runtime.recipe(selector)?;
    let document = runtime.state.recipes().get(recipe.id)?;
    let schedule = runtime.state.schedules().schedule_source(
        &document.source_text,
        &ScheduleOptions {
            default_duration_seconds,
        },
    )?;
    match output {
        OutputFormat::Human => {
            for operation in &schedule.operations {
                println!(
                    "{:>6}–{:<6} {:<20} {}",
                    operation.start_seconds,
                    operation.end_seconds,
                    operation.symbol,
                    operation.action
                );
            }
            println!("makespan: {} seconds", schedule.makespan_seconds);
            Ok(())
        }
        _ => output.value(&schedule),
    }
}

pub fn render_recipe(
    runtime: &Runtime,
    selector: &str,
    unit_system: Option<culinator_models::UnitSystem>,
    decimals: bool,
    output: OutputFormat,
) -> Result<()> {
    let summary = runtime.recipe(selector)?;
    let document = runtime.state.recipes().get(summary.id)?;
    let mut recipe = culinator_parser::parse_recipe(&document.source_text)
        .map_err(|error| anyhow!(error.to_string()))?;
    if let Some(system) = unit_system {
        recipe = culinator_narrative::convert_recipe_units(
            &recipe,
            match system {
                culinator_models::UnitSystem::Metric => culinator_core::UnitSystem::Metric,
                culinator_models::UnitSystem::UsCustomary => {
                    culinator_core::UnitSystem::UsCustomary
                }
            },
        );
    }
    let content = culinator_narrative::extract_with(
        &recipe,
        if decimals {
            culinator_narrative::NumberStyle::Decimals
        } else {
            culinator_narrative::NumberStyle::Fractions
        },
    );
    if matches!(output, OutputFormat::Human) {
        println!("{}\n{}\n", summary.title, content.summary);
        if !content.ingredients.is_empty() {
            println!("Ingredients");
            for ingredient in &content.ingredients {
                println!("- {ingredient}");
            }
            println!();
        }
        if !content.equipment.is_empty() {
            println!("Equipment");
            for equipment in &content.equipment {
                println!("- {equipment}");
            }
            println!();
        }
        println!("Method");
        for section in &content.sections {
            if let Some(title) = &section.title {
                println!("\n{title}");
            }
            if let Some(note) = &section.note {
                println!("{note}");
            }
            for step in &section.steps {
                println!("{}. {}", step.number, step.text);
                if let Some(annotation) = step.annotation() {
                    println!("   {annotation}");
                }
            }
        }
        return Ok(());
    }
    let sections = content
        .sections
        .iter()
        .map(|section| {
            serde_json::json!({
                "process": section.process,
                "title": section.title,
                "note": section.note,
                "steps": section.steps.iter().map(|step| serde_json::json!({
                    "symbol": step.symbol,
                    "number": step.number,
                    "text": step.text,
                    "time": step.time,
                    "meta": step.meta,
                    "tools": step.tools,
                })).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    output.value(&serde_json::json!({
        "title": summary.title,
        "summary": content.summary,
        "ingredients": content.ingredients,
        "equipment": content.equipment,
        "sections": sections,
    }))
}

pub fn export_recipe(
    runtime: &Runtime,
    selector: &str,
    destination: &Path,
    options_file: Option<&Path>,
    output: OutputFormat,
) -> Result<()> {
    let recipe = runtime.recipe(selector)?;
    let options = if let Some(path) = options_file {
        read_json(path)?
    } else {
        RecipeExportOptions {
            site_title: None,
            author: None,
            description: None,
            include_source: true,
            formats: vec![
                RecipeExportFormat::Web,
                RecipeExportFormat::Markdown,
                RecipeExportFormat::PlainText,
                RecipeExportFormat::IngredientCsv,
                RecipeExportFormat::Json,
                RecipeExportFormat::PrintHtml,
                RecipeExportFormat::Epub,
            ],
            nutrition: NutritionFacts::default(),
        }
    };
    let bundle = runtime.state.exports().export_recipe(recipe.id, &options)?;
    fs::write(destination, bundle.archive)
        .with_context(|| format!("write {}", destination.display()))?;
    output.message(
        &format!("exported {} to {}", recipe.title, destination.display()),
        &serde_json::json!({"recipeId": recipe.id, "output": destination}),
    )
}

pub fn export_book(
    runtime: &Runtime,
    selector: &str,
    destination: &Path,
    options_file: Option<&Path>,
    output: OutputFormat,
) -> Result<()> {
    let book = runtime.book(selector)?;
    let options = if let Some(path) = options_file {
        read_json(path)?
    } else {
        BookExportOptions {
            formats: vec![
                BookExportFormat::Epub,
                BookExportFormat::PrintHtml,
                BookExportFormat::Web,
            ],
            ..BookExportOptions::default()
        }
    };
    let bundle = runtime.state.exports().export_book(book.id, &options)?;
    fs::write(destination, bundle.archive)
        .with_context(|| format!("write {}", destination.display()))?;
    output.message(
        &format!("exported {} to {}", book.title, destination.display()),
        &serde_json::json!({"bookId": book.id, "output": destination}),
    )
}

pub fn list_books(runtime: &Runtime, output: OutputFormat) -> Result<()> {
    let items = runtime.state.books().list()?;
    output.values(&items, |item| {
        format!(
            "{}\t{}\t{}\t{} recipes",
            item.id, item.symbol, item.title, item.recipe_count
        )
    })
}

pub fn new_book(
    runtime: &Runtime,
    title: &str,
    symbol: Option<String>,
    description: Option<String>,
    output: OutputFormat,
) -> Result<()> {
    let book = runtime.state.books().create(NewRecipeBook {
        title: title.into(),
        symbol,
        description,
    })?;
    output.message(&format!("created {} ({})", book.title, book.id), &book)
}

pub fn rename_book(
    runtime: &Runtime,
    selector: &str,
    title: &str,
    description: Option<String>,
    output: OutputFormat,
) -> Result<()> {
    let book = runtime.book(selector)?;
    let updated = runtime.state.books().update(
        book.id,
        NewRecipeBook {
            title: title.into(),
            symbol: Some(book.symbol),
            description: description.or(book.description),
        },
    )?;
    output.message(&format!("renamed book to {}", updated.title), &updated)
}

pub fn delete_book(runtime: &Runtime, selector: &str, confirmed: bool) -> Result<()> {
    if !confirmed {
        bail!("refusing to delete without --yes");
    }
    let book = runtime.book(selector)?;
    runtime.state.books().delete(book.id)?;
    println!("deleted {}", book.title);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn search(
    runtime: &Runtime,
    text: Option<String>,
    book: Option<&str>,
    allergens: Vec<String>,
    max_active_minutes: Option<f64>,
    hydration: Option<(f64, f64)>,
    limit: usize,
    output: OutputFormat,
) -> Result<()> {
    let book_id = book
        .map(|value| runtime.book(value).map(|item| item.id))
        .transpose()?;
    let hits = runtime.state.search().query(&SearchQuery {
        text,
        book_id,
        exclude_allergens: allergens,
        max_active_minutes,
        hydration: hydration.map(|(min, max)| culinator_models::RangeF64 {
            min: Some(min),
            max: Some(max),
        }),
        limit,
    })?;
    output.values(&hits, |hit| {
        format!("{:.2}\t{}\t{}", hit.score, hit.recipe_id, hit.title)
    })
}

pub fn convert_unit(
    runtime: &Runtime,
    value: f64,
    from: String,
    to: String,
    output: OutputFormat,
) -> Result<()> {
    let result = runtime.state.units().convert(&UnitConvertRequest {
        value,
        from_unit: from,
        to_unit: to,
    })?;
    match output {
        OutputFormat::Human => {
            println!("{} {}", result.value, result.unit);
            Ok(())
        }
        _ => output.value(&result),
    }
}

pub fn format_unit(
    runtime: &Runtime,
    request: UnitFormatRequest,
    output: OutputFormat,
) -> Result<()> {
    let result = runtime.state.units().format(&request)?;
    match output {
        OutputFormat::Human => {
            println!("{}", result.formatted);
            Ok(())
        }
        _ => output.value(&result),
    }
}

pub fn list_formulas(runtime: &Runtime, recipe: &str, output: OutputFormat) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    let formulas = runtime.state.formulas().list_for_recipe(recipe.id)?;
    output.values(&formulas, |formula| {
        format!("{}\t{}", formula.id, formula.name)
    })
}

pub fn get_formula(runtime: &Runtime, id: Uuid, output: OutputFormat) -> Result<()> {
    output.value(&runtime.state.formulas().get(id)?)
}

pub fn save_formula(runtime: &Runtime, input: &Path, output: OutputFormat) -> Result<()> {
    let content = fs::read_to_string(input).with_context(|| format!("read {}", input.display()))?;
    let formula: culinator_core::Formula =
        serde_json::from_str(&content).with_context(|| format!("parse {}", input.display()))?;
    runtime.state.formulas().save(&formula)?;
    output.message(
        &format!("saved formula {}", formula.name),
        &serde_json::json!({"id": formula.id, "name": formula.name}),
    )
}

pub fn calculate_formula(
    runtime: &Runtime,
    id: Uuid,
    target_mass_grams: f64,
    output: OutputFormat,
) -> Result<()> {
    let result = runtime
        .state
        .formulas()
        .calculate_and_record(id, target_mass_grams)?;
    output.value(&result)
}

pub fn formula_percentages(
    runtime: &Runtime,
    id: Uuid,
    total: bool,
    output: OutputFormat,
) -> Result<()> {
    let formula = runtime.state.formulas().get(id)?;
    let result = runtime.state.formulas().percentages(
        &formula,
        if total {
            PercentageView::Total
        } else {
            PercentageView::Reference
        },
    )?;
    output.value(&result)
}

pub fn build_preferment(
    runtime: &Runtime,
    request: PrefermentBuildRequest,
    output: OutputFormat,
) -> Result<()> {
    output.value(&runtime.state.formulas().build_preferment(request)?)
}

pub fn dough_temp(
    runtime: &Runtime,
    request: DoughTempRequest,
    output: OutputFormat,
) -> Result<()> {
    output.value(&runtime.state.formulas().dough_temp(request)?)
}

pub fn list_images(runtime: &Runtime, recipe: &str, output: OutputFormat) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    let images = runtime.state.list_recipe_images(recipe.id)?;
    output.values(&images, |image| {
        format!(
            "{}\t{}\t{}\t{} bytes",
            image.handle, image.role, image.media_type, image.byte_size
        )
    })
}

#[allow(clippy::too_many_arguments)]
pub fn add_image(
    runtime: &Runtime,
    recipe: &str,
    file: &Path,
    handle: Option<String>,
    role: String,
    operation_symbol: Option<String>,
    media_type: Option<String>,
    output: OutputFormat,
) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    let bytes = fs::read(file).with_context(|| format!("read {}", file.display()))?;
    let asset = runtime.state.upload_recipe_image(
        recipe.id,
        UploadRecipeImageRequest {
            handle,
            role,
            operation_symbol,
            media_type: media_type.unwrap_or_else(|| infer_media_type(file).into()),
            file_name: file
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned),
            data_base64: STANDARD.encode(bytes),
        },
    )?;
    output.message(&format!("uploaded {}", asset.handle), &asset)
}

pub fn get_image(runtime: &Runtime, recipe: &str, handle: &str, output: &Path) -> Result<()> {
    let recipe = runtime.recipe(recipe)?;
    let data = runtime
        .state
        .get_recipe_image(recipe.id, handle)?
        .ok_or_else(|| anyhow!("image not found: {handle}"))?;
    let bytes = STANDARD
        .decode(data.data_base64)
        .context("decode stored image")?;
    fs::write(output, bytes).with_context(|| format!("write {}", output.display()))?;
    println!("wrote {}", output.display());
    Ok(())
}

pub fn delete_image(runtime: &Runtime, recipe: &str, handle: &str, confirmed: bool) -> Result<()> {
    if !confirmed {
        bail!("refusing to delete without --yes");
    }
    let recipe = runtime.recipe(recipe)?;
    if !runtime.state.delete_recipe_image(recipe.id, handle)? {
        bail!("image not found: {handle}");
    }
    println!("deleted {handle}");
    Ok(())
}

pub fn initialize(runtime: &Runtime, status_only: bool, output: OutputFormat) -> Result<()> {
    let report = if status_only {
        runtime.state.init_status()?
    } else {
        runtime.state.initialize()?
    };
    output.value(&report)
}

fn slug(value: &str) -> String {
    let mut result = value
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    while result.contains("__") {
        result = result.replace("__", "_");
    }
    result.trim_matches('_').to_owned()
}

fn escape_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn infer_media_type(path: &Path) -> &'static str {
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

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("parse {}", path.display()))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn title_slug_is_a_dsl_symbol() {
        assert_eq!(slug("  Brown-Butter Cookies! "), "brown_butter_cookies");
    }

    #[test]
    fn media_type_uses_extension() {
        assert_eq!(infer_media_type(Path::new("cover.png")), "image/png");
    }

    #[test]
    fn catalog_import_rolls_back_invalid_recipe() {
        let directory = tempfile::tempdir().expect("temp dir");
        let runtime = Runtime::open(&directory.path().join("catalog.sqlite3")).expect("runtime");
        let source = directory.path().join("invalid.cg");
        fs::write(&source, "not a recipe").expect("write source");

        assert!(
            import_recipe(&runtime, &source, None, OutputFormat::Human).is_err(),
            "invalid DSL must be rejected by RecipeService"
        );
        assert!(
            runtime.state.recipes().list().expect("list").is_empty(),
            "failed import must not leave an empty recipe behind"
        );
    }
}
