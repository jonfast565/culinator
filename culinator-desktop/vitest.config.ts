import { defineConfig } from "vitest/config";

/**
 * Tests stay focused on pure transformation and composable behavior.
 *
 * Builder emission tests protect source integrity. Reading tests cover pure
 * allergen aggregation and persisted view settings without pulling in jsdom or
 * a component-testing stack. App tests cover the menu model shared by the
 * in-app menu bar and the native one.
 */
export default defineConfig({
  test: {
    environment: "node",
    include: [
      "src/app/**/*.test.ts",
      "src/features/formulas/**/*.test.ts",
      "src/features/recipe-builder/**/*.test.ts",
      "src/features/recipe-editor/**/*.test.ts",
      "src/features/reading/**/*.test.ts",
      "src/features/units/**/*.test.ts",
    ],
  },
});
