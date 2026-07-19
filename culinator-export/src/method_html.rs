//! Shared HTML fragments for the ingredient/equipment/method blocks, used by
//! the web, print, and EPUB renderers so they all present the same structure.

use culinator_narrative::RecipeContent;

pub(crate) fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Ingredient lists, one `<ul>` per variant group with a label paragraph
/// between labeled groups.
pub(crate) fn ingredients_html(content: &RecipeContent) -> String {
    let mut out = String::new();
    for group in &content.ingredient_groups {
        if let Some(label) = &group.label {
            out.push_str(&format!(
                "<p class=\"variant\"><em>{}</em></p>",
                escape(label)
            ));
        }
        out.push_str("<ul>");
        for item in &group.items {
            out.push_str(&format!("<li>{}</li>", escape(item)));
        }
        out.push_str("</ul>");
    }
    out
}

/// Bare `<ul>` of equipment; empty string when the recipe declares none.
pub(crate) fn equipment_html(content: &RecipeContent) -> String {
    if content.equipment.is_empty() {
        return String::new();
    }
    let items = content
        .equipment
        .iter()
        .map(|item| format!("<li>{}</li>", escape(item)))
        .collect::<String>();
    format!("<ul>{items}</ul>")
}

/// Method sections: an optional subheading at `subheading_level`, an optional
/// italic parallelism note, and an `<ol start>` preserving global numbering.
pub(crate) fn method_html(content: &RecipeContent, subheading_level: u8) -> String {
    let mut out = String::new();
    for section in &content.sections {
        if let Some(title) = &section.title {
            out.push_str(&format!(
                "<h{level}>{}</h{level}>",
                escape(title),
                level = subheading_level
            ));
        }
        if let Some(note) = &section.note {
            out.push_str(&format!("<p class=\"note\"><em>{}</em></p>", escape(note)));
        }
        let start = section.steps.first().map(|step| step.number).unwrap_or(1);
        out.push_str(&format!("<ol start=\"{start}\">"));
        for step in &section.steps {
            out.push_str(&format!("<li>{}", escape(&step.text)));
            if let Some(annotation) = step.annotation() {
                out.push_str(&format!(" <small>{}</small>", escape(&annotation)));
            }
            out.push_str("</li>");
        }
        out.push_str("</ol>");
    }
    out
}
