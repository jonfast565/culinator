mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use culinator_models::StructuredInputFormat;
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
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Check {
        file: PathBuf,
    },
    Parse {
        file: PathBuf,
    },
    InitDb {
        database: PathBuf,
    },
    Import {
        file: PathBuf,
        database: PathBuf,
    },
    ImportBook {
        file: PathBuf,
        database: PathBuf,
    },
    ListBooks {
        database: PathBuf,
    },
    CreateBook {
        database: PathBuf,
        title: String,
    },
    Export {
        file: PathBuf,
        output: PathBuf,
    },
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
    Search {
        database: PathBuf,
        query: String,
        #[arg(long)]
        book: Option<uuid::Uuid>,
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Check { file } => commands::check_recipe(&file),
        Command::Parse { file } => commands::parse_recipe(&file),
        Command::InitDb { database } => commands::init_database(&database),
        Command::Import { file, database } => commands::import_recipe(&file, &database),
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
        Command::Search {
            database,
            query,
            book,
            limit,
        } => commands::search_recipes(&database, &query, book, limit),
    }
}
#[cfg(test)]
mod test;
