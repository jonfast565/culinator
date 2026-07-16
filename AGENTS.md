# Agent notes — Culinograph

Working notes for future coding sessions. Keep this current when you learn
something non-obvious. `CLAUDE.md` links here.

## Architecture at a glance

- **Rust workspace** — the domain lives in `culinograph-core` / `culinograph-models`;
  services in `culinograph-application`; concrete adapters in `culinograph-parser`,
  `culinograph-scheduler`, `culinograph-export`, `culinograph-import`; the
  WebSocket/SQLite server in `culinograph-service`; CLI in `culinograph-cli`.
- **Desktop app** — `desktop/` is a Tauri + Vue 3 (`<script setup lang="ts">`) frontend
  that talks to `culinograph-service` over WebSocket. Tests: `npm run typecheck`,
  `npm run lint` (zero warnings), `npm run format:check`.
- **Recipe DSL** — a small `.cg` language. Grammar reference: `docs/GRAMMAR.ebnf`.

## Two things that must stay in sync (easy to forget)

1. **Two parsers.** The Rust semantic parser (`culinograph-parser/src/semantic.rs`)
   is the source of truth for validation, scheduling, export. The frontend has a
   *separate* regex parser (`desktop/src/features/recipe-editor/model.ts`,
   `parseUiModel`) that drives the editor UI (outline, ingredients, visual
   workflow graph). Any DSL syntax change usually needs to land in **both**, and
   they should desugar identically.
2. **Two seed copies.** Sample recipes exist as Rust `.cg` files in
   `culinograph-service/src/seed/*.cg` (loaded via `include_str!` in
   `culinograph-service/src/state.rs`) *and* as embedded template strings in
   `desktop/src/services/api/seed-recipes.ts`. Update both; they had already
   drifted once (frontend guac baked prep into ingredient names). When new
   syntax lands, migrate the seeds to use it (user preference).

## DSL specifics worth remembering

- **Scheduling uses only explicit `after` dependencies** (`culinograph-scheduler`).
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

## Visual workflow graph

`desktop/src/features/visual-authoring/components/VisualAuthoringPanel.vue` renders a
layered DAG (HTML nodes over an SVG edge layer): operation + resource nodes, solid
arrows for data flow, dashed for `after`, longest-path layering into "stages" with a
per-stage concurrency read (active hands vs. unattended), labor-colored nodes, and
material/ingredient state badges. Operation nodes stay editable via the inspector
sidebar (edits the original source by byte `range`, so do NOT desugar the whole
source string — it would corrupt every offset).

## Test layout

Sibling `test.rs` modules per source file; `autobins=false` gotcha for `src/bin`
crates. See the user memory note "Test layout convention".
