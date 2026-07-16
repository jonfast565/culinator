# CLAUDE.md

Guidance for Claude Code when working in this repository. Detailed engineering
notes live in [AGENTS.md](./AGENTS.md) — read it before non-trivial changes.

## Always: keep seed recipes current and in sync

Sample recipes exist in **two** places that must match:

- Rust: `culinator-service/src/seed/*.cg`
- Frontend: `culinator-desktop/src/services/api/seed-recipes.ts` (embedded copies)

When you add or change DSL syntax, **update the seed recipes to use the new
syntax** (where it applies) and update **both** copies. Verify with
`cargo test -p culinator-service` (the seed parse/schedule tests) and the
frontend `npm run typecheck`.

## Other essentials

- The DSL has two parsers (Rust semantic = source of truth; frontend
  `parseUiModel` regex = editor UI). DSL changes usually land in both. See
  [AGENTS.md](./AGENTS.md).
- Grammar reference: `docs/GRAMMAR.ebnf`.
- Frontend gates: `npm run typecheck`, `npm run lint` (zero warnings),
  `npm run format:check`.
