mod catalog;
mod commands;
mod imports;
mod output;
mod runtime;
mod workflows;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use culinator_models::{
    DoughTempRequest, Locale, PrefermentBuildRequest, StructuredInputFormat, UnitFormatRequest,
    UnitSystem,
};
use output::OutputFormat;
use runtime::Runtime;
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, ValueEnum)]
enum CliStructuredFormat {
    JsonLd,
    Json,
    Yaml,
}

impl From<CliStructuredFormat> for StructuredInputFormat {
    fn from(value: CliStructuredFormat) -> Self {
        match value {
            CliStructuredFormat::JsonLd => StructuredInputFormat::JsonLd,
            CliStructuredFormat::Json => StructuredInputFormat::Json,
            CliStructuredFormat::Yaml => StructuredInputFormat::Yaml,
        }
    }
}

#[derive(Parser)]
#[command(
    name = "culinator",
    version,
    about = "Typed food-production DSL toolchain"
)]
struct Cli {
    /// Catalog database used by domain commands.
    #[arg(long, global = true, default_value = "culinator-dev.sqlite3")]
    database: PathBuf,
    /// Human-readable, JSON array/object, or newline-delimited JSON output.
    #[arg(long, global = true, value_enum, default_value_t)]
    format: OutputFormat,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate a local recipe document.
    Check { file: PathBuf },
    /// Parse a local recipe document as JSON.
    Parse { file: PathBuf },
    /// Manage recipes in the catalog.
    Recipe {
        #[command(subcommand)]
        command: RecipeCommand,
    },
    /// Manage recipe books.
    Book {
        #[command(subcommand)]
        command: BookCommand,
    },
    /// Search the catalog with the same filters as the desktop.
    Search {
        query: Option<String>,
        #[arg(long)]
        book: Option<String>,
        #[arg(long = "exclude-allergen")]
        exclude_allergens: Vec<String>,
        #[arg(long)]
        max_active_minutes: Option<f64>,
        #[arg(long, requires = "hydration_max")]
        hydration_min: Option<f64>,
        #[arg(long, requires = "hydration_min")]
        hydration_max: Option<f64>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    /// Convert and format measurements.
    Unit {
        #[command(subcommand)]
        command: UnitCommand,
    },
    /// Inspect and calculate stored formulas.
    Formula {
        #[command(subcommand)]
        command: FormulaCommand,
    },
    /// Manage persisted recipe image assets.
    Image {
        #[command(subcommand)]
        command: ImageCommand,
    },
    /// Link ingredients to nutrition data and calculate labels.
    Nutrition {
        #[command(subcommand)]
        command: NutritionCommand,
    },
    /// Manage food-safety plans and monitoring records.
    Haccp {
        #[command(subcommand)]
        command: HaccpCommand,
    },
    /// Record kitchen execution sessions.
    Cook {
        #[command(subcommand)]
        command: CookCommand,
    },
    /// Initialize seeds and nutrition, or inspect initialization status.
    Init {
        #[arg(long)]
        status: bool,
    },
    /// Import local recipes, structured data, or scanned images.
    Import {
        #[command(subcommand)]
        command: ImportCommand,
    },
    // Legacy flat commands retained for script compatibility.
    #[command(hide = true)]
    InitDb { database: PathBuf },
    #[command(name = "import-legacy", hide = true)]
    LegacyImport { file: PathBuf, database: PathBuf },
    #[command(hide = true)]
    ImportBook { file: PathBuf, database: PathBuf },
    #[command(hide = true)]
    ListBooks { database: PathBuf },
    #[command(hide = true)]
    CreateBook { database: PathBuf, title: String },
    #[command(hide = true)]
    Export { file: PathBuf, output: PathBuf },
    #[command(hide = true)]
    ExportBook {
        database: PathBuf,
        #[arg(long)]
        book: uuid::Uuid,
        output: PathBuf,
    },
    ImportStructured {
        file: PathBuf,
        #[arg(long, value_enum, default_value_t = CliStructuredFormat::JsonLd)]
        format: CliStructuredFormat,
        #[arg(long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum RecipeCommand {
    List,
    Get {
        recipe: String,
        #[arg(long)]
        source: bool,
    },
    New {
        #[arg(long)]
        book: Option<String>,
        #[arg(long)]
        title: Option<String>,
    },
    Import {
        file: PathBuf,
        #[arg(long)]
        book: Option<String>,
    },
    Save {
        recipe: String,
        file: PathBuf,
    },
    Edit {
        recipe: String,
        #[arg(long)]
        editor: Option<String>,
    },
    Delete {
        recipe: String,
        #[arg(long)]
        yes: bool,
    },
    Move {
        recipe: String,
        /// Destination book. Omit to move to Unfiled.
        #[arg(long)]
        book: Option<String>,
        #[arg(long, default_value_t = 0)]
        position: i64,
    },
    Validate {
        file: PathBuf,
    },
    Schedule {
        recipe: String,
        #[arg(long, default_value_t = 300)]
        default_duration_seconds: u64,
    },
    Render {
        recipe: String,
        #[arg(long, value_enum)]
        system: Option<CliUnitSystem>,
        #[arg(long)]
        decimals: bool,
    },
    Export {
        recipe: String,
        output: PathBuf,
        /// RecipeExportOptions JSON document.
        #[arg(long)]
        options: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ImportCommand {
    Recipe {
        file: PathBuf,
        #[arg(long)]
        book: Option<String>,
    },
    Structured {
        file: PathBuf,
        #[arg(long, value_enum, default_value_t = CliStructuredFormat::JsonLd)]
        format: CliStructuredFormat,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        save: bool,
        #[arg(long, requires = "save")]
        book: Option<String>,
    },
    Scan {
        #[arg(required = true)]
        images: Vec<PathBuf>,
        #[arg(long)]
        target_language: Option<String>,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        save: bool,
        #[arg(long, requires = "save")]
        book: Option<String>,
    },
    Settings,
    Configure {
        #[arg(long)]
        api_key: Option<String>,
        #[arg(long)]
        model: Option<String>,
        #[arg(long)]
        local_ocr: Option<bool>,
        #[arg(long)]
        tesseract_command: Option<String>,
    },
}

#[derive(Subcommand)]
enum BookCommand {
    List,
    New {
        title: String,
        #[arg(long)]
        symbol: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    Rename {
        book: String,
        title: String,
        #[arg(long)]
        description: Option<String>,
    },
    Delete {
        book: String,
        #[arg(long)]
        yes: bool,
    },
    Export {
        book: String,
        output: PathBuf,
        /// BookExportOptions JSON document.
        #[arg(long)]
        options: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum UnitCommand {
    Convert {
        value: f64,
        from: String,
        to: String,
    },
    Format {
        value: f64,
        unit: String,
        #[arg(long, value_enum, default_value_t)]
        system: CliUnitSystem,
        #[arg(long, value_enum, default_value_t)]
        locale: CliLocale,
    },
}

#[derive(Copy, Clone, Debug, Default, ValueEnum)]
enum CliUnitSystem {
    #[default]
    Metric,
    UsCustomary,
}

impl From<CliUnitSystem> for UnitSystem {
    fn from(value: CliUnitSystem) -> Self {
        match value {
            CliUnitSystem::Metric => Self::Metric,
            CliUnitSystem::UsCustomary => Self::UsCustomary,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, ValueEnum)]
enum CliLocale {
    #[default]
    EnUs,
    EnGb,
}

impl From<CliLocale> for Locale {
    fn from(value: CliLocale) -> Self {
        match value {
            CliLocale::EnUs => Self::EnUs,
            CliLocale::EnGb => Self::EnGb,
        }
    }
}

#[derive(Subcommand)]
enum FormulaCommand {
    List {
        recipe: String,
    },
    Get {
        formula: uuid::Uuid,
    },
    /// Save a Formula JSON document.
    Save {
        input: PathBuf,
    },
    Calculate {
        formula: uuid::Uuid,
        target_mass_grams: f64,
    },
    Percentages {
        formula: uuid::Uuid,
        #[arg(long)]
        total: bool,
    },
    Preferment {
        kind: String,
        flour_pct: f64,
        hydration: f64,
        #[arg(long, default_value_t = 0.0)]
        inoculation: f64,
        #[arg(long, default_value = "preferment")]
        stage: String,
    },
    DoughTemp {
        desired_dough_temp: f64,
        friction_factor: f64,
        flour_temp: f64,
        room_temp: f64,
        #[arg(long)]
        preferment_temp: Option<f64>,
    },
}

#[derive(Subcommand)]
enum ImageCommand {
    List {
        recipe: String,
    },
    Add {
        recipe: String,
        file: PathBuf,
        #[arg(long)]
        handle: Option<String>,
        #[arg(long, default_value = "cover")]
        role: String,
        #[arg(long)]
        operation: Option<String>,
        #[arg(long)]
        media_type: Option<String>,
    },
    Get {
        recipe: String,
        handle: String,
        output: PathBuf,
    },
    Delete {
        recipe: String,
        handle: String,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum NutritionCommand {
    Status,
    Search {
        query: String,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    State {
        recipe: String,
    },
    Links {
        recipe: String,
    },
    Link {
        recipe: String,
        resource: String,
        fdc_id: i64,
    },
    Unlink {
        recipe: String,
        resource: String,
    },
    AutoLink {
        recipe: String,
        #[arg(long, default_value_t = 0.45)]
        min_score: f64,
        #[arg(long)]
        dry_run: bool,
    },
    Calculate {
        recipe: String,
        #[arg(long)]
        servings: Option<f64>,
        #[arg(long)]
        serving_size: Option<String>,
        #[arg(long)]
        serving_size_grams: Option<f64>,
    },
    /// Save recipe-level nutrition from a SaveRecipeNutritionRequest JSON document.
    Save {
        recipe: String,
        input: PathBuf,
    },
    /// Save an ingredient override from a SaveIngredientManualNutritionRequest JSON document.
    SaveManual {
        recipe: String,
        input: PathBuf,
    },
    DeleteManual {
        recipe: String,
        resource: String,
    },
}

#[derive(Subcommand)]
enum HaccpCommand {
    List {
        recipe: String,
    },
    Get {
        plan: uuid::Uuid,
    },
    New {
        recipe: String,
        title: String,
        #[arg(long)]
        description: Option<String>,
    },
    /// Replace a plan from a SaveHaccpPlanRequest JSON document.
    Save {
        plan: uuid::Uuid,
        input: PathBuf,
    },
    /// Add a monitoring record from a NewHaccpMonitoringRecord JSON document.
    Record {
        ccp: uuid::Uuid,
        input: PathBuf,
    },
    Delete {
        plan: uuid::Uuid,
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum CookCommand {
    List {
        recipe: String,
    },
    Get {
        attempt: uuid::Uuid,
    },
    Start {
        recipe: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long)]
        scale_factor: Option<f64>,
    },
    /// Update a try from an UpdateRecipeTry JSON document.
    Update {
        attempt: uuid::Uuid,
        input: PathBuf,
    },
    /// Update an operation from an UpdateTryOperation JSON document.
    UpdateOperation {
        attempt: uuid::Uuid,
        operation: uuid::Uuid,
        input: PathBuf,
    },
    /// Add an observation from a NewTryObservation JSON document.
    Observe {
        attempt: uuid::Uuid,
        input: PathBuf,
    },
    Delete {
        attempt: uuid::Uuid,
        #[arg(long)]
        yes: bool,
    },
}

fn main() -> Result<()> {
    run(Cli::parse())
}

fn run(cli: Cli) -> Result<()> {
    let output = cli.format;
    match cli.command {
        Command::Check { file } => commands::check_recipe(&file),
        Command::Parse { file } => commands::parse_recipe(&file),
        Command::Recipe { command } => {
            let runtime = Runtime::open(&cli.database)?;
            match command {
                RecipeCommand::List => catalog::list_recipes(&runtime, output),
                RecipeCommand::Get { recipe, source } => {
                    catalog::get_recipe(&runtime, &recipe, source, output)
                }
                RecipeCommand::New { book, title } => {
                    catalog::new_recipe(&runtime, book.as_deref(), title.as_deref(), output)
                }
                RecipeCommand::Import { file, book } => {
                    catalog::import_recipe(&runtime, &file, book.as_deref(), output)
                }
                RecipeCommand::Save { recipe, file } => {
                    catalog::save_recipe(&runtime, &recipe, &file, output)
                }
                RecipeCommand::Edit { recipe, editor } => {
                    catalog::edit_recipe(&runtime, &recipe, editor.as_deref())
                }
                RecipeCommand::Delete { recipe, yes } => {
                    catalog::delete_recipe(&runtime, &recipe, yes)
                }
                RecipeCommand::Move {
                    recipe,
                    book,
                    position,
                } => catalog::move_recipe(&runtime, &recipe, book.as_deref(), position),
                RecipeCommand::Validate { file } => commands::check_recipe(&file),
                RecipeCommand::Schedule {
                    recipe,
                    default_duration_seconds,
                } => catalog::schedule_recipe(&runtime, &recipe, default_duration_seconds, output),
                RecipeCommand::Render {
                    recipe,
                    system,
                    decimals,
                } => catalog::render_recipe(
                    &runtime,
                    &recipe,
                    system.map(Into::into),
                    decimals,
                    output,
                ),
                RecipeCommand::Export {
                    recipe,
                    output: path,
                    options,
                } => catalog::export_recipe(&runtime, &recipe, &path, options.as_deref(), output),
            }
        }
        Command::Book { command } => {
            let runtime = Runtime::open(&cli.database)?;
            match command {
                BookCommand::List => catalog::list_books(&runtime, output),
                BookCommand::New {
                    title,
                    symbol,
                    description,
                } => catalog::new_book(&runtime, &title, symbol, description, output),
                BookCommand::Rename {
                    book,
                    title,
                    description,
                } => catalog::rename_book(&runtime, &book, &title, description, output),
                BookCommand::Delete { book, yes } => catalog::delete_book(&runtime, &book, yes),
                BookCommand::Export {
                    book,
                    output: path,
                    options,
                } => catalog::export_book(&runtime, &book, &path, options.as_deref(), output),
            }
        }
        Command::Search {
            query,
            book,
            exclude_allergens,
            max_active_minutes,
            hydration_min,
            hydration_max,
            limit,
        } => {
            let runtime = Runtime::open(&cli.database)?;
            catalog::search(
                &runtime,
                query,
                book.as_deref(),
                exclude_allergens,
                max_active_minutes,
                hydration_min.zip(hydration_max),
                limit,
                output,
            )
        }
        Command::Unit { command } => {
            let runtime = Runtime::open(&cli.database)?;
            match command {
                UnitCommand::Convert { value, from, to } => {
                    catalog::convert_unit(&runtime, value, from, to, output)
                }
                UnitCommand::Format {
                    value,
                    unit,
                    system,
                    locale,
                } => catalog::format_unit(
                    &runtime,
                    UnitFormatRequest {
                        value,
                        unit,
                        unit_system: system.into(),
                        locale: locale.into(),
                    },
                    output,
                ),
            }
        }
        Command::Formula { command } => {
            let runtime = Runtime::open(&cli.database)?;
            match command {
                FormulaCommand::List { recipe } => {
                    catalog::list_formulas(&runtime, &recipe, output)
                }
                FormulaCommand::Get { formula } => catalog::get_formula(&runtime, formula, output),
                FormulaCommand::Save { input } => catalog::save_formula(&runtime, &input, output),
                FormulaCommand::Calculate {
                    formula,
                    target_mass_grams,
                } => catalog::calculate_formula(&runtime, formula, target_mass_grams, output),
                FormulaCommand::Percentages { formula, total } => {
                    catalog::formula_percentages(&runtime, formula, total, output)
                }
                FormulaCommand::Preferment {
                    kind,
                    flour_pct,
                    hydration,
                    inoculation,
                    stage,
                } => catalog::build_preferment(
                    &runtime,
                    PrefermentBuildRequest {
                        kind,
                        flour_pct,
                        hydration,
                        inoculation,
                        stage,
                    },
                    output,
                ),
                FormulaCommand::DoughTemp {
                    desired_dough_temp,
                    friction_factor,
                    flour_temp,
                    room_temp,
                    preferment_temp,
                } => catalog::dough_temp(
                    &runtime,
                    DoughTempRequest {
                        desired_dough_temp,
                        friction_factor,
                        flour_temp,
                        room_temp,
                        preferment_temp,
                    },
                    output,
                ),
            }
        }
        Command::Image { command } => {
            let runtime = Runtime::open(&cli.database)?;
            match command {
                ImageCommand::List { recipe } => catalog::list_images(&runtime, &recipe, output),
                ImageCommand::Add {
                    recipe,
                    file,
                    handle,
                    role,
                    operation,
                    media_type,
                } => catalog::add_image(
                    &runtime, &recipe, &file, handle, role, operation, media_type, output,
                ),
                ImageCommand::Get {
                    recipe,
                    handle,
                    output,
                } => catalog::get_image(&runtime, &recipe, &handle, &output),
                ImageCommand::Delete {
                    recipe,
                    handle,
                    yes,
                } => catalog::delete_image(&runtime, &recipe, &handle, yes),
            }
        }
        Command::Nutrition { command } => {
            let runtime = Runtime::open(&cli.database)?;
            match command {
                NutritionCommand::Status => workflows::nutrition_status(&runtime, output),
                NutritionCommand::Search { query, limit } => {
                    workflows::nutrition_search(&runtime, &query, limit, output)
                }
                NutritionCommand::State { recipe } => {
                    workflows::nutrition_state(&runtime, &recipe, output)
                }
                NutritionCommand::Links { recipe } => {
                    workflows::nutrition_links(&runtime, &recipe, output)
                }
                NutritionCommand::Link {
                    recipe,
                    resource,
                    fdc_id,
                } => workflows::nutrition_link(&runtime, &recipe, resource, fdc_id, output),
                NutritionCommand::Unlink { recipe, resource } => {
                    workflows::nutrition_unlink(&runtime, &recipe, &resource)
                }
                NutritionCommand::AutoLink {
                    recipe,
                    min_score,
                    dry_run,
                } => workflows::nutrition_auto_link(&runtime, &recipe, min_score, dry_run, output),
                NutritionCommand::Calculate {
                    recipe,
                    servings,
                    serving_size,
                    serving_size_grams,
                } => workflows::nutrition_calculate(
                    &runtime,
                    &recipe,
                    servings,
                    serving_size,
                    serving_size_grams,
                    output,
                ),
                NutritionCommand::Save { recipe, input } => {
                    workflows::nutrition_save(&runtime, &recipe, &input, output)
                }
                NutritionCommand::SaveManual { recipe, input } => {
                    workflows::nutrition_save_manual(&runtime, &recipe, &input, output)
                }
                NutritionCommand::DeleteManual { recipe, resource } => {
                    workflows::nutrition_delete_manual(&runtime, &recipe, &resource)
                }
            }
        }
        Command::Haccp { command } => {
            let runtime = Runtime::open(&cli.database)?;
            match command {
                HaccpCommand::List { recipe } => workflows::haccp_list(&runtime, &recipe, output),
                HaccpCommand::Get { plan } => workflows::haccp_get(&runtime, plan, output),
                HaccpCommand::New {
                    recipe,
                    title,
                    description,
                } => workflows::haccp_create(&runtime, &recipe, title, description, output),
                HaccpCommand::Save { plan, input } => {
                    workflows::haccp_save(&runtime, plan, &input, output)
                }
                HaccpCommand::Record { ccp, input } => {
                    workflows::haccp_record(&runtime, ccp, &input, output)
                }
                HaccpCommand::Delete { plan, yes } => workflows::haccp_delete(&runtime, plan, yes),
            }
        }
        Command::Cook { command } => {
            let runtime = Runtime::open(&cli.database)?;
            match command {
                CookCommand::List { recipe } => workflows::cook_list(&runtime, &recipe, output),
                CookCommand::Get { attempt } => workflows::cook_get(&runtime, attempt, output),
                CookCommand::Start {
                    recipe,
                    title,
                    notes,
                    scale_factor,
                } => workflows::cook_start(&runtime, &recipe, title, notes, scale_factor, output),
                CookCommand::Update { attempt, input } => {
                    workflows::cook_update(&runtime, attempt, &input, output)
                }
                CookCommand::UpdateOperation {
                    attempt,
                    operation,
                    input,
                } => workflows::cook_update_operation(&runtime, attempt, operation, &input, output),
                CookCommand::Observe { attempt, input } => {
                    workflows::cook_observe(&runtime, attempt, &input, output)
                }
                CookCommand::Delete { attempt, yes } => {
                    workflows::cook_delete(&runtime, attempt, yes)
                }
            }
        }
        Command::Init { status } => {
            let runtime = Runtime::open(&cli.database)?;
            catalog::initialize(&runtime, status, output)
        }
        Command::Import { command } => {
            let runtime = Runtime::open(&cli.database)?;
            match command {
                ImportCommand::Recipe { file, book } => {
                    catalog::import_recipe(&runtime, &file, book.as_deref(), output)
                }
                ImportCommand::Structured {
                    file,
                    format,
                    output: source_output,
                    save,
                    book,
                } => imports::structured(
                    &runtime,
                    &file,
                    format.into(),
                    source_output.as_ref(),
                    save,
                    book.as_deref(),
                    output,
                ),
                ImportCommand::Scan {
                    images,
                    target_language,
                    output: source_output,
                    save,
                    book,
                } => imports::scan(
                    &runtime,
                    images,
                    target_language,
                    source_output.as_ref(),
                    save,
                    book.as_deref(),
                    output,
                ),
                ImportCommand::Settings => imports::settings(&runtime, output),
                ImportCommand::Configure {
                    api_key,
                    model,
                    local_ocr,
                    tesseract_command,
                } => imports::configure(
                    &runtime,
                    api_key,
                    model,
                    local_ocr,
                    tesseract_command,
                    output,
                ),
            }
        }
        Command::InitDb { database } => commands::init_database(&database),
        Command::LegacyImport { file, database } => commands::import_recipe(&file, &database),
        Command::ImportBook { file, database } => commands::import_recipe_book(&file, &database),
        Command::ListBooks { database } => commands::list_books(&database),
        Command::CreateBook { database, title } => commands::create_book(&database, &title),
        Command::Export { file, output } => commands::export_recipe(&file, &output),
        Command::ExportBook {
            database,
            book,
            output,
        } => commands::export_book(&database, book, &output),
        Command::ImportStructured {
            file,
            format,
            output,
        } => commands::import_structured(&file, format.into(), output.as_deref()),
    }
}
#[cfg(test)]
mod test;
