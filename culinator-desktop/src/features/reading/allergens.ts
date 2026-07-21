import type { UiRecipeModel } from "../recipe-editor/model";

/** Collect recipe-level allergens using the same sorted, deduplicated semantics as search. */
export function collectRecipeAllergens(model: UiRecipeModel): string[] {
  const unique = new Map<string, string>();
  for (const resource of model.resources) {
    const allergen = resource.allergen?.trim();
    if (allergen) unique.set(allergen.toLocaleLowerCase(), allergen);
  }
  return [...unique.values()].sort((left, right) =>
    left.localeCompare(right, undefined, { sensitivity: "base" }),
  );
}

export function formatAllergen(allergen: string): string {
  const words = allergen.replaceAll("_", " ").trim().toLocaleLowerCase();
  return words ? words[0].toLocaleUpperCase() + words.slice(1) : "";
}
