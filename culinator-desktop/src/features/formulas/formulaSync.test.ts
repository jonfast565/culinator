/**
 * Round-trip helpers for formulaSync — keep apply-to-recipe from drifting off
 * the parser.
 */
import { beforeAll, describe, expect, it } from "vitest";
import { parseUiModel } from "../recipe-editor/model";
import { applyFormulaToSource, formulaFromUi, looksLikeBreadRecipe } from "./formulaSync";
import { loadParser } from "../recipe-builder/test-support";
import type { Formula } from "../../domain/types";

beforeAll(loadParser);

describe("formulaSync", () => {
  it("writes a formula block the parser accepts and projects", () => {
    const source = `culinator 0.3;
recipe loaf {
    title "Loaf";
    ingredient flour measured by mass { quantity 500 g; }
    ingredient water measured by mass { quantity 350 g; }
}
`;
    const formula: Formula = {
      id: "00000000-0000-0000-0000-000000000001",
      recipe_id: "r1",
      symbol: "dough",
      name: "Dough",
      basis: "reference_percent",
      ingredients: [
        {
          id: "a",
          symbol: "flour",
          name: "flour",
          stage: "final",
          basis: "reference_percent",
          percentage: 100,
          mass_grams: 500,
          is_reference: true,
          is_flour: true,
          water_fraction: 0,
          scalable: true,
          properties: {},
        },
        {
          id: "b",
          symbol: "water",
          name: "water",
          stage: "final",
          basis: "reference_percent",
          percentage: 70,
          mass_grams: 350,
          is_reference: false,
          is_flour: false,
          water_fraction: 1,
          scalable: true,
          properties: {},
        },
      ],
      properties: { target: "850 g" },
    };
    const next = applyFormulaToSource(source, formula, {
      target_mass_grams: 850,
      reference_mass_grams: 500,
      total_flour_grams: 500,
      total_mass_grams: 850,
      hydration_percent: 70,
      prefermented_flour_percent: 0,
      lines: [
        {
          ingredient_id: "a",
          symbol: "flour",
          name: "flour",
          stage: "final",
          percentage: 100,
          mass_grams: 500,
          is_reference: true,
          is_flour: true,
          total_percentage: 58.8,
        },
        {
          ingredient_id: "b",
          symbol: "water",
          name: "water",
          stage: "final",
          percentage: 70,
          mass_grams: 350,
          is_reference: false,
          is_flour: false,
          total_percentage: 41.2,
        },
      ],
    });
    expect(next).toContain("formula dough relative to flour");
    expect(next).toContain("percentage 100%");
    expect(next).toContain("quantity 500 g");
    const model = parseUiModel(next);
    expect(model.diagnostics).toHaveLength(0);
    expect(model.formulas ?? []).toHaveLength(1);
    expect(model.formulas?.[0].ingredients[0].percentage).toBe(100);
    const roundTrip = formulaFromUi(model.formulas![0], "r1", model.resources);
    expect(roundTrip.ingredients[0].is_flour).toBe(true);
    expect(roundTrip.ingredients[1].water_fraction).toBe(1);
  });

  it("does not smash braces when scaled quantities change length", () => {
    // Regression: patching `7 g` → `11.1 g` with stale ranges left
    // `quantity 11.1 g; g;` and glued `}formula` onto the previous block.
    const source = `culinator 0.3;
recipe pizza {
    title "Pizza";
    ingredient flour measured by mass { quantity 400 g; }
    ingredient yeast measured by mass { quantity 7 g; }
    ingredient oil measured by mass { quantity 14 g; }
    formula dough relative to flour {
        target 421 g;
        ingredient flour as Flour<BakersPercent> {
            percentage 100%;
            reference true;
            flour true;
        }
        ingredient yeast as Ingredient<BakersPercent> {
            percentage 1.75%;
        }
        ingredient oil as Ingredient<BakersPercent> {
            percentage 3.5%;
            role fat;
        }
    }
}
`;
    const formula: Formula = {
      id: "00000000-0000-0000-0000-000000000010",
      recipe_id: "r1",
      symbol: "dough",
      name: "dough",
      basis: "reference_percent",
      ingredients: [
        {
          id: "f",
          symbol: "flour",
          name: "flour",
          stage: "final",
          basis: "reference_percent",
          percentage: 100,
          mass_grams: 636,
          is_reference: true,
          is_flour: true,
          water_fraction: 0,
          scalable: true,
          properties: {},
        },
        {
          id: "y",
          symbol: "yeast",
          name: "yeast",
          stage: "final",
          basis: "reference_percent",
          percentage: 1.75,
          mass_grams: 11.1,
          is_reference: false,
          is_flour: false,
          water_fraction: 0,
          scalable: true,
          properties: {},
        },
        {
          id: "o",
          symbol: "oil",
          name: "oil",
          stage: "final",
          basis: "reference_percent",
          percentage: 3.5,
          mass_grams: 22.3,
          is_reference: false,
          is_flour: false,
          water_fraction: 0,
          scalable: true,
          properties: { role: "fat" },
        },
      ],
      properties: { target: "669.4 g" },
    };
    const next = applyFormulaToSource(source, formula, {
      target_mass_grams: 669.4,
      reference_mass_grams: 636,
      total_flour_grams: 636,
      total_mass_grams: 669.4,
      hydration_percent: 0,
      prefermented_flour_percent: 0,
      lines: [
        {
          ingredient_id: "f",
          symbol: "flour",
          name: "flour",
          stage: "final",
          percentage: 100,
          mass_grams: 636,
          is_reference: true,
          is_flour: true,
          total_percentage: 95,
        },
        {
          ingredient_id: "y",
          symbol: "yeast",
          name: "yeast",
          stage: "final",
          percentage: 1.75,
          mass_grams: 11.1,
          is_reference: false,
          is_flour: false,
          total_percentage: 1.7,
        },
        {
          ingredient_id: "o",
          symbol: "oil",
          name: "oil",
          stage: "final",
          percentage: 3.5,
          mass_grams: 22.3,
          is_reference: false,
          is_flour: false,
          total_percentage: 3.3,
        },
      ],
    });
    expect(next).toContain("quantity 11.1 g;");
    expect(next).toContain("quantity 22.3 g;");
    expect(next).not.toMatch(/quantity [^;]+; g;/);
    expect(next).not.toContain("}formula");
    expect(parseUiModel(next).diagnostics).toHaveLength(0);
  });

  it("detects bread-like recipes", () => {
    expect(
      looksLikeBreadRecipe(
        [
          { symbol: "flour", name: "bread flour", kind: "ingredient", measurement: "mass" },
          { symbol: "water", name: "water", kind: "ingredient", measurement: "mass" },
        ],
        false,
      ),
    ).toBe(true);
    expect(
      looksLikeBreadRecipe(
        [{ symbol: "chicken", name: "chicken", kind: "ingredient", measurement: "mass" }],
        false,
      ),
    ).toBe(false);
  });
});
