import { defineConfig } from "vitest/config";

/**
 * Tests stay focused on pure transformation and composable behavior.
 *
 * Builder emission tests protect source integrity. Reading tests cover pure
 * allergen aggregation and persisted view settings without pulling in jsdom or
 * a component-testing stack.
 */
export default defineConfig({
  test: {
    environment: "node",
    include: ["src/features/recipe-builder/**/*.test.ts", "src/features/reading/**/*.test.ts"],
  },
});
