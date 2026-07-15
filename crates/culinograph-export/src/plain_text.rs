use crate::content; use culinograph_core::Recipe; use culinograph_models::RecipeExportOptions;
pub(crate) fn render(recipe:&Recipe, options:&RecipeExportOptions)->String { let c=content::extract(recipe); let mut out=format!("{}\n{}\n\n",recipe.title,"=".repeat(recipe.title.chars().count())); if let Some(d)=&options.description { out.push_str(d); out.push_str("\n\n"); } out.push_str("INGREDIENTS\n"); for i in c.ingredients {out.push_str(&format!("- {i}\n"));} out.push_str("\nMETHOD\n"); for (n,i) in c.instructions.iter().enumerate(){out.push_str(&format!("{}. {i}\n",n+1));} out }

#[cfg(test)]
mod test;
