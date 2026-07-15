import type { RecipeDocument, RecipeSummary } from "../../domain/types";
import { hasConfiguredService, serviceRequest } from "../transport/websocket-client";
import { readRecipes, writeRecipes } from "./browser-store";
import { validateRecipe } from "./validation-api";

export async function listRecipes(): Promise<RecipeSummary[]> {
  if (hasConfiguredService()) return serviceRequest("/api/v1/recipes");
  return readRecipes().map((recipe) => ({
    id: recipe.id,
    bookId: recipe.bookId,
    symbol: recipe.symbol,
    title: recipe.title,
    protocolVersion: recipe.protocolVersion,
    updatedAt: recipe.updatedAt,
  }));
}
export async function getRecipe(id: string): Promise<RecipeDocument> {
  if (hasConfiguredService()) return serviceRequest(`/api/v1/recipes/${encodeURIComponent(id)}`);
  const recipe = readRecipes().find((item) => item.id === id);
  if (!recipe) throw new Error("Recipe not found");
  return recipe;
}
export async function createRecipe(bookId?: string | null): Promise<RecipeDocument> {
  if (hasConfiguredService())
    return serviceRequest("/api/v1/recipes", { method: "POST", body: JSON.stringify({ bookId }) });
  const recipe: RecipeDocument = {
    id: crypto.randomUUID(),
    bookId: bookId ?? null,
    symbol: "new_recipe",
    title: "Untitled Recipe",
    protocolVersion: "0.3",
    updatedAt: new Date().toISOString(),
    sourceText: `culinograph 0.3;\n\nrecipe new_recipe {\n    title "Untitled Recipe";\n}\n`,
  };
  writeRecipes([recipe, ...readRecipes()]);
  return recipe;
}
export async function saveRecipe(id: string, sourceText: string): Promise<RecipeDocument> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/recipes/${encodeURIComponent(id)}`, {
      method: "PUT",
      body: JSON.stringify({ sourceText }),
    });
  const recipes = readRecipes();
  const index = recipes.findIndex((item) => item.id === id);
  if (index < 0) throw new Error("Recipe not found");
  const outline = (await validateRecipe(sourceText)).outline;
  recipes[index] = {
    ...recipes[index],
    sourceText,
    title: outline?.title ?? recipes[index].title,
    symbol: outline?.symbol ?? recipes[index].symbol,
    updatedAt: new Date().toISOString(),
  };
  writeRecipes(recipes);
  return recipes[index];
}
export async function deleteRecipe(id: string): Promise<void> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/recipes/${encodeURIComponent(id)}`, { method: "DELETE" });
  writeRecipes(readRecipes().filter((item) => item.id !== id));
}
export async function moveRecipeToBook(
  id: string,
  bookId?: string | null,
  position = 0,
): Promise<void> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/recipes/${encodeURIComponent(id)}/book`, {
      method: "PUT",
      body: JSON.stringify({ bookId, position }),
    });
  const recipes = readRecipes();
  const recipe = recipes.find((item) => item.id === id);
  if (!recipe) throw new Error("Recipe not found");
  recipe.bookId = bookId ?? null;
  recipe.updatedAt = new Date().toISOString();
  writeRecipes(recipes);
}
