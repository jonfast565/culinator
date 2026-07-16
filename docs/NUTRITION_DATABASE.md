# USDA FoodData Central database builder

`culinator-nutrition-fdc` builds a local, searchable SQLite nutrition catalog from the USDA FoodData Central **Full Download of All Data Types** CSV archive.

The importer streams the large CSV files row-by-row. ZIP sources are extracted into a temporary directory and removed after the build. The resulting database keeps the FDC identifiers, food descriptions, branded-food metadata, nutrient definitions, and nutrient values, plus an FTS5 search index.

## Build from an existing archive

```bash
cargo run -p culinator-nutrition-fdc --bin culinator-fdc-build -- \
  --source FoodData_Central_csv_2026-04-30.zip \
  --output data/fdc.sqlite3 \
  --release 2026-04 \
  --replace
```

## Download and build

```bash
cargo run -p culinator-nutrition-fdc --bin culinator-fdc-build -- \
  --download \
  --output data/fdc.sqlite3 \
  --release 2026-04 \
  --replace
```

The default URL is deliberately explicit and versioned. Update it when adopting a newer USDA release rather than silently changing an existing build.

## Interfaces

The `culinator-models` crate owns two replaceable contracts:

- `NutritionCatalog` for food search and nutrient lookup.
- `NutritionImportStore` for bulk import builders.

`SqliteNutritionCatalog` implements both. A PostgreSQL, remote API, or embedded read-only implementation can be introduced without changing consumers.
