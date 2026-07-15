# Culinograph architecture

## Workspace boundaries

- `culinograph-core`: dependency-light domain types and formula evaluation.
- `culinograph-parser`: source text to typed AST. It does not persist or serve data.
- `culinograph-validator`: semantic validation over the typed AST.
- `culinograph-sqlite`: migrations and repository functions.
- `culinograph-service`: authenticated HTTP application layer. Routes are split by domain.
- `culinograph-cli`: thin command-line composition over parser, validator, and repository crates.
- `culinograph-lsp`: editor protocol adapter over parser and validator.
- `apps/desktop/src-tauri`: Tauri lifecycle host only. It starts and stops the HTTP service.
- `apps/desktop/src`: Vue 3 presentation layer, feature composables, and typed WebSocket client.

Dependencies point inward: adapters depend on domain crates, while `culinograph-core` does not depend on Tauri, Axum, SQLite, or the frontend.

## Local desktop service

The desktop host binds Axum to `127.0.0.1:0`, allowing the operating system to select an unused port. A new 128-bit token is generated for every application launch. The endpoint and token are injected into the WebView at page load; recipe operations never use Tauri commands.

Every API request must include:

```http
Authorization: Bearer <launch-token>
Origin: <approved-webview-origin>
```

Production origins are exact-matched. The development service accepts origins passed explicitly with `--origin`.

## Frontend organization

- `api.ts`: recipe/formula API facade and browser demo adapter.
- `lib/service-client.ts`: secure HTTP bootstrap and transport.
- `types.ts`: API contracts.
- `app/App.vue`: thin application composition. Feature state lives in composables, rendering lives in feature components, and transport/persistence live under `services/`.

## Quality gates

`make check` runs formatting, linting, tests, type checking, and the frontend production build. GitHub Actions runs Rust and frontend checks independently.

## Desktop transport

The Vue WebView maintains one authenticated WebSocket to the embedded Axum service. Request/response correlation, server events, reconnection, and lifecycle management live in `apps/desktop/src/services/transport/websocket-client.ts`.

## Recipe-book aggregate

`RecipeBook` is the organizational aggregate above `Recipe`. A recipe may belong to one book in
SQLite and has an explicit `book_position`; deleting a book leaves its recipes intact and unfiled.
Portable DSL book documents may contain multiple complete recipe declarations. The service exposes
book lifecycle and membership over both HTTP and WebSocket, while the desktop UI treats books as
the primary navigation hierarchy.

## Desktop frontend module boundaries

The Vue frontend follows feature-first boundaries:

```text
src/
├── app/                    # application composition only
├── domain/                 # shared transport/domain contracts
├── features/
│   ├── library/            # recipe-book navigation and workflow
│   ├── recipe-editor/      # source editor, parsed projections, diagnostics
│   └── formulas/           # scaling and reverse-percentage tools
├── services/
│   ├── api/                # recipe, book, formula, validation facades
│   └── transport/          # authenticated WebSocket RPC client
├── shared/components/      # presentation-only reusable controls
└── styles/                 # base, layout, and component primitives
```

Vue single-file components are intentionally small. Stateful workflows are implemented as
composables, and components do not access SQLite, WebSocket protocol details, or localStorage
directly. The browser demonstration backend is isolated in `services/api/browser-store.ts`.

## Parser architecture

The parser uses a lossless concrete syntax layer before semantic projection. The concrete layer preserves every token and trivia byte and is the source of truth for editor/LSP modifications. See `docs/LOSSLESS_PARSING.md`.
