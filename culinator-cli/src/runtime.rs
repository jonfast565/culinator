use anyhow::{Result, anyhow};
use culinator_models::{RecipeBookSummary, RecipeSummary};
use culinator_service::ServiceState;
use std::path::Path;
use uuid::Uuid;

/// In-process access to the same application services used by the desktop.
pub struct Runtime {
    pub state: ServiceState,
}

impl Runtime {
    pub fn open(database: &Path) -> Result<Self> {
        let settings = database.with_file_name("settings.json");
        let state = ServiceState::sqlite(database.to_path_buf(), settings)
            .map_err(|error| anyhow!(error.to_string()))?;
        Ok(Self { state })
    }

    pub fn recipe(&self, selector: &str) -> Result<RecipeSummary> {
        resolve_recipe(self.state.recipes().list()?, selector)
    }

    pub fn book(&self, selector: &str) -> Result<RecipeBookSummary> {
        resolve_book(self.state.books().list()?, selector)
    }
}

fn resolve_recipe(items: Vec<RecipeSummary>, selector: &str) -> Result<RecipeSummary> {
    if let Ok(id) = Uuid::parse_str(selector) {
        return items
            .into_iter()
            .find(|item| item.id == id)
            .ok_or_else(|| anyhow!("recipe not found: {selector}"));
    }
    resolve_named(
        items,
        selector,
        |item| (&item.symbol, &item.title),
        "recipe",
    )
}

fn resolve_book(items: Vec<RecipeBookSummary>, selector: &str) -> Result<RecipeBookSummary> {
    if let Ok(id) = Uuid::parse_str(selector) {
        return items
            .into_iter()
            .find(|item| item.id == id)
            .ok_or_else(|| anyhow!("recipe book not found: {selector}"));
    }
    resolve_named(
        items,
        selector,
        |item| (&item.symbol, &item.title),
        "recipe book",
    )
}

fn resolve_named<T>(
    items: Vec<T>,
    selector: &str,
    names: impl Fn(&T) -> (&str, &str),
    kind: &str,
) -> Result<T> {
    let mut matches = items.into_iter().filter(|item| {
        let (symbol, title) = names(item);
        symbol == selector || title.eq_ignore_ascii_case(selector)
    });
    let Some(found) = matches.next() else {
        return Err(anyhow!("{kind} not found: {selector}"));
    };
    if matches.next().is_some() {
        return Err(anyhow!("ambiguous {kind} selector: {selector}"));
    }
    Ok(found)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn resolves_recipe_by_symbol() {
        let item = RecipeSummary {
            id: Uuid::new_v4(),
            book_id: None,
            symbol: "toast".into(),
            title: "Toast".into(),
            protocol_version: "0.3".into(),
            updated_at: String::new(),
        };
        assert_eq!(
            resolve_recipe(vec![item.clone()], "toast").expect("resolve"),
            item
        );
    }
}
