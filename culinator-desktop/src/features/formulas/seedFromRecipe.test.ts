import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  chooseReference,
  declaredGrams,
  percentagesFromWeights,
  seedFormulaFromRecipe,
} from "./seedFromRecipe";
import type { FormulaIngredient } from "../../domain/types";
import type { UiResource } from "../recipe-editor/model";

vi.mock("../../services/api/units-api", () => ({
  convertUnits: vi.fn(async (request: { value: number; fromUnit: string; toUnit: string; ingredient?: string }) => {
    if (request.fromUnit === "g" && request.toUnit === "g") {
      return { value: request.value, unit: "g", dimension: "mass" };
    }
    if (request.fromUnit === "ml" && request.toUnit === "g" && request.ingredient) {
      const hint = request.ingredient.toLowerCase();
      if (hint.includes("water")) {
        return { value: request.value, unit: "g", dimension: "mass" };
      }
      if (hint.includes("oil")) {
        return { value: request.value * 0.91, unit: "g", dimension: "mass" };
      }
    }
    if (request.fromUnit === "ml" && request.toUnit === "g") {
      return { value: request.value, unit: "ml", dimension: "unknown" };
    }
    return { value: request.value, unit: request.fromUnit, dimension: "unknown" };
  }),
}));

function resource(partial: Partial<UiResource> & Pick<UiResource, "symbol">): UiResource {
  return {
    kind: "ingredient",
    measurement: "mass",
    name: partial.symbol,
    ...partial,
  };
}

function item(partial: Partial<FormulaIngredient> & Pick<FormulaIngredient, "symbol">): FormulaIngredient {
  return {
    id: partial.id ?? crypto.randomUUID(),
    symbol: partial.symbol,
    name: partial.name ?? partial.symbol,
    stage: "final",
    basis: "reference_percent",
    percentage: partial.percentage ?? null,
    mass_grams: partial.mass_grams ?? null,
    is_reference: partial.is_reference ?? false,
    is_flour: partial.is_flour ?? false,
    water_fraction: partial.water_fraction ?? 0,
    scalable: true,
    properties: {},
  };
}

describe("declaredGrams", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("converts mass quantities to grams", async () => {
    await expect(declaredGrams("400 g")).resolves.toBe(400);
  });

  it("converts volume quantities when an ingredient density is known", async () => {
    await expect(declaredGrams("200 ml", "lukewarm water")).resolves.toBe(200);
  });

  it("leaves volume blank without an ingredient hint", async () => {
    await expect(declaredGrams("200 ml")).resolves.toBeNull();
  });
});

describe("seedFormulaFromRecipe", () => {
  it("weighs volume water via density so percentages fill in", async () => {
    const formula = await seedFormulaFromRecipe("r1", "Pizza Dough", [
      resource({ symbol: "flour", name: "wheat flour", quantity: "400 g" }),
      resource({ symbol: "water", name: "lukewarm water", quantity: "200 ml" }),
      resource({ symbol: "yeast", name: "dry yeast", quantity: "7 g" }),
    ]);

    const water = formula.ingredients.find((row) => row.symbol === "water");
    expect(water?.mass_grams).toBe(200);
    expect(water?.percentage).toBe(50);
    expect(water?.water_fraction).toBe(1);
  });
});

describe("chooseReference / percentagesFromWeights", () => {
  it("picks the heaviest flour as reference", () => {
    const flour = item({ symbol: "flour", mass_grams: 400, is_flour: true });
    const water = item({ symbol: "water", mass_grams: 200, water_fraction: 1 });
    const picked = chooseReference([flour, water]);
    expect(picked?.symbol).toBe("flour");
    if (picked) picked.is_reference = true;
    percentagesFromWeights([flour, water]);
    expect(flour.percentage).toBe(100);
    expect(water.percentage).toBe(50);
  });
});
