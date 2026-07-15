# Export formats

A single export request can include any combination of these formats:

- `web`: responsive `index.html` plus `nutrition-facts.svg`.
- `print_html`: a standalone HTML document intended for browser printing or PDF creation.
- `markdown`: `recipe.md` with ingredients, method, and a nutrition summary.
- `plain_text`: `recipe.txt` for email, notes, and legacy systems.
- `ingredient_csv`: `ingredients.csv` for spreadsheets and purchasing workflows.
- `json`: the complete semantic recipe AST as `recipe.json`.
- `epub`: a self-contained EPUB 3 file with recipe content and Nutrition Facts artwork.

The ZIP always contains `manifest.json`; `recipe.cg` is included when `includeSource` is enabled.
