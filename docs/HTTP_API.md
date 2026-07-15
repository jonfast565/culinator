# Local HTTP API

The Tauri host embeds `culinograph-service` in the desktop process. It binds to `127.0.0.1:0`, receives an operating-system-selected port, and generates a new launch token.

The WebView receives this runtime bootstrap:

```json
{
  "endpoint": "http://127.0.0.1:<ephemeral-port>",
  "token": "<per-launch-token>"
}
```

Every request requires an approved `Origin` and:

```http
Authorization: Bearer <per-launch-token>
```

## Routes

- `GET /health`
- `GET|POST /api/v1/recipes`
- `GET|PUT|DELETE /api/v1/recipes/{id}`
- `POST /api/v1/validation`
- `POST /api/v1/formulas/calculate`
- `POST /api/v1/formulas/percentages`
- `PUT /api/v1/formulas`
- `GET /api/v1/recipes/{recipe_id}/formulas`
- `GET /api/v1/formulas/{formula_id}`
- `POST /api/v1/formulas/{formula_id}/runs`

## Browser development

Use `npm run dev:service` from `apps/desktop`. It launches the development service, parses its JSON bootstrap, and starts Vite with `VITE_CULINOGRAPH_API_URL` and `VITE_CULINOGRAPH_API_TOKEN` set automatically.

## Recipe books

- `GET /api/v1/books`
- `POST /api/v1/books`
- `PUT /api/v1/books/{id}`
- `DELETE /api/v1/books/{id}`
- `PUT /api/v1/recipes/{id}/book`
