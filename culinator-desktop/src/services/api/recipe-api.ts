import type { RecipeDocument, RecipeSummary } from "../../domain/types";
import { serviceRequest } from "../transport/websocket-client";

export async function listRecipes(): Promise<RecipeSummary[]> {
  return serviceRequest("/api/v1/recipes");
}
export async function getRecipe(id: string): Promise<RecipeDocument> {
  return serviceRequest(`/api/v1/recipes/${encodeURIComponent(id)}`);
}
export async function createRecipe(bookId?: string | null): Promise<RecipeDocument> {
  return serviceRequest("/api/v1/recipes", { method: "POST", body: JSON.stringify({ bookId }) });
}
export async function saveRecipe(id: string, sourceText: string): Promise<RecipeDocument> {
  return serviceRequest(`/api/v1/recipes/${encodeURIComponent(id)}`, {
    method: "PUT",
    body: JSON.stringify({ sourceText }),
  });
}
export async function deleteRecipe(id: string): Promise<void> {
  return serviceRequest(`/api/v1/recipes/${encodeURIComponent(id)}`, { method: "DELETE" });
}
export async function moveRecipeToBook(
  id: string,
  bookId?: string | null,
  position = 0,
): Promise<void> {
  return serviceRequest(`/api/v1/recipes/${encodeURIComponent(id)}/book`, {
    method: "PUT",
    body: JSON.stringify({ bookId, position }),
  });
}
