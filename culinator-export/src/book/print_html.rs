use super::util::{escape, group_recipes};
use crate::content;
use culinator_core::Recipe;
use culinator_models::BookExportOptions;

const CSS: &str = r#"
*{box-sizing:border-box}
body{margin:0;background:#fff;color:#1d2721;font:16px/1.55 Georgia,serif}
.page{max-width:42rem;margin:0 auto;padding:3rem 2rem}
.title-page{text-align:center;padding:6rem 2rem}
.title-page h1{font-size:3rem;margin:0 0 1rem}
.toc h2,.section-divider h2,.recipe h2{font-size:1.5rem;margin:2rem 0 1rem;border-bottom:1px solid #ccc}
.section-divider{text-align:center;padding:5rem 2rem}
.section-divider h2{font-size:2.5rem;border:none}
.recipe{margin-bottom:3rem}
@media print{
  .page-break{page-break-before:always}
  .title-page{page-break-after:always}
  .toc{page-break-after:always}
  .section-divider{page-break-before:always;page-break-after:always}
  .recipe{page-break-before:always}
}
"#;

pub(crate) fn render(
    book_title: &str,
    recipes: &[(Recipe, String)],
    options: &BookExportOptions,
) -> String {
    let sections = group_recipes(recipes);
    let show_dividers = options.section_dividers && sections.len() > 1;
    let mut body = String::new();

    body.push_str(&format!(
        r#"<section class="page title-page"><h1>{title}</h1>"#,
        title = escape(book_title)
    ));
    if let Some(author) = options.author.as_deref() {
        body.push_str(&format!("<p class=\"author\">{}</p>", escape(author)));
    }
    if let Some(description) = options.description.as_deref() {
        body.push_str(&format!("<p>{}</p>", escape(description)));
    }
    body.push_str(&format!(
        "<p>{} recipe{}</p></section>",
        recipes.len(),
        if recipes.len() == 1 { "" } else { "s" }
    ));

    if options.toc {
        body.push_str(r#"<section class="page toc page-break"><h2>Contents</h2><ol>"#);
        for section in &sections {
            if show_dividers {
                body.push_str(&format!(
                    "<li><strong>{}</strong><ol>",
                    escape(&section.title)
                ));
            }
            for entry in &section.recipes {
                body.push_str(&format!(
                    "<li><a href=\"#{slug}\">{title}</a></li>",
                    slug = entry.slug,
                    title = escape(&entry.recipe.title)
                ));
            }
            if show_dividers {
                body.push_str("</ol></li>");
            }
        }
        body.push_str("</ol></section>");
    }

    for section in &sections {
        if show_dividers {
            body.push_str(&format!(
                r#"<section class="page section-divider page-break"><h2>{title}</h2></section>"#,
                title = escape(&section.title)
            ));
        }
        for entry in &section.recipes {
            let content = content::extract(entry.recipe);
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
            body.push_str(&format!(
                r#"<section class="page recipe page-break" id="{slug}">
<h2>{title}</h2>
<h3>Ingredients</h3><ul>{ingredients}</ul>
<h3>Method</h3><ol>{instructions}</ol>
</section>"#,
                slug = entry.slug,
                title = escape(&entry.recipe.title),
                ingredients = ingredients,
                instructions = instructions
            ));
        }
    }

    format!(
        r#"<!doctype html><html lang="en"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title}</title><style>{css}</style></head><body>{body}</body></html>"#,
        title = escape(book_title),
        css = CSS,
        body = body
    )
}

#[cfg(test)]
mod test;
