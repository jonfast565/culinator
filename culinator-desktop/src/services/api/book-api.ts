import type { RecipeBookSummary } from "../../domain/types";
import { serviceRequest } from "../transport/websocket-client";

export async function listRecipeBooks(): Promise<RecipeBookSummary[]> {
  return serviceRequest("/api/v1/books");
}
export async function createRecipeBook(
  title: string,
  description?: string,
): Promise<RecipeBookSummary> {
  return serviceRequest("/api/v1/books", {
    method: "POST",
    body: JSON.stringify({ title, description }),
  });
}
export async function updateRecipeBook(
  id: string,
  title: string,
  description?: string,
): Promise<RecipeBookSummary> {
  return serviceRequest(`/api/v1/books/${encodeURIComponent(id)}`, {
    method: "PUT",
    body: JSON.stringify({ title, description }),
  });
}
export async function deleteRecipeBook(id: string): Promise<void> {
  return serviceRequest(`/api/v1/books/${encodeURIComponent(id)}`, { method: "DELETE" });
}
