import type {
  NutritionCatalogStatus,
  NutritionFacts,
  NutritionSearchResult,
  RecipeNutritionResult,
  ResourceNutritionLink,
} from "../../domain/types";
import { hasConfiguredService, serviceRequest, serviceRpc } from "../transport/websocket-client";

const browserLinks = new Map<string, ResourceNutritionLink[]>();

export async function getNutritionStatus(): Promise<NutritionCatalogStatus> {
  if (hasConfiguredService()) return serviceRequest("/api/v1/nutrition/status");
  return { catalogAvailable: false };
}

export async function searchNutritionFoods(
  query: string,
  limit = 20,
): Promise<NutritionSearchResult[]> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/nutrition/search?q=${encodeURIComponent(query)}&limit=${limit}`);
  if (!query.trim()) return [];
  return [
    {
      fdcId: 1001,
      description: `${query} (browser preview)`,
      dataType: "SR Legacy",
      brandOwner: null,
      servingSize: 100,
      servingSizeUnit: "g",
    },
  ];
}

export async function listNutritionLinks(recipeId: string): Promise<ResourceNutritionLink[]> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/recipes/${encodeURIComponent(recipeId)}/nutrition/links`);
  return browserLinks.get(recipeId) ?? [];
}

export async function linkIngredientNutrition(
  recipeId: string,
  resourceSymbol: string,
  fdcId: number,
): Promise<ResourceNutritionLink> {
  if (hasConfiguredService())
    return serviceRpc("nutrition.link", { recipeId, resourceSymbol, fdcId });
  const link: ResourceNutritionLink = {
    recipeId,
    resourceSymbol,
    fdcId,
    foodDescription: `FDC ${fdcId}`,
    linkedAt: new Date().toISOString(),
  };
  const links = browserLinks.get(recipeId) ?? [];
  const index = links.findIndex((item) => item.resourceSymbol === resourceSymbol);
  if (index >= 0) links[index] = link;
  else links.push(link);
  browserLinks.set(recipeId, links);
  return structuredClone(link);
}

export async function unlinkIngredientNutrition(
  recipeId: string,
  resourceSymbol: string,
): Promise<void> {
  if (hasConfiguredService()) return serviceRpc("nutrition.unlink", { recipeId, resourceSymbol });
  const links = browserLinks.get(recipeId) ?? [];
  browserLinks.set(
    recipeId,
    links.filter((item) => item.resourceSymbol !== resourceSymbol),
  );
}

export async function calculateRecipeNutrition(
  recipeId: string,
  input: {
    servingsPerContainer?: number;
    servingSize?: string;
    servingSizeGrams?: number | null;
  } = {},
): Promise<RecipeNutritionResult> {
  if (hasConfiguredService()) return serviceRpc("nutrition.calculate", { recipeId, ...input });
  const links = browserLinks.get(recipeId) ?? [];
  const facts: NutritionFacts = {
    servingsPerContainer: input.servingsPerContainer ?? 1,
    servingSize: input.servingSize ?? "1 serving",
    servingSizeGrams: input.servingSizeGrams ?? null,
    calories: links.length > 0 ? 120 : 0,
    totalFatGrams: 4,
    saturatedFatGrams: 1,
    transFatGrams: 0,
    cholesterolMilligrams: 0,
    sodiumMilligrams: 180,
    totalCarbohydrateGrams: 18,
    dietaryFiberGrams: 2,
    totalSugarsGrams: 3,
    addedSugarsGrams: 0,
    proteinGrams: 5,
    vitaminDMicrograms: null,
    calciumMilligrams: null,
    ironMilligrams: null,
    potassiumMilligrams: null,
  };
  return {
    facts,
    totalMassGrams: 250,
    linkedIngredientCount: links.length,
    totalIngredientCount: links.length,
    ingredients: links.map((link) => ({
      resourceSymbol: link.resourceSymbol,
      resourceName: link.resourceSymbol,
      massGrams: 250,
      fdcId: link.fdcId,
      foodDescription: link.foodDescription,
      linked: true,
    })),
    warnings:
      links.length === 0
        ? ["Link ingredients in browser preview mode to simulate calculated facts."]
        : [],
  };
}

export function nutritionFactsFromResult(result: RecipeNutritionResult): NutritionFacts {
  return structuredClone(result.facts);
}
