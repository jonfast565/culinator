use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn check_recipe(file: &Path) -> Result<()> {
    let source = fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
    let recipe = culinator_parser::parse_recipe(&source)?;
    let diagnostics = culinator_validator::validate(&recipe);
    if diagnostics.is_empty() {
        println!("OK: {}", recipe.title);
    } else {
        for diagnostic in diagnostics {
            println!("{}: {}", diagnostic.code, diagnostic.message);
        }
    }
    Ok(())
}

pub fn parse_recipe(file: &Path) -> Result<()> {
    let source = fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
    let recipe = culinator_parser::parse_recipe(&source)?;
    println!("{}", serde_json::to_string_pretty(&recipe)?);
    Ok(())
}
#[cfg(test)]
mod test;
