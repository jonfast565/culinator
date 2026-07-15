## 0.1.1 - In-process HTTP service

- Replaced Tauri command invocation with a versioned Axum JSON API.
- Added `culinograph-service` as a reusable in-process service crate.
- Added startup retry, health endpoint, graceful cancellation, and configurable frontend base URL.
- Kept browser-only localStorage fallback for standalone UI development.

# Changelog

## 0.3 scaffold

- Added readable typed declarations such as `ingredient flour measured by mass`.
- Retained backwards compatibility with generic declarations such as `Ingredient<Mass>`.
- Added optional recipe/process type annotations and `operation ... does ...` syntax.
- Added readable formula bases: `relative to <reference>` and `of total`.
- Added source-backed recipe title editing and quick-add controls.
- Added parsed resource cards.
- Added dependency-flow / earliest-start timeline projection.
- Expanded CodeMirror highlighting and updated examples, grammar, and language documentation.

## In-process service hardening and workspace organization

- Bind the embedded Axum service to an operating-system-selected loopback port.
- Generate a new bearer token for each Tauri launch.
- Add exact origin validation and restricted authenticated CORS.
- Inject runtime service bootstrap into the WebView without Tauri commands.
- Add graceful service shutdown tied to the desktop window lifecycle.
- Add a standalone development service and Vite orchestration script.
- Split the service into auth, error, models, state, and route modules.
- Split CLI recipe and database commands into modules.
- Split frontend transport, recipe editor model/panels, and formula UI into feature modules.
- Add ESLint, Prettier, TypeScript checks, rustfmt/clippy aliases, Make targets, and CI jobs.
- Add architecture and local-service security documentation.

## WebSocket transport scaffold

- Added authenticated WebSocket RPC at `/ws`.
- Moved all desktop GUI recipe, validation, and formula traffic to one persistent socket.
- Added exact-origin checks and per-launch token authentication through the WebSocket subprotocol handshake.
- Added request correlation, timeouts, bounded exponential reconnect, and server events.
- Added live recipe/formula change notifications and visible GUI connection status.
- Kept HTTP endpoints for external integrations and diagnostics.

## Recipe-book organization

- Added `RecipeBook` and `Document` domain types.
- Added `book { recipe { ... } }` DSL documents.
- Added normalized SQLite recipe-book storage and recipe ordering.
- Added HTTP and WebSocket book CRUD plus recipe-move operations.
- Added CLI book import, creation, and listing commands.
- Added recipe-book navigation and recipe reassignment in the Tauri GUI.

## Vue frontend and module cleanup

- Replaced the React desktop application with Vue 3 and Vue single-file components.
- Split recipe-book navigation, editing, formula calculation, API access, browser persistence, and
  WebSocket transport into explicit feature and service modules.
- Replaced the monolithic frontend stylesheet with base, layout, and component style layers.
- Added Vue-aware ESLint, `vue-tsc`, Prettier, and Vite build verification.
- Preserved the Tauri-hosted authenticated WebSocket service and Rust crate boundaries.

## Unified recipe export

- Added `culinograph-export` with static HTML, Recipe JSON-LD, SVG Nutrition Facts, manifest, and ZIP bundle generation.
- Added `RecipeExporter` port and application `ExportService`.
- Added authenticated HTTP and WebSocket export endpoints.
- Added CLI `export` command and Vue export panel.

## OCR and AI import

- Added camera/image recipe import in Vue.
- Added replaceable OCR, AI interpreter, and settings interfaces.
- Added optional local Tesseract OCR and OpenAI Responses API conversion to validated Culinograph DSL.
- Added authenticated WebSocket RPC methods for import settings and translation.

## Lossless parser scaffold

- Added a byte-preserving concrete syntax tree with trivia and unknown-token retention.
- Added source ranges, non-overlapping text edits, exact round trips, and semantic reprojection.
- Added block-comment support to semantic parsing and parser-layer tests.

## Visual authoring and scheduling
- Added source-preserving Vue visual authoring for titles and operations.
- Added `culinograph-scheduler` and the replaceable `RecipeScheduler` port.
- Added dependency/resource-aware schedule generation and WebSocket RPC.
- Replaced the timeline placeholder with an interactive Gantt view and critical-path highlighting.
