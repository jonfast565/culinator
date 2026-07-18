import type {
  AutoLinkResult,
  FuzzyFoodMatch,
  IngredientManualNutrition,
  NutritionCatalogStatus,
  NutritionFacts,
  NutritionSearchResult,
  RecipeNutritionResult,
  RecipeNutritionState,
  ResourceNutritionLink,
  SaveIngredientManualNutritionRequest,
  SaveRecipeNutritionRequest,
} from "../../domain/types";
import { hasConfiguredService, serviceRpc } from "../transport/websocket-client";

const browserLinks = new Map<string, ResourceNutritionLink[]>();
const browserState = new Map<string, RecipeNutritionState>();

export function emptyNutritionFacts(): NutritionFacts {
  return {
    servingsPerContainer: 1,
    servingSize: "1 serving",
    servingSizeGrams: null,
    calories: 0,
    totalFatGrams: 0,
    saturatedFatGrams: 0,
    transFatGrams: 0,
    cholesterolMilligrams: 0,
    sodiumMilligrams: 0,
    totalCarbohydrateGrams: 0,
    dietaryFiberGrams: 0,
    totalSugarsGrams: 0,
    addedSugarsGrams: 0,
    proteinGrams: 0,
    vitaminDMicrograms: null,
    calciumMilligrams: null,
    ironMilligrams: null,
    potassiumMilligrams: null,
  };
}

export function per100gFacts(): NutritionFacts {
  return {
    ...emptyNutritionFacts(),
    servingSize: "100 g",
    servingSizeGrams: 100,
  };
}

export async function getNutritionStatus(): Promise<NutritionCatalogStatus> {
  if (hasConfiguredService()) {
    return serviceRpc<NutritionCatalogStatus>("nutrition.status", {});
  }
  return { catalogAvailable: false };
}

export async function searchNutritionFoods(
  query: string,
  limit = 20,
): Promise<NutritionSearchResult[]> {
  if (hasConfiguredService())
    return serviceRpc("nutrition.search", { query, limit });
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

export async function fuzzyMatchNutritionFoods(
  query: string,
  limit = 5,
): Promise<FuzzyFoodMatch[]> {
  if (hasConfiguredService()) return serviceRpc("nutrition.fuzzyMatch", { query, limit });
  const results = await searchNutritionFoods(query, limit);
  return results.map((result, index) => ({
    result,
    score: 1 - index * 0.1,
  }));
}

export async function getNutritionState(recipeId: string): Promise<RecipeNutritionState> {
  if (hasConfiguredService()) return serviceRpc("nutrition.getState", { recipeId });
  return (
    browserState.get(recipeId) ?? {
      recipeId,
      links: browserLinks.get(recipeId) ?? [],
      manualIngredients: [],
      manualOverride: false,
      manualFacts: null,
    }
  );
}

export async function saveRecipeNutrition(
  recipeId: string,
  input: SaveRecipeNutritionRequest,
): Promise<RecipeNutritionState> {
  if (hasConfiguredService()) return serviceRpc("nutrition.saveRecipe", { recipeId, ...input });
  const state: RecipeNutritionState = {
    recipeId,
    links: browserLinks.get(recipeId) ?? [],
    manualIngredients: browserState.get(recipeId)?.manualIngredients ?? [],
    manualOverride: input.manualOverride,
    manualFacts: input.facts ?? null,
  };
  browserState.set(recipeId, state);
  return structuredClone(state);
}

export async function saveIngredientManualNutrition(
  recipeId: string,
  input: SaveIngredientManualNutritionRequest,
): Promise<IngredientManualNutrition> {
  if (hasConfiguredService())
    return serviceRpc("nutrition.saveIngredientManual", { recipeId, ...input });
  const entry: IngredientManualNutrition = {
    recipeId,
    resourceSymbol: input.resourceSymbol,
    factsPer100g: structuredClone(input.factsPer100g),
    updatedAt: new Date().toISOString(),
  };
  const state = browserState.get(recipeId) ?? {
    recipeId,
    links: browserLinks.get(recipeId) ?? [],
    manualIngredients: [],
    manualOverride: false,
    manualFacts: null,
  };
  const index = state.manualIngredients.findIndex(
    (item) => item.resourceSymbol === input.resourceSymbol,
  );
  if (index >= 0) state.manualIngredients[index] = entry;
  else state.manualIngredients.push(entry);
  browserState.set(recipeId, state);
  return structuredClone(entry);
}

export async function deleteIngredientManualNutrition(
  recipeId: string,
  resourceSymbol: string,
): Promise<void> {
  if (hasConfiguredService())
    return serviceRpc("nutrition.deleteIngredientManual", { recipeId, resourceSymbol });
  const state = browserState.get(recipeId);
  if (!state) return;
  state.manualIngredients = state.manualIngredients.filter(
    (item) => item.resourceSymbol !== resourceSymbol,
  );
  browserState.set(recipeId, state);
}

export async function autoLinkIngredients(
  recipeId: string,
  input: { minScore?: number; dryRun?: boolean } = {},
): Promise<AutoLinkResult> {
  if (hasConfiguredService()) return serviceRpc("nutrition.autoLink", { recipeId, ...input });
  return { linked: [], skipped: [], suggestions: [] };
}

export async function listNutritionLinks(recipeId: string): Promise<ResourceNutritionLink[]> {
  if (hasConfiguredService()) return serviceRpc("nutrition.listLinks", { recipeId });
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
  const state = browserState.get(recipeId);
  if (state?.manualOverride && state.manualFacts) {
    return {
      facts: structuredClone(state.manualFacts),
      totalMassGrams: 0,
      linkedIngredientCount: 0,
      totalIngredientCount: 0,
      ingredients: [],
      warnings: ["Using saved recipe-level manual nutrition override."],
      manualOverride: true,
      calculated: false,
    };
  }
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
      manual: false,
    })),
    warnings:
      links.length === 0
        ? ["Link ingredients in browser preview mode to simulate calculated facts."]
        : [],
    manualOverride: false,
    calculated: true,
  };
}

export function nutritionFactsFromResult(result: RecipeNutritionResult): NutritionFacts {
  return structuredClone(result.facts);
}
