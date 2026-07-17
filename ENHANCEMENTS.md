# Culinator Enhancements Roadmap

This document collects intentionally deferred capabilities that can be added without changing the core dependency direction:

```text
core <- application ports <- adapters <- delivery/UI
```

New functionality should normally enter through an application-layer interface and be implemented by one or more adapters.

## 1. OCR and AI recipe-book import

### Goal
Import photographs, scans, PDFs, and complete recipe books, recover text and layout, then convert the result into valid Culinator DSL.

### Proposed application ports

```rust
pub trait OcrEngine {
    fn recognize(&self, input: OcrInput) -> Result<OcrDocument, ImportError>;
}

pub trait RecipeInterpreter {
    fn interpret(&self, document: &OcrDocument) -> Result<ImportDraft, ImportError>;
}

pub trait ImportReviewer {
    fn validate_draft(&self, draft: ImportDraft) -> Result<ReviewedImport, ImportError>;
}
```

### Candidate adapters
- Local image preprocessing using the `image` crate.
- Local OCR through Tesseract bindings or another replaceable Rust OCR adapter.
- PDF page rendering through a PDFium-based adapter.
- OpenAI-backed structured interpretation that receives OCR text, page structure, the current language specification, and a strict output schema.
- Fully local interpretation adapter for offline installations.

### Settings
Store non-secret preferences in an application settings file:

```toml
[import]
ocr_engine = "tesseract"
interpreter = "openai"
model = "configured-model"

[security]
api_key_source = "os-keychain"
```

API keys should be stored in the operating-system credential store, not directly in the settings file or SQLite database.

### Review workflow
1. Import images or PDF pages.
2. Deskew, denoise, crop, and detect page regions.
3. OCR each page while retaining bounding boxes and confidence.
4. Detect recipe boundaries, headings, ingredient tables, instructions, notes, and page continuations.
5. Generate typed DSL as a draft.
6. Parse and validate the draft.
7. Present side-by-side source-page and DSL review.
8. Require explicit confirmation before committing recipes to a book.

## 2. Nutrition database and Nutrition Facts labels

**Status (2026-07):** MVP workflow shipped — FDC catalog search, fuzzy ingredient auto-link, per-recipe FDC links (`resource_nutrition_links`), per-ingredient manual facts and recipe-level manual override (`recipe_nutrition`, `resource_nutrition_manual`), calculate/aggregate from linked + manual sources, and saveable nutrition tab in the editor (`NutritionPanel`). Export panel still supports ad-hoc label edits at export time. Remaining: cooking-loss/retention models, jurisdiction rounding, dedicated label renderers, book export wiring to saved facts.

- Pluggable nutrient-data providers.
- USDA FoodData Central adapter and immutable nutrition snapshots.
- Ingredient matching and manual override workflow. *(partial — fuzzy match + manual entry shipped)*
- Edible portion, preparation loss, cooking loss, moisture loss, and nutrient-retention models.
- Jurisdiction-specific label policies and rounding rules.
- US Nutrition Facts, EU nutrition declaration, and custom analytical reports.
- Allergen aggregation and cross-contact annotations.
- Per-serving and per-100 g projections.
- PDF, SVG, PNG, and print-ready label renderers.

Suggested ports:

```rust
pub trait NutritionProvider { /* lookup and snapshots */ }
pub trait NutritionCalculator { /* recipe aggregation */ }
pub trait NutritionLabelRenderer { /* jurisdiction-specific output */ }
```

## 3. Recipe-book generation and publishing

**Status (2026-07):** Shipped book-level EPUB, print-ready HTML, and static-site export via `RecipeBookExporter` / `books.export` RPC and desktop “Export book” UI. Native PDF rendering remains out of scope (print-to-PDF via browser).

- Book-level title pages, introductions, sections, indexes, and table of contents.
- Recipe ordering, categories, tags, cross-references, and reusable introductory text.
- Multiple layout templates for print, EPUB, HTML, PDF, and web publishing.
- Automatically generated ingredient, allergen, technique, equipment, and time indexes.
- Embedded photos, process diagrams, timelines, and nutrition labels.
- Consistent unit-system conversion across an entire book.
- Print imposition, bleed, margins, page numbering, and accessible electronic output.

Suggested port:

```rust
pub trait RecipeBookRenderer {
    fn render(&self, book: &RecipeBook, options: RenderOptions) -> Result<Artifact, RenderError>;
}
```

## 4. Advanced scheduling and Gantt planning

- Critical-path analysis.
- Earliest/latest start calculation.
- Multiple cooks and labor calendars.
- Exclusive and capacity-based equipment constraints.
- Counter, refrigerator, freezer, proofing, and storage capacity.
- Monitored operations with periodic attention requirements.
- Batch staggering and repeated production cycles.
- Deadline-driven reverse scheduling.
- Probabilistic duration estimates based on prior executions.
- Interactive Gantt chart with safe dependency editing.

## 5. Execution and kitchen mode

- Guided step-by-step execution.
- Concurrent task dashboard.
- Timers derived from operation states.
- Live measurements and observations.
- Pause, resume, skip, retry, substitute, and corrective events.
- Immutable execution history.
- Comparison of planned versus actual timing, yield, temperature, and mass.
- Voice control and accessibility-focused kitchen display.
- Offline-first execution with later synchronization.

## 6. Smart appliance and sensor adapters

- Oven, induction burner, mixer, scale, thermometer, humidity, pH, and water-activity interfaces.
- Capability discovery rather than vendor-specific assumptions.
- Safe command authorization and confirmation policies.
- Sensor event recording in execution histories.
- Simulated appliances for testing and recipe validation.

Suggested ports:

```rust
pub trait ApplianceGateway { /* capabilities and commands */ }
pub trait SensorStream { /* typed observations */ }
```

## 7. Formula and scaling enhancements

**Status (2026-07):** Shipped preferment builders (poolish/biga/levain/sponge/soaker/tangzhong), extended bakery metrics (salt/fat/sugar/effective hydration), and desired-dough-temperature water calculation in core + formulas UI.

- Arbitrary reference groups beyond flour.
- Multiple simultaneous bases.
- Piece count, pan area, pan volume, concentration, molarity-like food ratios, and serving-relative formulas.
- Preferments, levains, soakers, scalds, tangzhong, poolish, biga, and old-dough models.
- Desired dough temperature and water-temperature calculations.
- Salt, sugar, fat, enrichment, inoculation, prefermented flour, and effective hydration metrics.
- Constraint solving when any supported variable is supplied.
- Rounding policies based on available scale precision.
- Minimum practical ingredient quantities and staged premixes.
- Formula inheritance and version comparison.

## 8. Unit and physical-property system

**Status (2026-07):** Shipped dimensional conversion/formatting in `culinator-core` units module, `UnitService`, `units.convert` / `units.format` RPC, and a desktop unit converter widget.

- Full dimensional-analysis library.
- Locale-aware unit formatting.
- Density and temperature-dependent density conversions.
- Ingredient-specific volume-to-mass conversions with provenance.
- Piece-weight distributions and edible-yield factors.
- Pan geometry and surface-area scaling.
- Temperature scales, rates, concentration, humidity, and pressure dimensions.

## 9. Food safety and compliance

- HACCP plans and critical control points.
- Time-temperature safety rules.
- Cooling, reheating, hot-holding, and cold-holding validation.
- pH and water-activity constraints.
- Cleaning and sanitizing procedures.
- Traceability, lots, suppliers, expiration dates, and recall reports.
- Region-specific regulatory rule adapters.
- Explicit warnings that software guidance does not replace professional or regulatory review.

## 10. Inventory, purchasing, and costing

- Ingredient inventory and lot tracking.
- Purchase units versus recipe units.
- Supplier catalogs and price histories.
- Shopping lists aggregated across recipes and books.
- Waste, trim, yield, and spoilage calculations.
- Recipe, batch, serving, and menu costing.
- Labor and energy costing.
- Reorder thresholds and production demand forecasts.

## 11. Reusable components and package registry

- Versioned reusable processes and sub-recipes.
- Namespaces, imports, aliases, and dependency locking.
- A registry for public and private recipe components.
- Semantic versioning and compatibility checks.
- Trust, signatures, provenance, and license metadata.
- Safe update previews showing formula and process changes.

## 12. Language and LSP enhancements

- Lossless concrete syntax tree preserving comments and formatting.
- Incremental parser.
- Accurate source spans for every AST node.
- Type inference with explicit hover explanations.
- Completion based on operation signatures and resource types.
- Go-to-definition, find references, rename, semantic tokens, formatting, and code actions.
- Unit mismatch and unavailable-resource diagnostics.
- Dependency-cycle visualization.
- Refactoring from inline steps to reusable processes.
- DSL schema/version migration tools.

## 13. Visual authoring

- Structured forms that round-trip through a lossless syntax tree.
- Drag-and-drop dependency graph.
- Timeline/Gantt editing.
- Material-flow and container-state visualization.
- Formula spreadsheet view.
- Side-by-side source and structured editing.
- Conflict-aware multi-window editing.
- Recipe comparison and revision diff views.

## 14. Collaboration and synchronization

- User accounts and role-based permissions.
- Local-first synchronization.
- Recipe and book sharing.
- Comments, review requests, and approvals.
- Optimistic concurrency and revision conflicts.
- Real-time collaboration over a transport adapter.
- Audit history and signed releases.
- Self-hosted and managed-cloud deployments.

## 15. Search and discovery

**Status (2026-07):** Shipped FTS5 recipe search with structured filters (allergens, active time, hydration), `search.query` RPC, CLI search command, and shelf/book search UI.

- Full-text search across recipes, books, ingredients, techniques, and notes.
- Typed queries such as hydration range, allergen exclusions, equipment requirements, or maximum active time.
- Similar-recipe detection.
- Ingredient substitution search.
- Semantic search as an optional adapter rather than a core dependency.

## 16. Substitutions and adaptations

- Typed substitution rules with functional roles.
- Dietary and allergen adaptations.
- Scaling effects on process duration and equipment.
- Altitude and atmospheric-pressure adjustments.
- Convection versus conventional oven adaptations.
- Pan substitution and geometry calculations.
- Confidence and required-review metadata for generated adaptations.

## 17. Media and annotation

- Recipe, ingredient, operation, and execution photos.
- Video segments linked to operations.
- Image annotations and crop variants.
- Process endpoint reference images for color and texture.
- Content-addressed media storage.
- Accessible captions and alternative text.

## 18. Import/export ecosystem

**Status (2026-07):** Shipped schema.org JSON-LD / JSON / YAML structured import, book static-site export, and existing recipe-level formats. Common recipe-manager formats and calendar export remain future work.

- JSON and YAML intermediate representation.
- Schema.org Recipe import/export.
- Common recipe-manager formats.
- CSV and spreadsheet formula exchange.
- Markdown recipe cards.
- Static website generation.
- Calendar and production-plan export.
- Stable API SDKs for Rust, TypeScript, Python, and other consumers.

## 19. Analytics and experimentation

- Yield variance, timing variance, bake loss, and execution reliability.
- Controlled recipe experiments with hypotheses and measured outcomes.
- Version-to-version sensory and process comparisons.
- Statistical analysis adapters.
- Dashboards for production trends and bottlenecks.

## 20. Security and privacy enhancements

**Status (2026-07):** Shipped OS keychain + encrypted-file fallback for API keys via `culinator-secrets`; import settings no longer persist plaintext keys.

- OS keychain integration.
- Encrypted local secrets.
- Fine-grained import and network permissions.
- Sandboxed third-party adapters.
- Signed plug-ins.
- Configurable telemetry with opt-in only.
- Data export and complete deletion tools.

## Prioritization suggestion

### Near term
1. Lossless parser and source spans.
2. Complete repository conformance tests reusable by every database adapter.
3. Structured recipe editor that round-trips safely.
4. OCR interface plus local OCR proof of concept.
5. AI interpreter interface with strict validated output.
6. Nutrition provider interface and basic per-serving calculation.

### Medium term
1. Advanced scheduler and kitchen execution mode.
2. Recipe-book renderer.
3. Inventory and costing.
4. Smart-scale and thermometer adapters.
5. Import/export ecosystem.

### Long term
1. Collaboration and synchronization.
2. Package registry.
3. Commercial production planning.
4. Regulatory and food-safety rule packs.
5. Appliance automation.
