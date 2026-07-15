use crate::{content, label};
use culinograph_core::Recipe;
use culinograph_models::{ApplicationError, RecipeExportOptions};
use std::io::{Cursor, Write};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

pub(crate) fn render(
    recipe: &Recipe,
    options: &RecipeExportOptions,
) -> Result<Vec<u8>, ApplicationError> {
    let content = content::extract(recipe);
    let ingredients = content
        .ingredients
        .iter()
        .map(|item| format!("<li>{}</li>", escape(item)))
        .collect::<String>();
    let instructions = content
        .instructions
        .iter()
        .map(|item| format!("<li>{}</li>", escape(item)))
        .collect::<String>();
    let recipe_xhtml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?><html xmlns="http://www.w3.org/1999/xhtml"><head><title>{title}</title><link rel="stylesheet" type="text/css" href="style.css"/></head><body><h1>{title}</h1><p>{description}</p><h2>Ingredients</h2><ul>{ingredients}</ul><h2>Method</h2><ol>{instructions}</ol><h2>Nutrition Facts</h2><img src="nutrition-facts.svg" alt="Nutrition Facts"/></body></html>"#,
        title = escape(&recipe.title),
        description = escape(options.description.as_deref().unwrap_or("")),
        ingredients = ingredients,
        instructions = instructions
    );
    let nav = format!(
        r#"<?xml version="1.0" encoding="utf-8"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Contents</title></head><body><nav epub:type="toc"><ol><li><a href="recipe.xhtml">{}</a></li></ol></nav></body></html>"#,
        escape(&recipe.title)
    );
    let opf = format!(
        r#"<?xml version="1.0" encoding="utf-8"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="book-id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="book-id">urn:uuid:{}</dc:identifier><dc:title>{}</dc:title><dc:language>en</dc:language></metadata><manifest><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="recipe" href="recipe.xhtml" media-type="application/xhtml+xml"/><item id="css" href="style.css" media-type="text/css"/><item id="label" href="nutrition-facts.svg" media-type="image/svg+xml"/></manifest><spine><itemref idref="recipe"/></spine></package>"#,
        recipe.id,
        escape(&recipe.title)
    );
    let container = r#"<?xml version="1.0" encoding="UTF-8"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#;
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(
            "mimetype",
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
        )
        .map_err(err)?;
    writer.write_all(b"application/epub+zip").map_err(err)?;
    let compressed = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (path,data) in [("META-INF/container.xml",container.as_bytes()),("OEBPS/content.opf",opf.as_bytes()),("OEBPS/nav.xhtml",nav.as_bytes()),("OEBPS/recipe.xhtml",recipe_xhtml.as_bytes()),("OEBPS/style.css",b"body{font-family:serif;line-height:1.5;max-width:42em;margin:auto;padding:2em}img{max-width:22em}" as &[u8])]{writer.start_file(path,compressed).map_err(err)?;writer.write_all(data).map_err(err)?;}
    writer
        .start_file("OEBPS/nutrition-facts.svg", compressed)
        .map_err(err)?;
    writer
        .write_all(label::render(&options.nutrition).as_bytes())
        .map_err(err)?;
    writer.finish().map(|c| c.into_inner()).map_err(err)
}
fn err<E: std::fmt::Display>(e: E) -> ApplicationError {
    ApplicationError::Internal(e.to_string())
}
fn escape(v: &str) -> String {
    v.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod test;
