# Backend implementation

The backend is organized around dependency inversion.

## Crates

- `culinograph-core`: domain entities, quantities, formulas, recipe books, recipes, processes, operations, and calculations. It has no infrastructure dependencies.
- `culinograph-application`: application models, errors, ports, and use-case services. Consumers depend on its traits rather than concrete adapters.
- `culinograph-parser`: Culinograph DSL parser and `DocumentParser` adapter.
- `culinograph-validator`: semantic validation and `RecipeValidator` adapter.
- `culinograph-sqlite`: migrations and `SqliteCatalogRepository`, implementing recipe, recipe-book, and formula repository ports.
- `culinograph-service`: Axum HTTP/WebSocket delivery layer composed from application services.
- `culinograph-cli`: command-line delivery layer.
- `culinograph-lsp`: language-server delivery layer using parser and validation services.
- `apps/desktop/src-tauri`: Tauri composition root and lifecycle host for the in-process service.

## Interface boundaries

The application crate owns these replaceable interfaces:

- `DocumentParser`
- `RecipeValidator`
- `RecipeRepository`
- `RecipeBookRepository`
- `FormulaRepository`
- `CatalogRepository`

SQLite is only one implementation of the repository ports. A PostgreSQL, remote HTTP, in-memory, or browser-backed implementation can be added without changing application services.

## Composition

Concrete adapters are selected only at process composition roots:

```text
CLI/Tauri/service binary
  -> application services
  -> application ports
  -> parser/validator/SQLite adapters
```

Route handlers do not construct database connections or parse source directly. They call application services.
