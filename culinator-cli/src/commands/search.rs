use anyhow::{Context, Result};
use culinator_models::{RecipeSearch, SearchQuery};
use culinator_sqlite::SqliteCatalogRepository;
use std::path::Path;
use uuid::Uuid;

pub fn search_recipes(
    database: &Path,
    query: &str,
    book_id: Option<Uuid>,
    limit: usize,
) -> Result<()> {
    let repository = SqliteCatalogRepository::new(database);
    repository.initialize().context("initialize database")?;
    let hits = repository
        .query(&SearchQuery {
            text: Some(query.to_owned()),
            book_id,
            exclude_allergens: Vec::new(),
            max_active_minutes: None,
            hydration: None,
            limit,
        })
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if hits.is_empty() {
        println!("No matches.");
        return Ok(());
    }
    for hit in hits {
        println!("{:.2}\t{}\t{}", hit.score, hit.recipe_id, hit.title);
        if !hit.snippet.is_empty() {
            println!("  {}", hit.snippet);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "search/test.rs"]
mod search_test;
