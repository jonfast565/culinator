import { describe, expect, it } from "vitest";
import type { UiResource } from "../recipe-editor/model";
import { collectIngredientSubstitutions } from "./substitutions";

function resource(overrides: Partial<UiResource>): UiResource {
  return {
    symbol: "ingredient",
    name: "Ingredient",
    kind: "ingredient",
    measurement: "mass",
    ...overrides,
  };
}

describe("collectIngredientSubstitutions", () => {
  it("collects authored alternatives for ingredients", () => {
    const substitutions = collectIngredientSubstitutions([
      resource({
        symbol: "milk",
        name: "whole milk",
        substitutes: [" oat milk ", "Oat Milk", "soy milk"],
      }),
      resource({ symbol: "butter", name: "butter", substitutes: ["margarine"] }),
    ]);

    expect(substitutions).toEqual([
      { symbol: "butter", ingredient: "butter", alternatives: ["margarine"] },
      {
        symbol: "milk",
        ingredient: "whole milk",
        alternatives: ["Oat Milk", "soy milk"],
      },
    ]);
  });

  it("ignores non-ingredients and empty alternatives", () => {
    expect(
      collectIngredientSubstitutions([
        resource({ kind: "equipment", substitutes: ["whisk"] }),
        resource({ substitutes: [" ", ""] }),
      ]),
    ).toEqual([]);
  });
});
