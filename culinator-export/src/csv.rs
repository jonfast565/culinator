use crate::content;
use culinator_core::Recipe;
pub(crate) fn render(recipe: &Recipe) -> String {
    let mut out = "position,ingredient\r\n".to_owned();
    for (index, item) in content::extract(recipe).ingredients.iter().enumerate() {
        out.push_str(&(index + 1).to_string());
        out.push(',');
        out.push('"');
        out.push_str(&item.replace('"', "\"\""));
        out.push_str("\"\r\n");
    }
    out
}

#[cfg(test)]
mod test;
