use culinator_core::{Recipe, Value};

pub(crate) const DEFAULT_SECTION: &str = "Recipes";

pub(crate) fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn slug(value: &str) -> String {
    let mut out = String::new();
    for c in value.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

pub(crate) fn section_of(recipe: &Recipe) -> String {
    recipe
        .properties
        .get("section")
        .and_then(property_text)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SECTION.to_owned())
}

fn property_text(value: &Value) -> Option<String> {
    match value {
        Value::Text(text) | Value::Symbol(text) => Some(text.clone()),
        _ => None,
    }
}

pub(crate) struct GroupedRecipe<'a> {
    pub recipe: &'a Recipe,
    pub _source: &'a str,
    pub slug: String,
}

pub(crate) struct RecipeSection<'a> {
    pub title: String,
    pub slug: String,
    pub recipes: Vec<GroupedRecipe<'a>>,
}

pub(crate) fn group_recipes<'a>(recipes: &'a [(Recipe, String)]) -> Vec<RecipeSection<'a>> {
    let mut order = Vec::new();
    let mut groups: std::collections::BTreeMap<String, Vec<GroupedRecipe<'a>>> =
        std::collections::BTreeMap::new();

    for (recipe, source) in recipes {
        let section = section_of(recipe);
        if !groups.contains_key(&section) {
            order.push(section.clone());
        }
        groups.entry(section).or_default().push(GroupedRecipe {
            recipe,
            _source: source,
            slug: unique_slug(&recipe.title, recipe.id),
        });
    }

    order
        .into_iter()
        .map(|title| {
            let slug = slug(&title);
            let recipes = groups.remove(&title).unwrap_or_default();
            RecipeSection {
                title,
                slug,
                recipes,
            }
        })
        .collect()
}

fn unique_slug(title: &str, id: uuid::Uuid) -> String {
    let base = slug(title);
    if base.is_empty() {
        format!("recipe-{}", &id.to_string()[..8])
    } else {
        base
    }
}
