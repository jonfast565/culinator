# WebSocket API

The desktop GUI uses one persistent authenticated WebSocket connection for all application data.
HTTP remains available for diagnostics and non-GUI integrations, but the Vue application does not
use it for recipe or formula operations.

## Connection

- URL: `ws://127.0.0.1:<ephemeral-port>/ws`
- Origin: exact-match against the launch allow-list
- Subprotocols:
  - `culinator.v1`
  - `culinator.auth.<per-launch-token>`

The server selects `culinator.v1`; the authentication protocol is only used during the handshake.
The token is generated for every Tauri launch and is never persisted.

## Request envelope

```json
{
  "id": "e611f0c8-8b4d-4ec4-9851-636a1f1cf568",
  "method": "recipes.save",
  "params": {
    "id": "...",
    "sourceText": "culinator 0.2; ..."
  }
}
```

## Response envelope

```json
{
  "id": "e611f0c8-8b4d-4ec4-9851-636a1f1cf568",
  "ok": true,
  "result": {}
}
```

Errors use:

```json
{
  "id": "...",
  "ok": false,
  "error": {
    "code": "operation_failed",
    "message": "Recipe not found"
  }
}
```

## Server events

```json
{
  "event": "recipes.changed",
  "payload": {
    "kind": "saved",
    "id": "..."
  }
}
```

Current event names:

- `service.ready`
- `recipes.changed`
- `formulas.changed`

## RPC methods

- `service.ping`
- `recipes.list`
- `recipes.get`
- `recipes.create`
- `recipes.save`
- `recipes.delete`
- `recipes.validate`
- `formulas.calculate`
- `formulas.percentages`
- `formulas.save`
- `formulas.list`
- `formulas.get`

## Reconnection

The frontend retries with bounded exponential backoff from 250 ms to 5 seconds. Pending requests are
rejected when a connection is interrupted rather than silently replayed, avoiding accidental duplicate
writes. Read operations can be retried by their callers after reconnection.

## Recipe-book methods

- `books.list`
- `books.create` with `{ title, symbol?, description? }`
- `books.update` with `{ id, title, symbol?, description? }`
- `books.delete` with `{ id }`
- `recipes.move` with `{ id, bookId?, position? }`

The service publishes `books.changed` whenever a book or its membership changes.
