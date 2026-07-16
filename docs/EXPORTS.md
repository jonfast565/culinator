# Unified recipe export

Culinator exports one ZIP containing a static recipe webpage and its matching nutrition label.

Files:
- `index.html`: responsive standalone recipe page with embedded Nutrition Facts SVG and Recipe JSON-LD.
- `nutrition-facts.svg`: scalable label artwork.
- `recipe.json`: typed recipe AST.
- `manifest.json`: bundle metadata.
- `recipe.cg`: optional original DSL source.

The nutrition label generator is suitable for planning and mockups. Regulatory use requires review of nutrient inputs, rounding, serving definitions, and jurisdiction-specific requirements.

CLI:
```bash
culinator export recipe.cg recipe-export.zip
```

WebSocket RPC: `recipes.export` with `{ id, options }`.
