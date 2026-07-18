# Agent notes — Culinator

Working notes for future coding sessions. Keep this current when you learn
something non-obvious. `CLAUDE.md` links here.

## Architecture at a glance

- **Rust workspace** — the domain lives in `culinator-core` / `culinator-models`;
  services in `culinator-application`; concrete adapters in `culinator-parser`,
  `culinator-scheduler`, `culinator-export`, `culinator-import`; the
  WebSocket/SQLite server in `culinator-service`; CLI in `culinator-cli`.
- **Desktop app** — `culinator-desktop/` is a Tauri + Vue 3 (`<script setup lang="ts">`) frontend
  that talks to `culinator-service` over WebSocket. Tests: `npm run typecheck`,
  `npm run lint` (zero warnings), `npm run format:check`.
- **Recipe DSL** — a small `.cg` language. Grammar reference: `docs/GRAMMAR.ebnf`.
- **Prose→DSL conversion rules** — `docs/AI_RECIPE_CONVERSION.md` (what nuance
  must survive: prep descriptors, `divided`, equipment, doneness cues,
  `to_taste`/`size`/`variant`/`note`/`repeat`). The seed recipes were rewritten
  to follow these; that doc's "seed bug" call-outs are worked before/after
  examples.

## Two parsers that must stay in sync (easy to forget)

**Two parsers.** The Rust semantic parser (`culinator-parser/src/semantic.rs`)
is the source of truth for validation, scheduling, export. The frontend has a
*separate* regex parser (`culinator-desktop/src/features/recipe-editor/model.ts`,
`parseUiModel`) that drives the editor UI (outline, ingredients, visual
workflow graph). Any DSL syntax change usually needs to land in **both**, and
they should desugar identically.

**Seed recipes.** Sample recipes live only as Rust `.cg` files in
`culinator-service/src/seed/*.cg` (loaded via `include_str!` in
`culinator-service/src/state.rs`). Recipes are stored exclusively in the
backend, which seeds these on startup — the desktop app has no embedded copies.
When new syntax lands, migrate the seeds to use it (user preference).

## Two prose generators that must also stay in sync

Step/ingredient prose is derived twice: the exporter
(`culinator-export/src/content.rs`, used by plain text, markdown, web/print
HTML, and EPUB) and the frontend narrative
(`culinator-desktop/src/features/recipe-editor/narrative.ts`, used by the
reading page, book previews, and kitchen mode). The sentence heuristics —
multi-word symbol verbs ("mix_dry", "warm_up", "bake_covered"), lay-on verbs
("Top X with Y", "Dip X in Y"), cook-style quantities (fractions, dropped
`count` units, pluralized count nouns), `to_taste` phrasing, and °C/°F
doneness — are mirrored between them; change both. The exporter additionally
weaves in tools/containers/equipment, which the frontend doesn't model yet.

**Prose audit corpus:** `examples/prose-audit/*.cg` holds 18 real recipes
(based.cooking, public domain) converted per `docs/AI_RECIPE_CONVERSION.md`,
chosen to stress the generator (phrasal verbs, internal-temp doneness, repeat
batches, variant groups, divided ingredients). To eyeball the generated prose,
render them with a tiny bin that calls `parse_recipe` + `StaticRecipeExporter`
with the single `PlainText` format (`include_source: false` makes the bundle
archive the bare text file).

## DSL specifics worth remembering

- **Scheduling uses only explicit `after` dependencies** (`culinator-scheduler`).
  Data flow (`produces` → `input`) does **not** create ordering — bindings only
  drive equipment/labor/container resource conflicts. So any sugar that emits an
  operation must produce a *predictable* operation symbol that downstream `after`
  lists can reference.
- **`register_intermediates`** (in `semantic.rs`) auto-creates a `Material`
  (kind `Intermediate`) for any operation output that isn't a declared resource,
  so authors don't declare a `material` for every partial product. `parseUiModel`
  mirrors this so the graph can render intermediate nodes.
- **`prep <verb> <ingredient> [into <output>] (; | { ... })`** desugars to an
  operation named `<verb>_<ingredient>` (`does <verb>`, `input [ingredient]`,
  `produces <output>`, default labor `active`). Default output symbol is
  `<ingredient>_<verb>`. Implemented in both parsers.
- **Resource `state`** (`state ripe;`, `state grated;`, `state melted;`) is just a
  conventional property on a resource block — the generic property path stores it
  in Rust with no special handling; `parseUiModel` lifts it to `UiResource.state`
  and the UI shows a badge. Type-system `states` on `TypeDeclaration` exist but are
  currently unused; a future state machine could formalize this.
- **Prose-nuance fields are typed (not generic properties)** — unlike `state`,
  these get real fields in both parsers and models: on a resource `to_taste`,
  `size`, `variant`, and `notes` (from repeatable `note "…";`); on an operation
  `repeat` and `notes`. `repeat` is the only one with scheduling weight — the
  scheduler treats `duration` as per-repetition and counts `duration × repeat`
  (`culinator-scheduler/src/lib.rs::duration_seconds`). Adding a field to
  `Resource`/`Operation` means updating every struct literal in
  `culinator-parser/src/semantic.rs` **and** the test literals in
  `culinator-scheduler`, `culinator-export`, `culinator-sqlite`, `culinator-models`,
  `culinator-validator`.

## Visual workflow graph

`culinator-desktop/src/features/visual-authoring/components/VisualAuthoringPanel.vue` renders a
layered DAG (HTML nodes over an SVG edge layer): operation + resource nodes, solid
arrows for data flow, dashed for `after`, longest-path layering into "stages" with a
per-stage concurrency read (active hands vs. unattended), labor-colored nodes, and
material/ingredient state badges. Operation nodes stay editable via the inspector
sidebar (edits the original source by byte `range`, so do NOT desugar the whole
source string — it would corrupt every offset).

## Recipe images, sections, and the book UI

- **`section`, `image` (recipe cover), `photo` (per-step)** are DSL properties that
  ride the **generic `property` path** — the Rust `semantic.rs` parser stores them with
  no special arm; only the frontend `parseUiModel` extracts them (`UiRecipeModel.section`
  / `.coverImage`, `UiOperation.photo`). No Rust parser change was needed.
- **Image values are either an external URL/`data:` URI (rendered directly) or an asset
  handle**. Handle bytes live in the `recipe_images` side table (`migrations/009`), served
  by `RecipeImageRepository` (port) → `culinator-sqlite/src/images.rs` →
  `CatalogRepository` supertrait → `ServiceState::{list,get,upload,delete}_recipe_image`
  → WS RPC `images.*` (`ws.rs`). Frontend: `services/api/image-api.ts` (serviceRpc +
  localStorage fallback), resolved by `features/reading/components/RecipeImage.vue`.
- **`recipe_images` is deliberately NOT in `replace_recipe_entities`** — that function
  wipes+reinserts child rows on every `save_recipe`, which would destroy image bytes.
  Images are managed only via the dedicated upload/delete methods and cascade-delete with
  the recipe.
- **Book UI** (`features/bookshelf/`): the shelf → open book (StPageFlip via
  `usePageFlip.ts`) → reading page (`features/reading/`). A book's leaves (cover, auto-TOC
  front matter, section dividers, recipe cards) are built by `bookContents.ts`. Editing is
  the reading page + `EditDrawer.vue` (live preview); `sourcePatch.ts` patches `.cg`
  properties in place.

## Test layout

Sibling `test.rs` modules per source file; `autobins=false` gotcha for `src/bin`
crates. See the user memory note "Test layout convention".
