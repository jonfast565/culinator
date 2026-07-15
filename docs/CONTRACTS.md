# Shared contracts

`culinograph-models` is the stable dependency boundary for transport-neutral DTOs, errors, and replaceable interfaces. It contains no use-case orchestration and no infrastructure.

Adapters depend directly on `culinograph-models`:

- parser → `DocumentParser`
- validator → `RecipeValidator`
- SQLite catalog → recipe/book/formula repository traits
- FDC nutrition database → `NutritionCatalog` and `NutritionImportStore`

`culinograph-application` depends on those contracts and contains only use-case services. Delivery layers may depend on the application services, but infrastructure should not.
