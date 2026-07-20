import { defineConfig } from "vitest/config";

/**
 * Tests are scoped to the recipe builder's emission layer on purpose.
 *
 * `emit.ts` and `edits.ts` are pure `(string, Outline) => string` functions,
 * they are the code most likely to silently corrupt a recipe, and their
 * failures are invisible on screen — a dropped `allergen milk;` looks exactly
 * like nothing happening. The rest of the frontend is covered by `typecheck`,
 * `lint`, and the Rust suites; widening this pattern would pull in jsdom and a
 * component-testing stack that nothing here needs.
 */
export default defineConfig({
  test: {
    environment: "node",
    include: ["src/features/recipe-builder/**/*.test.ts"],
  },
});
