/** Adjustable type roles for recipe reading / index cards (chrome controls). */
export type RecipeTypeRole = "header" | "body" | "annotation";

/** Relative scale per role — persisted in view settings. */
export type RecipeTypeScale = "sm" | "md" | "lg";

export const RECIPE_TYPE_SCALE_FACTORS: Record<RecipeTypeScale, number> = {
  sm: 0.88,
  md: 1,
  lg: 1.14,
};

export const RECIPE_TYPE_SCALE_LABELS: Record<RecipeTypeScale, string> = {
  sm: "S",
  md: "M",
  lg: "L",
};

export function isRecipeTypeScale(value: string): value is RecipeTypeScale {
  return value === "sm" || value === "md" || value === "lg";
}

export function cycleRecipeTypeScale(current: RecipeTypeScale): RecipeTypeScale {
  return current === "sm" ? "md" : current === "md" ? "lg" : "sm";
}

export function recipeTypeScaleFactor(scale: RecipeTypeScale): number {
  return RECIPE_TYPE_SCALE_FACTORS[scale];
}
