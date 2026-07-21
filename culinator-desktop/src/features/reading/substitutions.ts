import type { UiResource } from "../recipe-editor/model";

export interface IngredientSubstitution {
  symbol: string;
  ingredient: string;
  alternatives: string[];
}

/** Build the actionable substitution list from authored ingredient metadata. */
export function collectIngredientSubstitutions(resources: UiResource[]): IngredientSubstitution[] {
  return resources
    .filter(
      (resource) =>
        resource.kind === "ingredient" &&
        resource.substitutes?.some((substitute) => substitute.trim().length > 0),
    )
    .map((resource) => ({
      symbol: resource.symbol,
      ingredient: resource.name || resource.symbol.replaceAll("_", " "),
      alternatives: [
        ...new Map(
          resource
            .substitutes!.map((substitute) => substitute.trim())
            .filter(Boolean)
            .map((substitute) => [substitute.toLocaleLowerCase(), substitute]),
        ).values(),
      ],
    }))
    .sort((left, right) => left.ingredient.localeCompare(right.ingredient));
}
