import type { RecipeBookSummary, RecipeDocument } from "../../domain/types";

const recipesKey = "culinograph.demo.recipes";
const booksKey = "culinograph.demo.books";

export function readBooks(): RecipeBookSummary[] {
  const stored = localStorage.getItem(booksKey);
  if (stored) return JSON.parse(stored) as RecipeBookSummary[];
  const books: RecipeBookSummary[] = [
    {
      id: crypto.randomUUID(),
      symbol: "favorites",
      title: "Favorites",
      description: "My recipe collection",
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
  const recipes: RecipeDocument[] = [
    {
      id: crypto.randomUUID(),
      bookId,
      symbol: "weeknight_lasagna",
      title: "Weeknight Lasagna",
      protocolVersion: "0.3",
      updatedAt: new Date().toISOString(),
      sourceText: `culinograph 0.3;\n\nrecipe weeknight_lasagna {\n    title "Weeknight Lasagna";\n\n    ingredient tomatoes measured by mass {\n        quantity 800 g;\n    }\n\n    process cooking {\n        operation simmer does heat {\n            input [tomatoes];\n            duration 30 min;\n            labor passive;\n        }\n    }\n}\n`,
    },
  ];
  writeRecipes(recipes);
  return recipes;
}

export function writeRecipes(recipes: RecipeDocument[]): void {
  localStorage.setItem(recipesKey, JSON.stringify(recipes));
}
