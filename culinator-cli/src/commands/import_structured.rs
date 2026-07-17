use anyhow::{Context, Result};
use culinator_import::StructuredRecipeParser;
use culinator_models::{StructuredInput, StructuredInputFormat, StructuredRecipeImporter};
use culinator_parser::parse_recipe;
use std::{fs, path::Path};

pub fn import_structured(
    file: &Path,
    format: StructuredInputFormat,
    output: Option<&Path>,
) -> Result<()> {
    let content = fs::read_to_string(file).with_context(|| format!("read {}", file.display()))?;
    let draft = StructuredRecipeParser
        .import(StructuredInput { format, content })
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    parse_recipe(&draft.source_text).map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if let Some(path) = output {
        fs::write(path, &draft.source_text).with_context(|| format!("write {}", path.display()))?;
        println!("Wrote {} to {}", draft.title, path.display());
    } else {
        print!("{}", draft.source_text);
    }
    if !draft.warnings.is_empty() {
        eprintln!("Warnings:");
        for warning in &draft.warnings {
            eprintln!("- {warning}");
        }
    }
    Ok(())
}
