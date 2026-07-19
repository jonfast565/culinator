# CLAUDE.md

Guidance for Claude Code when working in this repository. Detailed engineering
notes live in [AGENTS.md](./AGENTS.md) — read it before non-trivial changes.

## Always: keep seed recipes current

Sample recipes live only in the Rust service: `culinator-service/src/seed/*.cg`
(43 of them, each with a `section "…";` book chapter).
Recipes are stored exclusively in the backend, which seeds these on startup —
the desktop app has no embedded copies.

When you add or change DSL syntax, **update the seed recipes to use the new
syntax** (where it applies). Verify with `cargo test -p culinator-service` (the
seed parse/schedule tests).

## Other essentials

- The DSL has **one** parser: `culinator-parser`. The desktop app uses it via
  WebAssembly (`culinator-wasm`), so DSL changes land only in `semantic.rs`.
  Rebuild the bindings with `npm run build:wasm`. See [AGENTS.md](./AGENTS.md).
- Grammar reference: `docs/GRAMMAR.ebnf`.
- Frontend gates: `npm run typecheck`, `npm run lint` (zero warnings),
  `npm run format:check`.
