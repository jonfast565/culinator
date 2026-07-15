import type { RecipeBookSummary, RecipeDocument } from "../../domain/types";
import { seedRecipes } from "./seed-recipes";

const recipesKey = "culinograph.demo.recipes";
const booksKey = "culinograph.demo.books";

export function readBooks(): RecipeBookSummary[] {
  const stored = localStorage.getItem(booksKey);
  if (stored) return JSON.parse(stored) as RecipeBookSummary[];
  const books: RecipeBookSummary[] = [
    {
      id: crypto.randomUUID(),
      symbol: "sample_recipes",
      title: "Sample Recipes",
      description: "Alton Brown classics to get you started",
      protocolVersion: "0.3",
      recipeCount: 0,
      updatedAt: new Date().toISOString(),
    },
  ];
  writeBooks(books);
  return books;
}

export function writeBooks(books: RecipeBookSummary[]): void {
  localStorage.setItem(booksKey, JSON.stringify(books));
}

export function readRecipes(): RecipeDocument[] {
  const stored = localStorage.getItem(recipesKey);
  if (stored) return JSON.parse(stored) as RecipeDocument[];
  const bookId = readBooks()[0]?.id ?? null;
  const timestamp = new Date().toISOString();
  const recipes: RecipeDocument[] = seedRecipes.map((seed) => ({
    id: crypto.randomUUID(),
    bookId,
    symbol: seed.symbol,
    title: seed.title,
    protocolVersion: "0.3",
    updatedAt: timestamp,
    sourceText: seed.sourceText,
  }));
  writeRecipes(recipes);
  return recipes;
}

export function writeRecipes(recipes: RecipeDocument[]): void {
  localStorage.setItem(recipesKey, JSON.stringify(recipes));
}
