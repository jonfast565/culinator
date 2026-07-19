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

## 21. Home-kitchen workflow features

Concrete capabilities aimed at everyday cooking, meal planning, and library management. Several overlap with §2, §3, §10, §14, §15, and §18; this section captures the user-facing scope and suggested UI entry points.

### Recipe scaling

- Scale a recipe by serving count or an arbitrary multiplier from the reading page and kitchen mode.
- Rewrite ingredient quantities in display and, optionally, persist a scaled copy or kitchen try.
- Respect `divided` ingredients, variant groups, and count-based measures that should not scale.
- Surface yield/serving metadata from the DSL `yield` block when present.

### Shopping lists

- Build checkbox-style shopping lists from one recipe, a selection, or an entire book.
- Merge duplicate ingredients by symbol; show combined quantities with unit normalization.
- Optional grouping by aisle or category (manual tags or heuristics).
- Export or share as plain text, markdown, or CSV for use on a phone at the store.
- See also §10 (inventory and purchasing).

### Meal plans and menus

- Plan recipes across days and meal slots (breakfast, lunch, dinner, etc.).
- Calendar or week-grid view tied to the recipe library.
- “Cook today” opens kitchen mode for the planned slot.
- Aggregate shopping lists and nutrition across a plan.
- See also §10 (menu costing) and §4 (schedule rollup for a day’s production).

### Web and URL import

- Paste a recipe URL and import via JSON-LD extraction when available.
- AI or heuristic fallback for unstructured pages (complementing the existing photo/OCR and structured paste flows).
- Optional browser extension or bookmarklet for one-click capture.
- Preview, validate, and assign to a book before save — same wizard pattern as other import paths.
- See also §1 and §18.

### Pantry and “what can I make?”

- User-maintained pantry: ingredients on hand (symbols or free text).
- Search and rank recipes by pantry overlap (“uses only what I have”, “missing ≤ N items”).
- Highlight gaps and suggest declared DSL `substitutes` where applicable.
- See also §16 (substitutions).

### Reusable ingredient catalog

- Library-wide ingredient definitions reused across recipes (name, default units, nutrition facts).
- Custom nutrition and allergen metadata per catalog entry, not only per-recipe resource links.
- When authoring, pick from the catalog instead of re-entering quantities and facts.
- See also §2 (nutrition database).

### Cost analysis

- Manual or imported price per ingredient; per-recipe, per-serving, and per-batch cost.
- Menu and meal-plan cost rollups.
- Optional labor and energy cost using scheduled active time from §4.
- See also §10.

### Nutrition search, filters, and allergen UX

- Search filters: max calories, min protein, exclude food groups, nutrient ranges.
- Allergen badges on the reading page and in search results when nutrition links exist.
- Editor warnings when linked allergens appear in a recipe (complementing exclude-allergen search).
- Menu- and meal-plan-level nutrition summaries.
- See also §2 and §15.

### Print and export templates

- Preset layout themes for single recipes and books (compact card, full page with photo, index-at-back).
- Template picker before export; extends existing print HTML, EPUB, and static-site output.
- See also §3.

### Sharing and lightweight collaboration

- Share read-only recipe or book bundles (static export, signed link, or importable archive).
- Async handoff before real-time multi-user editing.
- See also §14 (full collaboration and sync).

### Substitution assistant

- “Out of this ingredient?” flow: show declared substitutes, pantry alternatives, and optional nutrition delta.
- Surface DSL `substitutes` fields in reading and kitchen mode, not only in the inspector.
- See also §16.

### Beverage and pairing notes

- Optional pairings section on recipes or book sections (wine, beer, non-alcoholic, serving suggestions).
- Lightweight tags or prose fields; no separate product surface required initially.

### Trash, recovery, and revision history

- Soft-delete recipes and books with restore from trash.
- Optional revision snapshots on save for diff and rollback.
- Kitchen tries already reference `recipe_revision_id`; extend to general recipe history where useful.

### Full library backup and restore

- One-click backup of the entire library (SQLite, `.cg` sources, and recipe images).
- Restore from backup on a new machine or after data loss.
- Complements §20 (export and deletion) for local-first users without cloud sync.

### Bundled recipe library and discovery

- Curated or licensed recipe collections users can browse, preview, and copy into their own books.
- Trending, seasonal, or editorially featured recipes on a home or discovery surface.
- Requires content licensing, updates, and moderation — largely separate from the authoring engine.
- Optional online catalog adapter; offline installs ship with seed recipes only.

### Multi-device cloud sync

- Automatic sync of recipes, books, images, shopping lists, and meal plans across computers.
- Conflict resolution, sync status UI, and optional managed-cloud or self-hosted backend.
- Subscription or infrastructure cost; complements but does not replace local backup (§ above).
- See also §14 (collaboration and synchronization).

### Real-time group cookbook editing

- Shared cookbooks multiple users edit concurrently with live updates.
- Roles, permissions, invitations, and optimistic concurrency or CRDT-style merging.
- Heavier than read-only sharing bundles; likely builds on cloud sync and account model.
- See also §14.

### Dedicated wine and beverage lists

- Standalone wine (and beverage) collections separate from per-recipe pairing notes.
- Vintage, region, grape, food-pairing tags, and links to recipes or menus.
- Extends § “Beverage and pairing notes” when users want a cellar-style catalog, not just inline prose.

### Reading and list accessibility

- Adjustable font size for the reading page, ingredient lists, and library search results.
- Respect system text-size preferences where possible; optional compact vs. comfortable density.
- Low implementation cost; polish for long cooking sessions and accessibility.

## Prioritization suggestion

### Near term
1. Lossless parser and source spans.
2. Complete repository conformance tests reusable by every database adapter.
3. Structured recipe editor that round-trips safely.
4. OCR interface plus local OCR proof of concept.
5. AI interpreter interface with strict validated output.
6. Nutrition provider interface and basic per-serving calculation.

### Suggested next wave (§21 home-kitchen workflows)

Ordered by impact and reuse of existing ingredients, search, kitchen mode, and export infrastructure:

1. Recipe scaling by servings.
2. Shopping lists from recipes and books.
3. Web URL import (JSON-LD first, AI fallback).
4. Pantry list and “recipes I can make” search.
5. Meal plans and menus with plan-level shopping and nutrition rollups.
6. Cost analysis (mirror nutrition linking pattern).
7. Allergen badges and nutrient search filters.
8. Full library backup and restore; trash with restore.
9. Print and export layout templates.
10. Read-only recipe and book sharing bundles.

### Longer term / lower priority (§21 heavier lift)

1. Bundled recipe library and discovery catalog.
2. Multi-device cloud sync.
3. Real-time group cookbook editing.
4. Dedicated wine and beverage lists.
5. Reading and list font-size accessibility (quick win if pulled forward).

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
