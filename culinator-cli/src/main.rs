mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    Check { file: PathBuf },
    Parse { file: PathBuf },
    InitDb { database: PathBuf },
    Import { file: PathBuf, database: PathBuf },
    ImportBook { file: PathBuf, database: PathBuf },
    ListBooks { database: PathBuf },
    CreateBook { database: PathBuf, title: String },
    Export { file: PathBuf, output: PathBuf },
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
    }
}
#[cfg(test)]
mod test;
