import type { Formula, FormulaIngredient } from "../../domain/types";
import type { UiResource } from "../recipe-editor/model";
import { convertUnits } from "../../services/api/units-api";
import { parseQuantity, quantityDimension } from "../units/quantityConvert";

/**
 * Ingredients whose name marks them as the flour-equivalent a baker's
 * percentage is taken against. Matching is a substring test on the declared
 * name and symbol, so "wheat flour", "bread_flour" and "semolina" all land.
 */
const FLOUR_WORDS = ["flour", "semolina", "cornmeal", "polenta", "masa", "rye", "spelt", "farina"];

/** Ingredients that count fully towards hydration. */
const WATER_WORDS = ["water", "ice"];

function haystack(resource: UiResource): string {
  return `${resource.name} ${resource.symbol}`.toLowerCase();
}

function matches(resource: UiResource, words: string[]): boolean {
  const text = haystack(resource);
  return words.some((word) => text.includes(word));
}

/**
 * The grams a declared quantity is worth, or `null` when it cannot be known.
 *
 * Mass units convert directly. Volume units use the ingredient density table
 * in Rust (`IngredientDensity`) via `convertUnits` — the same path nutrition
 * uses — so "200 ml" of lukewarm water becomes 200 g instead of a blank row.
 * Count quantities stay `null`; inventing piece weights here would guess wrong.
 */
export async function declaredGrams(
  quantity: string | undefined,
  ingredientHint?: string,
): Promise<number | null> {
  if (!quantity) return null;
  const parsed = parseQuantity(quantity);
  if (!parsed) return null;
  const dimension = quantityDimension(parsed.unit);
  if (dimension !== "mass" && dimension !== "volume") return null;
  if (dimension === "volume" && !ingredientHint?.trim()) return null;
  try {
    const converted = await convertUnits({
      value: parsed.value,
      fromUnit: parsed.unit,
      toUnit: "g",
      ...(dimension === "volume" ? { ingredient: ingredientHint } : {}),
    });
    if (converted.dimension !== "mass") return null;
    return Number.isFinite(converted.value) ? converted.value : null;
  } catch {
    return null;
  }
}

function draft(resource: UiResource, grams: number | null): FormulaIngredient {
  return {
    id: crypto.randomUUID(),
    symbol: resource.symbol,
    name: resource.name || resource.symbol,
    stage: "final",
    basis: "reference_percent",
    percentage: null,
    mass_grams: grams,
    is_reference: false,
    is_flour: matches(resource, FLOUR_WORDS),
    water_fraction: matches(resource, WATER_WORDS) ? 1 : 0,
    scalable: true,
    // Kept so the row can say "recipe says 200 ml" for anything we could not
    // weigh, instead of showing a silently empty cell.
    properties: resource.quantity ? { sourceQuantity: resource.quantity } : {},
  };
}

/**
 * Pick the row every other percentage is stated against: the heaviest flour if
 * the recipe has one, otherwise simply the heaviest ingredient. Recipes with no
 * weighable ingredient at all get no reference, which leaves every percentage
 * blank until the cook fills in a weight.
 */
export function chooseReference(ingredients: FormulaIngredient[]): FormulaIngredient | null {
  const weighable = ingredients.filter((item) => (item.mass_grams ?? 0) > 0);
  if (!weighable.length) return null;
  const heaviest = (pool: FormulaIngredient[]): FormulaIngredient =>
    pool.reduce((best, item) => ((item.mass_grams ?? 0) > (best.mass_grams ?? 0) ? item : best));
  const flours = weighable.filter((item) => item.is_flour);
  return flours.length ? heaviest(flours) : heaviest(weighable);
}

/** Restate every weighed row as a percentage of the reference row. */
export function percentagesFromWeights(ingredients: FormulaIngredient[]): void {
  const reference = ingredients.find((item) => item.is_reference);
  const divisor = reference?.mass_grams ?? 0;
  ingredients.forEach((item) => {
    item.percentage =
      divisor > 0 && item.mass_grams != null ? (item.mass_grams / divisor) * 100 : null;
  });
}

/**
 * Build a starting formula from the recipe the inspector is showing.
 *
 * Mass-measured ingredients arrive with a weight. Volume ingredients use the
 * shared density table when the name is known (water, oil, milk, …); otherwise
 * the gram field stays empty so the cook can supply the weight.
 */
export async function seedFormulaFromRecipe(
  recipeId: string,
  recipeTitle: string,
  resources: UiResource[],
): Promise<Formula> {
  const edible = resources.filter((resource) => resource.kind === "ingredient");
  const ingredients = await Promise.all(
    edible.map(async (resource) => {
      const hint = resource.name || resource.symbol;
      return draft(resource, await declaredGrams(resource.quantity, hint));
    }),
  );
  const reference = chooseReference(ingredients);
  if (reference) reference.is_reference = true;
  percentagesFromWeights(ingredients);

  return {
    id: crypto.randomUUID(),
    recipe_id: recipeId,
    symbol: "main_formula",
    name: recipeTitle ? `${recipeTitle} formula` : "Main formula",
    basis: "reference_percent",
    ingredients,
    properties: {},
  };
}

/** How many rows the seed could actually weigh — drives the empty state copy. */
export function weighedCount(formula: Formula): number {
  return formula.ingredients.filter((item) => (item.mass_grams ?? 0) > 0).length;
}
