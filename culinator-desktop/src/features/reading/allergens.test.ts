import { describe, expect, it } from "vitest";
import type { UiRecipeModel } from "../recipe-editor/model";
import { collectRecipeAllergens, formatAllergen } from "./allergens";

function modelWithAllergens(allergens: Array<string | undefined>): UiRecipeModel {
  return {
    title: "Test",
    symbol: "test",
    resources: allergens.map((allergen, index) => ({
      symbol: `ingredient_${index}`,
      name: `Ingredient ${index}`,
      kind: "ingredient",
      measurement: "mass",
      allergen,
    })),
    processes: [],
    operations: [],
    diagnostics: [],
  };
}

describe("collectRecipeAllergens", () => {
  it("sorts, trims, and deduplicates allergens case-insensitively", () => {
    const model = modelWithAllergens([" milk ", "egg", "Milk", undefined, ""]);
    expect(collectRecipeAllergens(model)).toEqual(["egg", "Milk"]);
  });

  it("returns an empty list when no allergens are declared", () => {
    expect(collectRecipeAllergens(modelWithAllergens([undefined]))).toEqual([]);
  });
});

describe("formatAllergen", () => {
  it("turns DSL symbols into readable labels", () => {
    expect(formatAllergen("tree_nut")).toBe("Tree nut");
  });
});
