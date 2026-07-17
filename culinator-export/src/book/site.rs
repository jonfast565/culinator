use super::util::{escape, group_recipes};
use crate::{html, label};
use culinator_core::Recipe;
use culinator_models::{ApplicationError, BookExportOptions, ExportFile, NutritionFacts};

const SHELF_CSS: &str = r#"
*{box-sizing:border-box}
body{margin:0;background:#f7f4ed;color:#1d2721;font:17px/1.55 system-ui,sans-serif}
main{max-width:960px;margin:auto;padding:3rem 1.5rem}
h1{font-family:Georgia,serif;font-size:clamp(2rem,6vw,3.5rem)}
section{margin:2.5rem 0}
h2{font-family:Georgia,serif;font-size:1.5rem;border-bottom:2px solid #1d2721;padding-bottom:.35rem}
ul{list-style:none;padding:0;margin:0}
li{margin:.65rem 0}
a{color:#1d2721;text-decoration:none;border-bottom:1px solid rgba(29,39,33,.25)}
a:hover{border-color:#1d2721}
"#;

pub(crate) fn render(
    book_title: &str,
    recipes: &[(Recipe, String)],
    options: &BookExportOptions,
) -> Result<Vec<ExportFile>, ApplicationError> {
    let sections = group_recipes(recipes);
    let recipe_options = super::recipe_options_from_book(options);
    let label_svg = label::render(&NutritionFacts::default());

    let mut files = Vec::new();
    let mut index = format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title}</title><link rel="stylesheet" href="styles.css"/></head><body>
<main><header><h1>{title}</h1>"#,
        title = escape(book_title)
    );
    if let Some(description) = options.description.as_deref() {
        index.push_str(&format!("<p>{}</p>", escape(description)));
    }
    index.push_str("</header>");

    for section in &sections {
        index.push_str(&format!("<section><h2>{}</h2><ul>", escape(&section.title)));
        for entry in &section.recipes {
            let path = format!("recipes/{}/index.html", entry.slug);
            index.push_str(&format!(
                "<li><a href=\"{path}\">{}</a></li>",
                escape(&entry.recipe.title)
            ));
            let page = html::render(entry.recipe, &recipe_options, &label_svg)?;
            files.push(ExportFile {
                path,
                media_type: "text/html; charset=utf-8".to_owned(),
                contents: page.into_bytes(),
            });
        }
        index.push_str("</ul></section>");
    }
    index.push_str("</main></body></html>");

    files.push(ExportFile {
        path: "index.html".to_owned(),
        media_type: "text/html; charset=utf-8".to_owned(),
        contents: index.into_bytes(),
    });
    files.push(ExportFile {
        path: "styles.css".to_owned(),
        media_type: "text/css".to_owned(),
        contents: SHELF_CSS.as_bytes().to_vec(),
    });

    Ok(files)
}
