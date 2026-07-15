import type { RecipeBookSummary } from "../../domain/types";
import { hasConfiguredService, serviceRequest } from "../transport/websocket-client";
import { readBooks, readRecipes, writeBooks, writeRecipes } from "./browser-store";

export async function listRecipeBooks(): Promise<RecipeBookSummary[]> {
  if (hasConfiguredService()) return serviceRequest("/api/v1/books");
  const recipes = readRecipes();
  return readBooks().map((book) => ({
    ...book,
    recipeCount: recipes.filter((recipe) => recipe.bookId === book.id).length,
  }));
}
export async function createRecipeBook(
  title: string,
  description?: string,
): Promise<RecipeBookSummary> {
  if (hasConfiguredService())
    return serviceRequest("/api/v1/books", {
      method: "POST",
      body: JSON.stringify({ title, description }),
    });
  const book: RecipeBookSummary = {
    id: crypto.randomUUID(),
    symbol: title
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "_")
      .replace(/^_|_$/g, ""),
    title,
    description,
    protocolVersion: "0.3",
    recipeCount: 0,
    updatedAt: new Date().toISOString(),
  };
  writeBooks([...readBooks(), book]);
  return book;
}
export async function updateRecipeBook(
  id: string,
  title: string,
  description?: string,
): Promise<RecipeBookSummary> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/books/${encodeURIComponent(id)}`, {
      method: "PUT",
      body: JSON.stringify({ title, description }),
    });
  const books = readBooks();
  const book = books.find((item) => item.id === id);
  if (!book) throw new Error("Recipe book not found");
  Object.assign(book, { title, description, updatedAt: new Date().toISOString() });
  writeBooks(books);
  return book;
}
export async function deleteRecipeBook(id: string): Promise<void> {
  if (hasConfiguredService())
    return serviceRequest(`/api/v1/books/${encodeURIComponent(id)}`, { method: "DELETE" });
  writeBooks(readBooks().filter((book) => book.id !== id));
  writeRecipes(
    readRecipes().map((recipe) => (recipe.bookId === id ? { ...recipe, bookId: null } : recipe)),
  );
}
