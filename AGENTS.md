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

## One parser, compiled to WebAssembly

There is exactly **one** grammar. `culinator-parser` is the source of truth for
validation, scheduling, and export, and the desktop app calls that same parser
compiled to WASM (`culinator-wasm` -> `culinator-desktop/src/generated/wasm/`).
`parseUiModel` in `features/recipe-editor/model.ts` is now a thin delegation to
it; the old regex parser is gone. A DSL change lands in `semantic.rs` only.

- **Build:** `npm run build:wasm` (wired into `npm run dev` / `npm run build`).
  One-time prerequisites: `rustup target add wasm32-unknown-unknown` and
  `cargo install wasm-bindgen-cli --version 0.2.126`.
- **Projection:** `culinator-wasm/src/ui_model.rs` maps the domain `Recipe` onto
  the editor's `UiRecipeModel` shape. Field names are camelCase to match the
  TypeScript interface exactly — if you add a field to `UiRecipeModel`, add it
  there.
- **Cost:** ~0.17 ms per parse, so it runs synchronously on every keystroke.
  `main.ts` awaits `initParser()` before mounting so every consumer stays sync.

**Error recovery.** `parse_recipe_recovering` returns a partial model plus
`diagnostics` instead of failing on the first syntax error — that is what makes
a live preview possible while a declaration is half-typed. Strict `parse_recipe`
is unchanged and still rejects anything that produces a diagnostic, so
validation/scheduling/export semantics are exactly as before.

**Spans.** `semantic.rs` records byte spans on `Resource`/`Operation`/
`TypeDeclaration` (they used to always be `None`). The inspector patches source
by these ranges, so a `prep` op's span deliberately covers the `prep` text it
was desugared from. Synthesized intermediates have no span — they have no
source.

**Seed recipes.** Sample recipes live only as Rust `.cg` files in
`culinator-service/src/seed/*.cg` (loaded via `include_str!` in
`culinator-service/src/state.rs`). Recipes are stored exclusively in the
backend, which seeds these on startup — the desktop app has no embedded copies.
When new syntax lands, migrate the seeds to use it (user preference).

## CLI and desktop use the same application runtime

Catalog-backed `culinator` commands construct `culinator_service::ServiceState`
in-process (`culinator-cli/src/runtime.rs`) and call the same
`culinator-application` services as the desktop's HTTP/WebSocket adapters. Keep
new CLI catalog features on that path; do not add new direct calls to the legacy
`culinator-sqlite` free functions. File-only commands such as `check` and
`parse` may continue to call parser/validator adapters directly.

**Keep the CLI and desktop in sync as best you can.** Both are just front ends
over the same `ServiceState` services, so a capability exposed in one should be
reachable from the other. When you add or change an application-service workflow,
surface it on **both** surfaces:

- **Desktop** — a WS RPC in `culinator-service/src/ws.rs` plus its
  `services/api/*` caller and UI.
- **CLI** — a subcommand in `culinator-cli/src/main.rs`, wired through
  `catalog.rs` / `workflows.rs` / `imports.rs`. The CLI command surface
  deliberately mirrors the desktop: catalog, import, formula, nutrition, HACCP,
  kitchen (`cook`), image, scheduling, and unit workflows all run the same
  runtime (see the `Bring CLI into application-service parity` commit). Its
  `search` even advertises "the same filters as the desktop."

CLI commands should accept/emit the same request/response structs the WS layer
uses (JSON in, JSON/JSONL/human out via `output.rs`) so the two stay behaviorally
identical. When one surface gains a workflow the other lacks, treat that as drift
to close, not an intentional asymmetry.

## One prose generator

Step sentences, ingredient lines, times, section grouping, and mise en place
are all produced by **`culinator-narrative`**. The static exporters
(`culinator-export`: plain text, markdown, web/print HTML, EPUB) and the desktop
reading page (via `culinator-wasm`) both render from it, so a step reads
identically wherever it appears.

This used to be duplicated in `narrative.ts` and drifted badly: the reading page
dropped step destinations ("Transfer the sauce mix." instead of "…into the
casserole"), omitted "Meanwhile" lead-ins and section parallelism notes
entirely, and rendered "0.5 tsp" where the exporter said "1/2 tsp".
`culinator-wasm/src/test.rs` asserts step-for-step parity against the exporter
across all 43 seeds — if that test fails, the duplication is creeping back.

- **Quantities** are converted and formatted in Rust. `convert_recipe_units`
  restates every amount once, up front, so ingredient lines, per-step amounts,
  oven temperatures, and internal-temp doneness cues all agree. The frontend no
  longer makes a WebSocket round-trip per quantity.
- **Number style** (`NumberStyle::Fractions` / `Decimals`) picks cooking
  fractions ("1/2 tsp") or decimals ("0.5 tsp"), toggled in the reading bar. It
  is an explicit parameter on every formatter down to `format_number`, so
  rendering has no hidden state: `extract_with(recipe, style)` is the entry
  point and `extract(recipe)` is the fractions default.
- **Cost:** ~0.24 ms for a full narrative, so it recomputes on every keystroke.

## Reading-page view settings (mise en place)

`features/reading/composables/useViewSettings.ts` (localStorage + provide/inject,
mirroring `useUnitDisplay`) carries `misePlacement`:

- `top-matter` (default) — traditional recipe card: one ingredient list and one
  equipment list above the method.
- `colocated` — each method section gets a `MiseBlock.vue` listing only what its
  own steps consume, and the top-matter lists are dropped.

`section_mise` walks a section's steps, resolves input bindings against the
resource table, and keeps only ingredients (a `material` input is an earlier
step's product, not something to have on hand). A divided ingredient's
**per-step** amount (`input jack 5 oz;`) wins over its whole-recipe total —
that is the entire point of the layout. Both the layout and its per-step amounts are derived by
`culinator-narrative::section_mise`, so the mise agrees with the prose.

## DSL specifics worth remembering

- **Scheduling uses only explicit `after` dependencies** (`culinator-scheduler`).
  Data flow (`produces` → `input`) does **not** create ordering — bindings only
  drive equipment/labor/container resource conflicts. So any sugar that emits an
  operation must produce a *predictable* operation symbol that downstream `after`
  lists can reference.
- **`register_intermediates`** (in `semantic.rs`) auto-creates a `Material`
  (kind `Intermediate`) for any operation output that isn't a declared resource,
  so authors don't declare a `material` for every partial product. These reach
  the UI through the WASM projection, so the graph renders intermediate nodes.
- **`prep <verb> <ingredient> [into <output>] (; | { ... })`** desugars to an
  operation named `<verb>_<ingredient>` (`does <verb>`, `input [ingredient]`,
  `produces <output>`, default labor `active`). Default output symbol is
  `<ingredient>_<verb>`. The desugared op keeps a span pointing at the original
  `prep` text so the inspector edits what the author wrote.
- **Resource `state`** (`state ripe;`, `state grated;`, `state melted;`) is just a
  conventional property on a resource block — the generic property path stores it
  in Rust with no special handling; the WASM projection lifts it to
  `UiResource.state` and the UI shows a badge. Type-system `states` on `TypeDeclaration` exist but are
  currently unused; a future state machine could formalize this.
- **Prose-nuance fields are typed (not generic properties)** — unlike `state`,
  these get real fields on the domain types: on a resource `to_taste`,
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
