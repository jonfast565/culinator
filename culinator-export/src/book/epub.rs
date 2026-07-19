use super::util::{escape, group_recipes};
use crate::{label, method_html};
use culinator_core::Recipe;
use culinator_models::{ApplicationError, BookExportOptions, NutritionFacts};
use std::io::{Cursor, Write};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

pub(crate) fn render(
    book_title: &str,
    recipes: &[(Recipe, String)],
    options: &BookExportOptions,
) -> Result<Vec<u8>, ApplicationError> {
    let sections = group_recipes(recipes);
    let nutrition = NutritionFacts::default();
    let label_svg = if options.include_nutrition {
        label::render(&nutrition)
    } else {
        String::new()
    };

    let mut manifest_items = Vec::new();
    let mut spine_items = Vec::new();
    let mut nav_entries = String::new();
    let mut landmark_entries = String::new();
    let mut files: Vec<(String, String)> = Vec::new();

    manifest_items.push(
        r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>"#
            .to_owned(),
    );
    manifest_items.push(r#"<item id="css" href="style.css" media-type="text/css"/>"#.to_owned());
    if options.include_nutrition {
        manifest_items.push(
            r#"<item id="label" href="nutrition-facts.svg" media-type="image/svg+xml"/>"#
                .to_owned(),
        );
    }

    if options.cover_image.is_some() {
        manifest_items.push(
            r#"<item id="cover" href="cover.xhtml" media-type="application/xhtml+xml"/>"#
                .to_owned(),
        );
        spine_items.push(r#"<itemref idref="cover"/>"#.to_owned());
        nav_entries.push_str(&format!(
            "<li><a href=\"cover.xhtml\">{}</a></li>",
            escape(book_title)
        ));
        landmark_entries.push_str("<li><a epub:type=\"cover\" href=\"cover.xhtml\">Cover</a></li>");
        files.push((
            "OEBPS/cover.xhtml".to_owned(),
            cover_xhtml(book_title, options),
        ));
    }

    nav_entries.push_str("<li><a href=\"nav.xhtml\">Contents</a></li>");

    for section in &sections {
        let section_href = format!("sections/{}.xhtml", section.slug);
        manifest_items.push(format!(
            r#"<item id="sec-{}" href="{section_href}" media-type="application/xhtml+xml"/>"#,
            section.slug
        ));
        nav_entries.push_str(&format!(
            "<li><span>{title}</span><ol>",
            title = escape(&section.title)
        ));
        landmark_entries.push_str(&format!(
            "<li><a epub:type=\"chapter\" href=\"{section_href}\">{title}</a></li>",
            title = escape(&section.title)
        ));

        if options.section_dividers && sections.len() > 1 {
            files.push((
                format!("OEBPS/{section_href}"),
                section_xhtml(&section.title),
            ));
        }

        for entry in &section.recipes {
            let href = format!("recipes/{}.xhtml", entry.slug);
            manifest_items.push(format!(
                r#"<item id="recipe-{}" href="{href}" media-type="application/xhtml+xml"/>"#,
                entry.slug
            ));
            spine_items.push(format!(r#"<itemref idref="recipe-{}"/>"#, entry.slug));
            nav_entries.push_str(&format!(
                "<li><a href=\"{href}\">{}</a></li>",
                escape(&entry.recipe.title)
            ));
            files.push((
                format!("OEBPS/{href}"),
                recipe_xhtml(entry.recipe, options, &label_svg),
            ));
        }
        nav_entries.push_str("</ol></li>");
    }

    let nav = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>Contents</title></head>
<body>
<nav epub:type="toc" id="toc"><ol>{nav_entries}</ol></nav>
<nav epub:type="landmarks" hidden=""><ol>{landmark_entries}</ol></nav>
</body></html>"#,
        nav_entries = nav_entries,
        landmark_entries = landmark_entries
    );

    let author = options
        .author
        .as_deref()
        .map(|value| format!("<dc:creator>{}</dc:creator>", escape(value)))
        .unwrap_or_default();
    let description = options
        .description
        .as_deref()
        .map(|value| format!("<dc:description>{}</dc:description>", escape(value)))
        .unwrap_or_default();

    let opf = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="book-id">
<metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
<dc:identifier id="book-id">urn:uuid:{id}</dc:identifier>
<dc:title>{title}</dc:title>
<dc:language>en</dc:language>
{author}{description}
</metadata>
<manifest>{manifest}</manifest>
<spine>{spine}</spine>
</package>"#,
        id = uuid::Uuid::new_v4(),
        title = escape(book_title),
        author = author,
        description = description,
        manifest = manifest_items.join(""),
        spine = spine_items.join("")
    );

    let container = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
<rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"#;

    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    writer
        .start_file(
            "mimetype",
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
        )
        .map_err(err)?;
    writer.write_all(b"application/epub+zip").map_err(err)?;

    let compressed = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (path, data) in [
        ("META-INF/container.xml", container.as_bytes()),
        ("OEBPS/content.opf", opf.as_bytes()),
        ("OEBPS/nav.xhtml", nav.as_bytes()),
        (
            "OEBPS/style.css",
            b"body{font-family:serif;line-height:1.5;max-width:42em;margin:auto;padding:2em}h1{font-size:2rem}img{max-width:22em}.section-divider{text-align:center;padding:4em 0}" as &[u8],
        ),
    ] {
        writer.start_file(path, compressed).map_err(err)?;
        writer.write_all(data).map_err(err)?;
    }

    for (path, data) in files {
        writer.start_file(path, compressed).map_err(err)?;
        writer.write_all(data.as_bytes()).map_err(err)?;
    }

    if options.include_nutrition {
        writer
            .start_file("OEBPS/nutrition-facts.svg", compressed)
            .map_err(err)?;
        writer.write_all(label_svg.as_bytes()).map_err(err)?;
    }

    writer
        .finish()
        .map(|cursor| cursor.into_inner())
        .map_err(err)
}

fn cover_xhtml(title: &str, options: &BookExportOptions) -> String {
    let subtitle = options.description.as_deref().unwrap_or("");
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><head><title>{title}</title>
<link rel="stylesheet" type="text/css" href="style.css"/></head>
<body><section class="cover"><h1>{title}</h1><p>{subtitle}</p></section></body></html>"#,
        title = escape(title),
        subtitle = escape(subtitle)
    )
}

fn section_xhtml(title: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><head><title>{title}</title>
<link rel="stylesheet" type="text/css" href="style.css"/></head>
<body><section class="section-divider"><h1>{title}</h1></section></body></html>"#,
        title = escape(title)
    )
}

fn recipe_xhtml(recipe: &Recipe, options: &BookExportOptions, label_svg: &str) -> String {
    let content = culinator_narrative::extract(recipe);
    let equipment = method_html::equipment_html(&content);
    let equipment_block = if equipment.is_empty() {
        String::new()
    } else {
        format!("<h2>Equipment</h2>{equipment}")
    };
    let nutrition = if options.include_nutrition && !label_svg.is_empty() {
        "<h2>Nutrition Facts</h2><img src=\"../nutrition-facts.svg\" alt=\"Nutrition Facts\"/>"
            .to_string()
    } else {
        String::new()
    };
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><head><title>{title}</title>
<link rel="stylesheet" type="text/css" href="../style.css"/></head>
<body>
<h1>{title}</h1>
<p>{summary}</p>
<p>{description}</p>
<h2>Ingredients</h2>{ingredients}
{equipment_block}<h2>Method</h2>{method}
{nutrition}
</body></html>"#,
        title = escape(&recipe.title),
        summary = escape(&content.summary),
        description = escape(options.description.as_deref().unwrap_or("")),
        ingredients = method_html::ingredients_html(&content),
        equipment_block = equipment_block,
        method = method_html::method_html(&content, 3),
        nutrition = nutrition
    )
}

fn err<E: std::fmt::Display>(error: E) -> ApplicationError {
    ApplicationError::Internal(error.to_string())
}

#[cfg(test)]
mod test;
