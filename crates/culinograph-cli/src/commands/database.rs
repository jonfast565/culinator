use anyhow::{Context, Result};
use rusqlite::Connection;
use std::{fs, path::Path};

pub fn init_database(database: &Path) -> Result<()> {
    let connection = Connection::open(database)
        .with_context(|| format!("opening {}", database.display()))?;
    culinograph_sqlite::migrate(&connection)?;
    println!("database initialized: {}", database.display());
    Ok(())
}

pub fn import_recipe(file: &Path, database: &Path) -> Result<()> {
    let source = fs::read_to_string(file)
        .with_context(|| format!("reading {}", file.display()))?;
    let recipe = culinograph_parser::parse_recipe(&source)?;
    let mut connection = Connection::open(database)
        .with_context(|| format!("opening {}", database.display()))?;
    culinograph_sqlite::migrate(&connection)?;
    culinograph_sqlite::save_recipe(&mut connection, &recipe, &source)?;
    println!("imported {}", recipe.title);
    Ok(())
}


pub fn import_recipe_book(file: &Path, database: &Path) -> Result<()> {
    let source = fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
    let book = culinograph_parser::parse_recipe_book(&source)?;
    let mut connection = Connection::open(database).with_context(|| format!("opening {}", database.display()))?;
    culinograph_sqlite::migrate(&connection)?;
    culinograph_sqlite::save_recipe_book(&mut connection, &book)?;
    println!("imported recipe book {} ({} recipes)", book.title, book.recipes.len());
    Ok(())
}

pub fn list_books(database: &Path) -> Result<()> {
    let connection = Connection::open(database).with_context(|| format!("opening {}", database.display()))?;
    culinograph_sqlite::migrate(&connection)?;
    for book in culinograph_sqlite::list_recipe_books(&connection)? {
        println!("{}\t{}\t{} recipes", book.id, book.title, book.recipe_count);
    }
    Ok(())
}

pub fn create_book(database: &Path, title: &str) -> Result<()> {
    use culinograph_core::RecipeBook;
    let mut connection = Connection::open(database).with_context(|| format!("opening {}", database.display()))?;
    culinograph_sqlite::migrate(&connection)?;
    let symbol = title.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>();
    let book = RecipeBook::empty(symbol, title, "0.3");
    culinograph_sqlite::save_recipe_book(&mut connection, &book)?;
    println!("created recipe book {} ({})", book.title, book.id);
    Ok(())
}
#[cfg(test)]
mod test;
