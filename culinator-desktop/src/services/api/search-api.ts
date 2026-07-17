import type { RangeF64, SearchHit, SearchQuery } from "../../domain/types";
import { hasConfiguredService, serviceRpc } from "../transport/websocket-client";

const STORE_KEY = "culinator.demo.recipes";

function localRecipes(): Array<{
  id: string;
  bookId?: string | null;
  title: string;
  sourceText?: string;
}> {
  try {
    const raw = window.localStorage.getItem(STORE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as Record<
      string,
      { id: string; bookId?: string | null; title: string; sourceText?: string }
    >;
    return Object.values(parsed);
  } catch {
    return [];
  }
}

function localSearch(query: SearchQuery): SearchHit[] {
  const term = query.text?.trim().toLowerCase() ?? "";
  const hits = localRecipes()
    .filter((recipe) => query.bookId == null || (recipe.bookId ?? null) === query.bookId)
    .filter((recipe) => {
      if (!term) return true;
      const haystack = `${recipe.title} ${recipe.sourceText ?? ""}`.toLowerCase();
      return term.split(/\s+/).every((token) => haystack.includes(token));
    })
    .slice(0, query.limit ?? 50)
    .map((recipe, index) => ({
      recipeId: recipe.id,
      bookId: recipe.bookId ?? null,
      title: recipe.title,
      snippet: recipe.title,
      score: 1 - index * 0.01,
    }));
  return hits;
}

export async function searchRecipes(query: SearchQuery): Promise<SearchHit[]> {
  if (hasConfiguredService())
    return serviceRpc<SearchHit[]>("search.query", { ...query } as Record<string, unknown>);
  return localSearch(query);
}

export type { RangeF64, SearchHit, SearchQuery };
